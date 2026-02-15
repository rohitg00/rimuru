use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, info};

use crate::error::{RimuruError, RimuruResult};
use crate::skillkit::{SearchFilters, Skill, SkillKitAgent, SkillKitBridge, SkillSearchResult};

#[derive(Debug, Clone, Default)]
pub struct SearchOptions {
    pub query: String,
    pub agent: Option<SkillKitAgent>,
    pub tags: Vec<String>,
    pub author: Option<String>,
    pub page: Option<usize>,
    pub per_page: Option<usize>,
    pub sort_by: Option<SearchSortBy>,
    pub sort_order: Option<SortOrder>,
}

impl SearchOptions {
    pub fn new(query: &str) -> Self {
        Self {
            query: query.to_string(),
            ..Default::default()
        }
    }

    pub fn with_agent(mut self, agent: SkillKitAgent) -> Self {
        self.agent = Some(agent);
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn with_author(mut self, author: &str) -> Self {
        self.author = Some(author.to_string());
        self
    }

    pub fn with_pagination(mut self, page: usize, per_page: usize) -> Self {
        self.page = Some(page);
        self.per_page = Some(per_page);
        self
    }

    pub fn with_sort(mut self, sort_by: SearchSortBy, order: SortOrder) -> Self {
        self.sort_by = Some(sort_by);
        self.sort_order = Some(order);
        self
    }

    pub fn to_filters(&self) -> SearchFilters {
        SearchFilters {
            agent: self.agent,
            tags: self.tags.clone(),
            author: self.author.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub enum SearchSortBy {
    #[default]
    Relevance,
    Downloads,
    Name,
    Updated,
    Created,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub enum SortOrder {
    #[default]
    Descending,
    Ascending,
}

#[derive(Debug, Clone)]
pub struct SearchPagination {
    pub page: usize,
    pub per_page: usize,
    pub total: usize,
    pub total_pages: usize,
    pub has_next: bool,
    pub has_prev: bool,
}

impl SearchPagination {
    pub fn new(page: usize, per_page: usize, total: usize) -> Self {
        let total_pages = total.div_ceil(per_page);
        Self {
            page,
            per_page,
            total,
            total_pages,
            has_next: page < total_pages,
            has_prev: page > 1,
        }
    }
}

#[derive(Debug, Clone)]
pub enum SearchProgress {
    Started,
    Searching { query: String },
    Filtering { count: usize },
    Sorting,
    Completed { total: usize, duration_ms: u64 },
    Error { message: String },
}

pub struct SkillSearcher {
    bridge: Arc<SkillKitBridge>,
}

impl SkillSearcher {
    pub fn new(bridge: Arc<SkillKitBridge>) -> Self {
        Self { bridge }
    }

    pub async fn search(&self, options: SearchOptions) -> RimuruResult<SkillSearchResult> {
        if options.query.is_empty() && options.tags.is_empty() && options.author.is_none() {
            return Err(RimuruError::ValidationError(
                "Search requires at least a query, tags, or author filter".to_string(),
            ));
        }

        info!("Searching marketplace for: {}", options.query);

        let filters =
            if options.agent.is_some() || !options.tags.is_empty() || options.author.is_some() {
                Some(options.to_filters())
            } else {
                None
            };

        let mut result = self.bridge.search(&options.query, filters).await?;

        if let Some(sort_by) = options.sort_by {
            self.sort_results(
                &mut result.skills,
                sort_by,
                options.sort_order.unwrap_or_default(),
            );
        }

        if let (Some(page), Some(per_page)) = (options.page, options.per_page) {
            result.page = page;
            result.per_page = per_page;
            let start = (page - 1) * per_page;
            let end = start + per_page;
            if start < result.skills.len() {
                result.skills = result.skills[start..end.min(result.skills.len())].to_vec();
            } else {
                result.skills = Vec::new();
            }
        }

        debug!("Search returned {} results", result.skills.len());
        Ok(result)
    }

    pub async fn search_with_progress(
        &self,
        options: SearchOptions,
        progress_tx: mpsc::Sender<SearchProgress>,
    ) -> RimuruResult<SkillSearchResult> {
        let start = std::time::Instant::now();

        let _ = progress_tx.send(SearchProgress::Started).await;
        let _ = progress_tx
            .send(SearchProgress::Searching {
                query: options.query.clone(),
            })
            .await;

        let result = self.search(options).await;

        match &result {
            Ok(r) => {
                let _ = progress_tx
                    .send(SearchProgress::Completed {
                        total: r.total,
                        duration_ms: start.elapsed().as_millis() as u64,
                    })
                    .await;
            }
            Err(e) => {
                let _ = progress_tx
                    .send(SearchProgress::Error {
                        message: e.to_string(),
                    })
                    .await;
            }
        }

        result
    }

    pub async fn quick_search(&self, query: &str) -> RimuruResult<Vec<Skill>> {
        let options = SearchOptions::new(query).with_pagination(1, 10);
        let result = self.search(options).await?;
        Ok(result.skills)
    }

    pub async fn search_by_agent(
        &self,
        query: &str,
        agent: SkillKitAgent,
    ) -> RimuruResult<Vec<Skill>> {
        let options = SearchOptions::new(query).with_agent(agent);
        let result = self.search(options).await?;
        Ok(result.skills)
    }

    pub async fn search_by_tags(&self, tags: Vec<String>) -> RimuruResult<Vec<Skill>> {
        let options = SearchOptions {
            query: String::new(),
            tags,
            ..Default::default()
        };
        let result = self.search(options).await?;
        Ok(result.skills)
    }

    pub async fn get_trending(&self, limit: usize) -> RimuruResult<Vec<Skill>> {
        let options = SearchOptions::new("*")
            .with_sort(SearchSortBy::Downloads, SortOrder::Descending)
            .with_pagination(1, limit);
        let result = self.search(options).await?;
        Ok(result.skills)
    }

    pub async fn get_recent(&self, limit: usize) -> RimuruResult<Vec<Skill>> {
        let options = SearchOptions::new("*")
            .with_sort(SearchSortBy::Created, SortOrder::Descending)
            .with_pagination(1, limit);
        let result = self.search(options).await?;
        Ok(result.skills)
    }

    fn sort_results(&self, skills: &mut [Skill], sort_by: SearchSortBy, order: SortOrder) {
        match sort_by {
            SearchSortBy::Name => {
                skills.sort_by(|a, b| a.name.cmp(&b.name));
            }
            SearchSortBy::Downloads => {
                skills.sort_by(|a, b| {
                    let a_downloads = a.downloads.unwrap_or(0);
                    let b_downloads = b.downloads.unwrap_or(0);
                    b_downloads.cmp(&a_downloads)
                });
            }
            SearchSortBy::Updated => {
                skills.sort_by(|a, b| {
                    let a_updated = a.updated_at;
                    let b_updated = b.updated_at;
                    match (b_updated, a_updated) {
                        (Some(b), Some(a)) => b.cmp(&a),
                        (Some(_), None) => std::cmp::Ordering::Less,
                        (None, Some(_)) => std::cmp::Ordering::Greater,
                        (None, None) => std::cmp::Ordering::Equal,
                    }
                });
            }
            SearchSortBy::Created => {
                skills.sort_by(|a, b| {
                    let a_created = a.created_at;
                    let b_created = b.created_at;
                    match (b_created, a_created) {
                        (Some(b), Some(a)) => b.cmp(&a),
                        (Some(_), None) => std::cmp::Ordering::Less,
                        (None, Some(_)) => std::cmp::Ordering::Greater,
                        (None, None) => std::cmp::Ordering::Equal,
                    }
                });
            }
            SearchSortBy::Relevance => {}
        }

        if matches!(order, SortOrder::Ascending) {
            skills.reverse();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_options_builder() {
        let options = SearchOptions::new("react")
            .with_agent(SkillKitAgent::ClaudeCode)
            .with_tags(vec!["frontend".to_string()])
            .with_pagination(1, 20);

        assert_eq!(options.query, "react");
        assert_eq!(options.agent, Some(SkillKitAgent::ClaudeCode));
        assert_eq!(options.tags.len(), 1);
        assert_eq!(options.page, Some(1));
        assert_eq!(options.per_page, Some(20));
    }

    #[test]
    fn test_search_pagination() {
        let pagination = SearchPagination::new(1, 20, 100);
        assert_eq!(pagination.total_pages, 5);
        assert!(pagination.has_next);
        assert!(!pagination.has_prev);

        let pagination = SearchPagination::new(5, 20, 100);
        assert!(!pagination.has_next);
        assert!(pagination.has_prev);
    }

    #[test]
    fn test_search_options_to_filters() {
        let options = SearchOptions::new("test")
            .with_agent(SkillKitAgent::Cursor)
            .with_tags(vec!["rust".to_string()]);

        let filters = options.to_filters();
        assert_eq!(filters.agent, Some(SkillKitAgent::Cursor));
        assert!(filters.tags.contains(&"rust".to_string()));
    }
}
