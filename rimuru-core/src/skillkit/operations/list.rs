use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};

use crate::error::RimuruResult;
use crate::skillkit::{InstalledSkill, SkillKitAgent, SkillKitBridge};

#[derive(Debug, Clone, Default)]
pub struct ListOptions {
    pub agent: Option<SkillKitAgent>,
    pub filter: Option<ListFilter>,
    pub sort_by: Option<ListSortBy>,
    pub include_disabled: bool,
    pub limit: Option<usize>,
}

impl ListOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn for_agent(mut self, agent: SkillKitAgent) -> Self {
        self.agent = Some(agent);
        self
    }

    pub fn with_filter(mut self, filter: ListFilter) -> Self {
        self.filter = Some(filter);
        self
    }

    pub fn sorted_by(mut self, sort_by: ListSortBy) -> Self {
        self.sort_by = Some(sort_by);
        self
    }

    pub fn include_disabled(mut self) -> Self {
        self.include_disabled = true;
        self
    }

    pub fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
}

#[derive(Debug, Clone, Default)]
pub enum ListFilter {
    #[default]
    All,
    Enabled,
    Disabled,
    ByTags(Vec<String>),
    ByAuthor(String),
    ByNamePattern(String),
    InstalledAfter(chrono::DateTime<chrono::Utc>),
    InstalledBefore(chrono::DateTime<chrono::Utc>),
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub enum ListSortBy {
    #[default]
    Name,
    InstalledAt,
    LastUsed,
    AgentCount,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResult {
    pub skills: Vec<InstalledSkill>,
    pub total: usize,
    pub by_agent: HashMap<SkillKitAgent, usize>,
    pub enabled_count: usize,
    pub disabled_count: usize,
}

impl ListResult {
    pub fn is_empty(&self) -> bool {
        self.skills.is_empty()
    }

    pub fn get_for_agent(&self, agent: &SkillKitAgent) -> Vec<&InstalledSkill> {
        self.skills
            .iter()
            .filter(|s| s.installed_for.contains(agent))
            .collect()
    }

    pub fn get_enabled(&self) -> Vec<&InstalledSkill> {
        self.skills.iter().filter(|s| s.enabled).collect()
    }

    pub fn get_disabled(&self) -> Vec<&InstalledSkill> {
        self.skills.iter().filter(|s| !s.enabled).collect()
    }
}

pub struct SkillLister {
    bridge: Arc<SkillKitBridge>,
}

impl SkillLister {
    pub fn new(bridge: Arc<SkillKitBridge>) -> Self {
        Self { bridge }
    }

    pub async fn list(&self, options: ListOptions) -> RimuruResult<ListResult> {
        info!("Listing installed skills with options: {:?}", options);

        let all_skills = if let Some(agent) = options.agent {
            self.bridge.list_for_agent(agent).await?
        } else {
            self.bridge.list_installed().await?
        };

        let mut skills: Vec<InstalledSkill> = all_skills
            .into_iter()
            .filter(|s| {
                if !options.include_disabled && !s.enabled {
                    return false;
                }

                if let Some(ref filter) = options.filter {
                    self.matches_filter(s, filter)
                } else {
                    true
                }
            })
            .collect();

        if let Some(sort_by) = options.sort_by {
            self.sort_skills(&mut skills, sort_by);
        }

        if let Some(limit) = options.limit {
            skills.truncate(limit);
        }

        let total = skills.len();
        let enabled_count = skills.iter().filter(|s| s.enabled).count();
        let disabled_count = skills.iter().filter(|s| !s.enabled).count();

        let mut by_agent: HashMap<SkillKitAgent, usize> = HashMap::new();
        for skill in &skills {
            for agent in &skill.installed_for {
                *by_agent.entry(*agent).or_insert(0) += 1;
            }
        }

        debug!("Found {} installed skills", total);

        Ok(ListResult {
            skills,
            total,
            by_agent,
            enabled_count,
            disabled_count,
        })
    }

    pub async fn list_all(&self) -> RimuruResult<Vec<InstalledSkill>> {
        let options = ListOptions::new().include_disabled();
        let result = self.list(options).await?;
        Ok(result.skills)
    }

    pub async fn list_for_agent(&self, agent: SkillKitAgent) -> RimuruResult<Vec<InstalledSkill>> {
        let options = ListOptions::new().for_agent(agent);
        let result = self.list(options).await?;
        Ok(result.skills)
    }

    pub async fn list_enabled(&self) -> RimuruResult<Vec<InstalledSkill>> {
        let options = ListOptions::new().with_filter(ListFilter::Enabled);
        let result = self.list(options).await?;
        Ok(result.skills)
    }

    pub async fn list_disabled(&self) -> RimuruResult<Vec<InstalledSkill>> {
        let options = ListOptions::new()
            .include_disabled()
            .with_filter(ListFilter::Disabled);
        let result = self.list(options).await?;
        Ok(result.skills)
    }

    pub async fn count(&self) -> RimuruResult<usize> {
        let skills = self.bridge.list_installed().await?;
        Ok(skills.len())
    }

    pub async fn count_for_agent(&self, agent: SkillKitAgent) -> RimuruResult<usize> {
        let skills = self.bridge.list_for_agent(agent).await?;
        Ok(skills.len())
    }

    pub async fn get_stats(&self) -> RimuruResult<ListStats> {
        let all_skills = self.bridge.list_installed().await?;

        let total = all_skills.len();
        let enabled = all_skills.iter().filter(|s| s.enabled).count();
        let disabled = all_skills.iter().filter(|s| !s.enabled).count();

        let mut by_agent: HashMap<SkillKitAgent, usize> = HashMap::new();
        let mut unique_skills_per_agent: HashMap<SkillKitAgent, Vec<String>> = HashMap::new();

        for skill in &all_skills {
            for agent in &skill.installed_for {
                *by_agent.entry(*agent).or_insert(0) += 1;
                unique_skills_per_agent
                    .entry(*agent)
                    .or_default()
                    .push(skill.skill.slug.clone());
            }
        }

        let agents_with_skills = by_agent.len();

        Ok(ListStats {
            total_skills: total,
            enabled_skills: enabled,
            disabled_skills: disabled,
            by_agent,
            agents_with_skills,
        })
    }

    pub async fn find_by_name(&self, name: &str) -> RimuruResult<Option<InstalledSkill>> {
        let all_skills = self.bridge.list_installed().await?;
        Ok(all_skills.into_iter().find(|s| {
            s.skill.name.eq_ignore_ascii_case(name) || s.skill.slug.eq_ignore_ascii_case(name)
        }))
    }

    pub async fn find_by_tags(&self, tags: &[String]) -> RimuruResult<Vec<InstalledSkill>> {
        let options = ListOptions::new().with_filter(ListFilter::ByTags(tags.to_vec()));
        let result = self.list(options).await?;
        Ok(result.skills)
    }

    fn matches_filter(&self, skill: &InstalledSkill, filter: &ListFilter) -> bool {
        match filter {
            ListFilter::All => true,
            ListFilter::Enabled => skill.enabled,
            ListFilter::Disabled => !skill.enabled,
            ListFilter::ByTags(tags) => tags.iter().any(|t| skill.skill.tags.contains(t)),
            ListFilter::ByAuthor(author) => skill
                .skill
                .author
                .as_ref()
                .map(|a| a.eq_ignore_ascii_case(author))
                .unwrap_or(false),
            ListFilter::ByNamePattern(pattern) => {
                skill
                    .skill
                    .name
                    .to_lowercase()
                    .contains(&pattern.to_lowercase())
                    || skill
                        .skill
                        .slug
                        .to_lowercase()
                        .contains(&pattern.to_lowercase())
            }
            ListFilter::InstalledAfter(date) => skill.installed_at > *date,
            ListFilter::InstalledBefore(date) => skill.installed_at < *date,
        }
    }

    fn sort_skills(&self, skills: &mut [InstalledSkill], sort_by: ListSortBy) {
        match sort_by {
            ListSortBy::Name => {
                skills.sort_by(|a, b| a.skill.name.cmp(&b.skill.name));
            }
            ListSortBy::InstalledAt => {
                skills.sort_by(|a, b| b.installed_at.cmp(&a.installed_at));
            }
            ListSortBy::LastUsed => {
                skills.sort_by(|a, b| b.installed_at.cmp(&a.installed_at));
            }
            ListSortBy::AgentCount => {
                skills.sort_by(|a, b| b.installed_for.len().cmp(&a.installed_for.len()));
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListStats {
    pub total_skills: usize,
    pub enabled_skills: usize,
    pub disabled_skills: usize,
    pub by_agent: HashMap<SkillKitAgent, usize>,
    pub agents_with_skills: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_options_builder() {
        let options = ListOptions::new()
            .for_agent(SkillKitAgent::ClaudeCode)
            .sorted_by(ListSortBy::Name)
            .include_disabled()
            .with_limit(10);

        assert_eq!(options.agent, Some(SkillKitAgent::ClaudeCode));
        assert!(matches!(options.sort_by, Some(ListSortBy::Name)));
        assert!(options.include_disabled);
        assert_eq!(options.limit, Some(10));
    }

    #[test]
    fn test_list_filter_default() {
        let filter = ListFilter::default();
        assert!(matches!(filter, ListFilter::All));
    }

    #[test]
    fn test_list_result_empty() {
        let result = ListResult {
            skills: vec![],
            total: 0,
            by_agent: HashMap::new(),
            enabled_count: 0,
            disabled_count: 0,
        };

        assert!(result.is_empty());
        assert!(result.get_enabled().is_empty());
        assert!(result.get_disabled().is_empty());
    }

    #[test]
    fn test_list_sort_by_default() {
        let sort_by = ListSortBy::default();
        assert!(matches!(sort_by, ListSortBy::Name));
    }
}
