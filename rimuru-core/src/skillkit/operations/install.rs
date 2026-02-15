use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, info};

use crate::error::{RimuruError, RimuruResult};
use crate::skillkit::{InstalledSkill, SkillKitAgent, SkillKitBridge};

#[derive(Debug, Clone, Default)]
pub struct InstallOptions {
    pub skill_name: String,
    pub agents: Vec<SkillKitAgent>,
    pub install_all: bool,
    pub force: bool,
    pub symlink: bool,
    pub dry_run: bool,
}

impl InstallOptions {
    pub fn new(skill_name: &str) -> Self {
        Self {
            skill_name: skill_name.to_string(),
            ..Default::default()
        }
    }

    pub fn for_agent(mut self, agent: SkillKitAgent) -> Self {
        self.agents.push(agent);
        self
    }

    pub fn for_agents(mut self, agents: Vec<SkillKitAgent>) -> Self {
        self.agents = agents;
        self
    }

    pub fn for_all_agents(mut self) -> Self {
        self.install_all = true;
        self.agents = SkillKitAgent::all().to_vec();
        self
    }

    pub fn force(mut self) -> Self {
        self.force = true;
        self
    }

    pub fn with_symlink(mut self) -> Self {
        self.symlink = true;
        self
    }

    pub fn dry_run(mut self) -> Self {
        self.dry_run = true;
        self
    }

    pub fn validate(&self) -> RimuruResult<()> {
        if self.skill_name.is_empty() {
            return Err(RimuruError::ValidationError(
                "Skill name is required".to_string(),
            ));
        }
        if self.agents.is_empty() && !self.install_all {
            return Err(RimuruError::ValidationError(
                "At least one agent must be specified or use install_all".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallResult {
    pub skill: InstalledSkill,
    pub agents_installed: Vec<SkillKitAgent>,
    pub agents_skipped: Vec<SkillKitAgent>,
    pub agents_failed: Vec<(SkillKitAgent, String)>,
    pub paths: Vec<PathBuf>,
    pub duration_ms: u64,
    pub was_dry_run: bool,
}

impl InstallResult {
    pub fn success_count(&self) -> usize {
        self.agents_installed.len()
    }

    pub fn failure_count(&self) -> usize {
        self.agents_failed.len()
    }

    pub fn is_complete_success(&self) -> bool {
        self.agents_failed.is_empty() && !self.agents_installed.is_empty()
    }

    pub fn is_partial_success(&self) -> bool {
        !self.agents_installed.is_empty() && !self.agents_failed.is_empty()
    }
}

#[derive(Debug, Clone)]
pub enum InstallProgress {
    Started {
        skill_name: String,
        total_agents: usize,
    },
    Downloading {
        skill_name: String,
    },
    Validating,
    InstallingForAgent {
        agent: SkillKitAgent,
        current: usize,
        total: usize,
    },
    AgentCompleted {
        agent: SkillKitAgent,
        success: bool,
    },
    CreatingSymlinks {
        count: usize,
    },
    Completed {
        success_count: usize,
        failure_count: usize,
        duration_ms: u64,
    },
    Error {
        message: String,
    },
}

pub struct SkillInstaller {
    bridge: Arc<SkillKitBridge>,
}

impl SkillInstaller {
    pub fn new(bridge: Arc<SkillKitBridge>) -> Self {
        Self { bridge }
    }

    pub async fn install(&self, options: InstallOptions) -> RimuruResult<InstallResult> {
        options.validate()?;
        let start = std::time::Instant::now();

        info!("Installing skill: {}", options.skill_name);

        if options.dry_run {
            return self.dry_run_install(&options).await;
        }

        let installed = if options.install_all {
            self.bridge.install_for_all(&options.skill_name).await?
        } else {
            self.bridge
                .install(&options.skill_name, &options.agents)
                .await?
        };

        let agents_installed = if options.install_all {
            SkillKitAgent::all().to_vec()
        } else {
            options.agents.clone()
        };

        let result = InstallResult {
            skill: installed,
            agents_installed,
            agents_skipped: Vec::new(),
            agents_failed: Vec::new(),
            paths: Vec::new(),
            duration_ms: start.elapsed().as_millis() as u64,
            was_dry_run: false,
        };

        debug!("Installation completed in {}ms", result.duration_ms);
        Ok(result)
    }

    pub async fn install_with_progress(
        &self,
        options: InstallOptions,
        progress_tx: mpsc::Sender<InstallProgress>,
    ) -> RimuruResult<InstallResult> {
        options.validate()?;
        let start = std::time::Instant::now();

        let total_agents = if options.install_all {
            SkillKitAgent::all().len()
        } else {
            options.agents.len()
        };

        let _ = progress_tx
            .send(InstallProgress::Started {
                skill_name: options.skill_name.clone(),
                total_agents,
            })
            .await;

        let _ = progress_tx
            .send(InstallProgress::Downloading {
                skill_name: options.skill_name.clone(),
            })
            .await;

        let _ = progress_tx.send(InstallProgress::Validating).await;

        let result = self.install(options).await;

        match &result {
            Ok(r) => {
                let _ = progress_tx
                    .send(InstallProgress::Completed {
                        success_count: r.success_count(),
                        failure_count: r.failure_count(),
                        duration_ms: start.elapsed().as_millis() as u64,
                    })
                    .await;
            }
            Err(e) => {
                let _ = progress_tx
                    .send(InstallProgress::Error {
                        message: e.to_string(),
                    })
                    .await;
            }
        }

        result
    }

    pub async fn install_multiple(
        &self,
        skill_names: &[&str],
        agents: &[SkillKitAgent],
    ) -> Vec<RimuruResult<InstallResult>> {
        let mut results = Vec::new();

        for skill_name in skill_names {
            let options = InstallOptions::new(skill_name).for_agents(agents.to_vec());
            let result = self.install(options).await;
            results.push(result);
        }

        results
    }

    pub async fn quick_install(
        &self,
        skill_name: &str,
        agent: SkillKitAgent,
    ) -> RimuruResult<InstalledSkill> {
        let options = InstallOptions::new(skill_name).for_agent(agent);
        let result = self.install(options).await?;
        Ok(result.skill)
    }

    pub async fn uninstall(&self, skill_name: &str) -> RimuruResult<()> {
        info!("Uninstalling skill: {}", skill_name);
        self.bridge.uninstall(skill_name).await
    }

    pub async fn reinstall(&self, options: InstallOptions) -> RimuruResult<InstallResult> {
        info!("Reinstalling skill: {}", options.skill_name);
        self.uninstall(&options.skill_name).await.ok();
        self.install(options.force()).await
    }

    pub async fn is_installed(&self, skill_name: &str) -> RimuruResult<bool> {
        let installed = self.bridge.list_installed().await?;
        Ok(installed.iter().any(|s| {
            s.skill.name.eq_ignore_ascii_case(skill_name)
                || s.skill.slug.eq_ignore_ascii_case(skill_name)
        }))
    }

    pub async fn get_installed_skill(
        &self,
        skill_name: &str,
    ) -> RimuruResult<Option<InstalledSkill>> {
        let installed = self.bridge.list_installed().await?;
        Ok(installed.into_iter().find(|s| {
            s.skill.name.eq_ignore_ascii_case(skill_name)
                || s.skill.slug.eq_ignore_ascii_case(skill_name)
        }))
    }

    async fn dry_run_install(&self, options: &InstallOptions) -> RimuruResult<InstallResult> {
        info!(
            "Dry run: would install {} for {:?}",
            options.skill_name, options.agents
        );

        let skill_details = self.bridge.get_skill_details(&options.skill_name).await?;

        let installed = InstalledSkill::new(skill_details, "[dry-run]", options.agents.clone());

        Ok(InstallResult {
            skill: installed,
            agents_installed: Vec::new(),
            agents_skipped: options.agents.clone(),
            agents_failed: Vec::new(),
            paths: Vec::new(),
            duration_ms: 0,
            was_dry_run: true,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skillkit::Skill;

    #[test]
    fn test_install_options_builder() {
        let options = InstallOptions::new("test-skill")
            .for_agent(SkillKitAgent::ClaudeCode)
            .force();

        assert_eq!(options.skill_name, "test-skill");
        assert_eq!(options.agents.len(), 1);
        assert!(options.force);
        assert!(!options.install_all);
    }

    #[test]
    fn test_install_options_for_all() {
        let options = InstallOptions::new("test-skill").for_all_agents();

        assert!(options.install_all);
        assert_eq!(options.agents.len(), 32);
    }

    #[test]
    fn test_install_options_validation() {
        let empty_name = InstallOptions::new("");
        assert!(empty_name.validate().is_err());

        let no_agents = InstallOptions::new("test-skill");
        assert!(no_agents.validate().is_err());

        let valid = InstallOptions::new("test-skill").for_agent(SkillKitAgent::ClaudeCode);
        assert!(valid.validate().is_ok());

        let valid_all = InstallOptions::new("test-skill").for_all_agents();
        assert!(valid_all.validate().is_ok());
    }

    #[test]
    fn test_install_result_counts() {
        let skill = Skill::new("Test", "Test skill");
        let installed = InstalledSkill::new(skill, "/path", vec![SkillKitAgent::ClaudeCode]);

        let result = InstallResult {
            skill: installed,
            agents_installed: vec![SkillKitAgent::ClaudeCode, SkillKitAgent::Cursor],
            agents_skipped: vec![],
            agents_failed: vec![(SkillKitAgent::Codex, "Error".to_string())],
            paths: vec![],
            duration_ms: 100,
            was_dry_run: false,
        };

        assert_eq!(result.success_count(), 2);
        assert_eq!(result.failure_count(), 1);
        assert!(result.is_partial_success());
        assert!(!result.is_complete_success());
    }
}
