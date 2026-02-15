use std::process::Stdio;
use tokio::process::Command;
use tracing::{debug, error, info, warn};

use super::config::{detect_npx_skillkit, detect_skillkit_installation, SkillKitConfigManager};
use super::types::{
    cli_response, InstalledSkill, MarketplaceStats, PublishResult, SearchFilters, Skill,
    SkillKitAgent, SkillKitInfo, SkillRecommendation, SkillSearchResult, TranslationResult,
};
use crate::error::{RimuruError, RimuruResult};

/// Strip ANSI escape codes from output
fn strip_ansi_codes(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // Skip until we find a letter (end of escape sequence)
            while let Some(&next) = chars.peek() {
                chars.next();
                if next.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            result.push(c);
        }
    }

    result
}

pub struct SkillKitBridge {
    config_manager: SkillKitConfigManager,
    skillkit_info: SkillKitInfo,
    use_npx: bool,
}

impl SkillKitBridge {
    pub async fn new() -> RimuruResult<Self> {
        let config_manager = SkillKitConfigManager::new()?;
        let skillkit_info = detect_skillkit_installation()?;
        let use_npx = if !skillkit_info.installed {
            detect_npx_skillkit()?
        } else {
            false
        };

        if !skillkit_info.installed && !use_npx {
            warn!("SkillKit is not installed. Install with: npm i -g skillkit");
        }

        Ok(Self {
            config_manager,
            skillkit_info,
            use_npx,
        })
    }

    pub fn is_available(&self) -> bool {
        self.skillkit_info.installed || self.use_npx
    }

    pub fn skillkit_info(&self) -> &SkillKitInfo {
        &self.skillkit_info
    }

    pub fn config_manager(&self) -> &SkillKitConfigManager {
        &self.config_manager
    }

    pub fn config_manager_mut(&mut self) -> &mut SkillKitConfigManager {
        &mut self.config_manager
    }

    async fn run_skillkit_command(&self, args: &[&str]) -> RimuruResult<String> {
        if !self.is_available() {
            return Err(RimuruError::SkillKit(
                "SkillKit is not installed. Install with: npm i -g skillkit".to_string(),
            ));
        }

        let (cmd, full_args) = if self.use_npx {
            let mut npx_args = vec!["skillkit"];
            npx_args.extend(args);
            ("npx", npx_args)
        } else {
            ("skillkit", args.to_vec())
        };

        debug!("Running SkillKit command: {} {:?}", cmd, full_args);

        let output = Command::new(cmd)
            .args(&full_args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .map_err(|e| RimuruError::SkillKit(format!("Failed to execute SkillKit: {}", e)))?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            Ok(stdout)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            error!("SkillKit command failed: {}", stderr);
            Err(RimuruError::SkillKit(format!(
                "SkillKit command failed: {}",
                stderr
            )))
        }
    }

    async fn run_skillkit_json<T: serde::de::DeserializeOwned>(
        &self,
        args: &[&str],
    ) -> RimuruResult<T> {
        let mut args_with_json = args.to_vec();
        args_with_json.push("--json");

        let raw_output = self.run_skillkit_command(&args_with_json).await?;

        // Strip ANSI escape codes (spinner chars, etc.)
        let output = strip_ansi_codes(&raw_output);

        // SkillKit CLI outputs spinner/progress characters before JSON
        // Find the first '[' or '{' to start JSON parsing
        // For objects starting with {, that's always the start
        // For arrays starting with [, only use it if it comes before the first {
        let brace_pos = output.find('{');
        let bracket_pos = output.find('[');

        let json_start = match (brace_pos, bracket_pos) {
            (Some(brace), Some(bracket)) => {
                // If bracket comes first AND is at a position before brace,
                // it's likely the start of an array response
                if bracket < brace {
                    bracket
                } else {
                    brace
                }
            }
            (Some(brace), None) => brace,
            (None, Some(bracket)) => bracket,
            (None, None) => 0,
        };

        let json_output = output[json_start..].trim();

        serde_json::from_str(json_output).map_err(|e| {
            debug!("Raw SkillKit output: {}", output);
            RimuruError::SkillKit(format!(
                "Failed to parse SkillKit output: {} (raw: {})",
                e,
                &json_output[..json_output.len().min(200)]
            ))
        })
    }

    pub async fn search(
        &self,
        query: &str,
        _filters: Option<SearchFilters>,
    ) -> RimuruResult<SkillSearchResult> {
        let args = vec!["marketplace", "search", query];

        info!("Searching SkillKit marketplace for: {}", query);
        let response: cli_response::MarketplaceSearchResponse =
            self.run_skillkit_json(&args).await?;
        Ok(response.to_search_result())
    }

    pub async fn install(
        &self,
        skill_name: &str,
        agents: &[SkillKitAgent],
    ) -> RimuruResult<InstalledSkill> {
        let mut args = vec!["install", skill_name, "--yes"];

        let agent_args: Vec<String> = agents
            .iter()
            .map(|a| format!("--agent={}", a.as_str()))
            .collect();

        for arg in &agent_args {
            args.push(arg);
        }

        info!("Installing skill '{}' for agents: {:?}", skill_name, agents);

        // Install doesn't have JSON output, construct result from success/failure
        let result = self.run_skillkit_command(&args).await;

        match result {
            Ok(_) => {
                let skill = Skill::new(skill_name, "Installed skill");
                Ok(InstalledSkill::new(
                    skill,
                    &format!("~/.skillkit/skills/{}", skill_name),
                    agents.to_vec(),
                ))
            }
            Err(e) => Err(e),
        }
    }

    pub async fn install_for_all(&self, skill_name: &str) -> RimuruResult<InstalledSkill> {
        let args = vec!["install", skill_name, "--all", "--yes"];
        info!("Installing skill '{}' for all agents", skill_name);

        let result = self.run_skillkit_command(&args).await;

        match result {
            Ok(_) => {
                let skill = Skill::new(skill_name, "Installed skill");
                Ok(InstalledSkill::new(
                    skill,
                    &format!("~/.skillkit/skills/{}", skill_name),
                    SkillKitAgent::all().to_vec(),
                ))
            }
            Err(e) => Err(e),
        }
    }

    pub async fn uninstall(&self, skill_name: &str) -> RimuruResult<()> {
        let args = vec!["uninstall", skill_name];
        info!("Uninstalling skill: {}", skill_name);
        self.run_skillkit_command(&args).await?;
        Ok(())
    }

    pub async fn translate(
        &self,
        skill_name: &str,
        from_agent: SkillKitAgent,
        to_agent: SkillKitAgent,
    ) -> RimuruResult<TranslationResult> {
        let from_str = from_agent.as_str();
        let to_str = to_agent.as_str();
        let args = vec!["translate", skill_name, "-f", from_str, "-t", to_str];

        info!(
            "Translating skill '{}' from {} to {}",
            skill_name, from_agent, to_agent
        );

        // Translation doesn't have JSON output, so run the command and construct result
        let result = self.run_skillkit_command(&args).await;

        match result {
            Ok(output) => Ok(TranslationResult::success(
                skill_name,
                from_agent,
                to_agent,
                output.trim(),
            )),
            Err(e) => Ok(TranslationResult::failure(
                skill_name,
                from_agent,
                to_agent,
                &e.to_string(),
            )),
        }
    }

    pub async fn list_installed(&self) -> RimuruResult<Vec<InstalledSkill>> {
        let args = vec!["list"];
        info!("Listing installed skills");
        let response: Vec<cli_response::InstalledSkillResponse> =
            self.run_skillkit_json(&args).await?;
        Ok(response.iter().map(|s| s.to_installed_skill()).collect())
    }

    pub async fn list_for_agent(&self, agent: SkillKitAgent) -> RimuruResult<Vec<InstalledSkill>> {
        let agent_str = agent.as_str();
        let args = vec!["list", "--agent", agent_str];
        info!("Listing skills for agent: {}", agent);
        let response: Vec<cli_response::InstalledSkillResponse> =
            self.run_skillkit_json(&args).await?;
        Ok(response.iter().map(|s| s.to_installed_skill()).collect())
    }

    pub async fn recommend(&self) -> RimuruResult<Vec<SkillRecommendation>> {
        let args = vec!["recommend"];
        info!("Getting skill recommendations");
        let response: cli_response::RecommendResponse = self.run_skillkit_json(&args).await?;
        Ok(response
            .recommendations
            .into_iter()
            .map(|r| {
                let skill = Skill::new(&r.name, &r.description);
                SkillRecommendation::new(
                    skill,
                    r.reason
                        .as_deref()
                        .unwrap_or("Recommended based on your project"),
                    r.score,
                )
            })
            .collect())
    }

    pub async fn recommend_for_workflow(
        &self,
        workflow_description: &str,
    ) -> RimuruResult<Vec<SkillRecommendation>> {
        let args = vec!["recommend", "--task", workflow_description];
        info!(
            "Getting recommendations for workflow: {}",
            workflow_description
        );
        let response: cli_response::RecommendResponse = self.run_skillkit_json(&args).await?;
        Ok(response
            .recommendations
            .into_iter()
            .map(|r| {
                let skill = Skill::new(&r.name, &r.description);
                SkillRecommendation::new(
                    skill,
                    r.reason
                        .as_deref()
                        .unwrap_or("Recommended based on your workflow"),
                    r.score,
                )
            })
            .collect())
    }

    pub async fn publish(&self, skill_path: &str) -> RimuruResult<PublishResult> {
        let args = vec!["publish", skill_path];
        info!("Publishing skill from: {}", skill_path);
        self.run_skillkit_json(&args).await
    }

    pub async fn get_skill_details(&self, skill_name: &str) -> RimuruResult<Skill> {
        let args = vec!["show", skill_name];
        info!("Getting details for skill: {}", skill_name);
        self.run_skillkit_json(&args).await
    }

    pub async fn get_marketplace_stats(&self) -> RimuruResult<MarketplaceStats> {
        let args = vec!["marketplace"];
        info!("Getting marketplace statistics");
        let response: cli_response::MarketplaceStatsResponse =
            self.run_skillkit_json(&args).await?;
        Ok(MarketplaceStats {
            total_skills: response.total_skills,
            total_downloads: 0,
            total_authors: response.sources,
            skills_by_agent: std::collections::HashMap::new(),
            trending_skills: Vec::new(),
            recent_skills: Vec::new(),
            last_updated: chrono::Utc::now(),
        })
    }

    pub async fn sync(&self) -> RimuruResult<()> {
        let args = vec!["sync"];
        info!("Syncing with SkillKit marketplace");
        self.run_skillkit_command(&args).await?;
        Ok(())
    }

    pub async fn context(&self, context_name: &str) -> RimuruResult<String> {
        let args = vec!["context", context_name];
        info!("Getting context: {}", context_name);
        self.run_skillkit_command(&args).await
    }

    pub async fn get_available_agents(&self) -> RimuruResult<Vec<SkillKitAgent>> {
        Ok(SkillKitAgent::all().to_vec())
    }

    pub async fn validate_skill(&self, skill_path: &str) -> RimuruResult<bool> {
        let args = vec!["validate", skill_path];
        info!("Validating skill at: {}", skill_path);
        let output = self.run_skillkit_command(&args).await;
        Ok(output.is_ok())
    }

    pub fn save_config(&self) -> RimuruResult<()> {
        self.config_manager.save_config()?;
        self.config_manager.save_preferences()?;
        self.config_manager.save_registry()?;
        Ok(())
    }
}

pub struct SkillKitBridgeBuilder {
    auto_detect: bool,
    prefer_npx: bool,
}

impl Default for SkillKitBridgeBuilder {
    fn default() -> Self {
        Self {
            auto_detect: true,
            prefer_npx: false,
        }
    }
}

impl SkillKitBridgeBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn auto_detect(mut self, enabled: bool) -> Self {
        self.auto_detect = enabled;
        self
    }

    pub fn prefer_npx(mut self, prefer: bool) -> Self {
        self.prefer_npx = prefer;
        self
    }

    pub async fn build(self) -> RimuruResult<SkillKitBridge> {
        let config_manager = SkillKitConfigManager::new()?;

        let (skillkit_info, use_npx) = if self.auto_detect {
            let info = detect_skillkit_installation()?;
            let npx = if !info.installed || self.prefer_npx {
                detect_npx_skillkit()?
            } else {
                false
            };
            (info, self.prefer_npx && npx)
        } else {
            (SkillKitInfo::default(), self.prefer_npx)
        };

        Ok(SkillKitBridge {
            config_manager,
            skillkit_info,
            use_npx,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skillkit_bridge_builder_default() {
        let builder = SkillKitBridgeBuilder::default();
        assert!(builder.auto_detect);
        assert!(!builder.prefer_npx);
    }

    #[test]
    fn test_skillkit_bridge_builder_chain() {
        let builder = SkillKitBridgeBuilder::new()
            .auto_detect(false)
            .prefer_npx(true);
        assert!(!builder.auto_detect);
        assert!(builder.prefer_npx);
    }
}
