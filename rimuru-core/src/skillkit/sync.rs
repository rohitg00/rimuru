use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{debug, info, warn};

use super::bridge::SkillKitBridge;
use super::config::InstalledSkillsRegistry;
use super::types::SkillKitAgent;
use crate::error::{RimuruError, RimuruResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub synced_at: DateTime<Utc>,
    pub skills_added: Vec<String>,
    pub skills_removed: Vec<String>,
    pub skills_updated: Vec<String>,
    pub agents_detected: Vec<SkillKitAgent>,
    pub errors: Vec<SyncError>,
    pub duration_ms: u64,
}

impl SyncResult {
    pub fn empty() -> Self {
        Self {
            synced_at: Utc::now(),
            skills_added: Vec::new(),
            skills_removed: Vec::new(),
            skills_updated: Vec::new(),
            agents_detected: Vec::new(),
            errors: Vec::new(),
            duration_ms: 0,
        }
    }

    pub fn has_changes(&self) -> bool {
        !self.skills_added.is_empty()
            || !self.skills_removed.is_empty()
            || !self.skills_updated.is_empty()
    }

    pub fn is_success(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn total_changes(&self) -> usize {
        self.skills_added.len() + self.skills_removed.len() + self.skills_updated.len()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncError {
    pub skill_name: Option<String>,
    pub agent: Option<SkillKitAgent>,
    pub message: String,
    pub recoverable: bool,
}

impl SyncError {
    pub fn new(message: &str) -> Self {
        Self {
            skill_name: None,
            agent: None,
            message: message.to_string(),
            recoverable: false,
        }
    }

    pub fn for_skill(skill_name: &str, message: &str) -> Self {
        Self {
            skill_name: Some(skill_name.to_string()),
            agent: None,
            message: message.to_string(),
            recoverable: true,
        }
    }

    pub fn for_agent(agent: SkillKitAgent, message: &str) -> Self {
        Self {
            skill_name: None,
            agent: Some(agent),
            message: message.to_string(),
            recoverable: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncOptions {
    pub check_updates: bool,
    pub detect_agents: bool,
    pub auto_remove_orphans: bool,
    pub force_refresh: bool,
    pub agents_to_sync: Option<Vec<SkillKitAgent>>,
}

impl Default for SyncOptions {
    fn default() -> Self {
        Self {
            check_updates: true,
            detect_agents: true,
            auto_remove_orphans: false,
            force_refresh: false,
            agents_to_sync: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillVersion {
    pub version: String,
    pub released_at: Option<DateTime<Utc>>,
    pub changelog: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillUpdate {
    pub skill_name: String,
    pub current_version: String,
    pub latest_version: String,
    pub update_available: bool,
    pub breaking_changes: bool,
    pub changelog: Option<String>,
}

impl SkillUpdate {
    pub fn new(skill_name: &str, current: &str, latest: &str) -> Self {
        Self {
            skill_name: skill_name.to_string(),
            current_version: current.to_string(),
            latest_version: latest.to_string(),
            update_available: current != latest,
            breaking_changes: false,
            changelog: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityChange {
    pub skill_name: String,
    pub agent: SkillKitAgent,
    pub change_type: CompatibilityChangeType,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompatibilityChangeType {
    Added,
    Removed,
    Deprecated,
    Breaking,
}

impl std::fmt::Display for CompatibilityChangeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Added => write!(f, "Added"),
            Self::Removed => write!(f, "Removed"),
            Self::Deprecated => write!(f, "Deprecated"),
            Self::Breaking => write!(f, "Breaking"),
        }
    }
}

pub struct SkillSyncer {
    bridge: SkillKitBridge,
}

impl SkillSyncer {
    pub fn new(bridge: SkillKitBridge) -> Self {
        Self { bridge }
    }

    pub fn bridge(&self) -> &SkillKitBridge {
        &self.bridge
    }

    pub async fn sync(&self, options: SyncOptions) -> RimuruResult<SyncResult> {
        let start = std::time::Instant::now();
        let mut result = SyncResult::empty();

        info!("Starting skill sync with options: {:?}", options);

        if options.detect_agents {
            match self.detect_available_agents().await {
                Ok(agents) => {
                    result.agents_detected = agents;
                    info!("Detected {} available agents", result.agents_detected.len());
                }
                Err(e) => {
                    warn!("Failed to detect agents: {}", e);
                    result
                        .errors
                        .push(SyncError::new(&format!("Agent detection failed: {}", e)));
                }
            }
        }

        let agents_to_sync = options
            .agents_to_sync
            .clone()
            .unwrap_or_else(|| result.agents_detected.clone());

        for agent in &agents_to_sync {
            match self.sync_agent_skills(agent, &options).await {
                Ok((added, removed, updated)) => {
                    result.skills_added.extend(added);
                    result.skills_removed.extend(removed);
                    result.skills_updated.extend(updated);
                }
                Err(e) => {
                    warn!("Failed to sync skills for agent {}: {}", agent, e);
                    result
                        .errors
                        .push(SyncError::for_agent(*agent, &e.to_string()));
                }
            }
        }

        if options.check_updates {
            if let Err(e) = self.check_marketplace_updates().await {
                warn!("Failed to check marketplace updates: {}", e);
                result
                    .errors
                    .push(SyncError::new(&format!("Update check failed: {}", e)));
            }
        }

        result.duration_ms = start.elapsed().as_millis() as u64;
        result.synced_at = Utc::now();

        info!(
            "Sync completed in {}ms: {} added, {} removed, {} updated, {} errors",
            result.duration_ms,
            result.skills_added.len(),
            result.skills_removed.len(),
            result.skills_updated.len(),
            result.errors.len()
        );

        Ok(result)
    }

    pub async fn detect_available_agents(&self) -> RimuruResult<Vec<SkillKitAgent>> {
        let mut available = Vec::new();

        for agent in SkillKitAgent::all() {
            if self.is_agent_available(agent).await {
                available.push(*agent);
            }
        }

        Ok(available)
    }

    async fn is_agent_available(&self, agent: &SkillKitAgent) -> bool {
        let agent_dir = match self.get_agent_skills_dir(agent) {
            Ok(dir) => dir,
            Err(_) => return false,
        };
        agent_dir.exists()
    }

    fn get_agent_skills_dir(&self, agent: &SkillKitAgent) -> RimuruResult<PathBuf> {
        let home = dirs::home_dir()
            .ok_or_else(|| RimuruError::Config("Could not determine home directory".to_string()))?;

        let skills_dir = match agent {
            SkillKitAgent::ClaudeCode => home.join(".claude").join("skills"),
            SkillKitAgent::Cursor => home.join(".cursor").join("skills"),
            SkillKitAgent::Codex => home.join(".codex").join("skills"),
            SkillKitAgent::GeminiCli => home.join(".gemini").join("skills"),
            SkillKitAgent::OpenCode => home.join(".opencode").join("skills"),
            SkillKitAgent::Windsurf => home.join(".windsurf").join("skills"),
            SkillKitAgent::GithubCopilot => home.join(".copilot").join("skills"),
            SkillKitAgent::Cline => home.join(".cline").join("skills"),
            SkillKitAgent::Continue => home.join(".continue").join("skills"),
            SkillKitAgent::Roo => home.join(".roo").join("skills"),
            SkillKitAgent::Goose => home.join(".goose").join("skills"),
            SkillKitAgent::Amp => home.join(".amp").join("skills"),
            _ => home.join(".skillkit").join("agents").join(agent.as_str()),
        };

        Ok(skills_dir)
    }

    async fn sync_agent_skills(
        &self,
        agent: &SkillKitAgent,
        options: &SyncOptions,
    ) -> RimuruResult<(Vec<String>, Vec<String>, Vec<String>)> {
        let mut added = Vec::new();
        let mut removed = Vec::new();
        let updated = Vec::new();

        let skills_dir = self.get_agent_skills_dir(agent)?;

        if !skills_dir.exists() {
            debug!("Skills directory does not exist for agent: {}", agent);
            return Ok((added, removed, updated));
        }

        let discovered = self.discover_skills_in_directory(&skills_dir)?;
        let registry = self.bridge.config_manager().registry();

        for skill_name in &discovered {
            if !registry.is_installed(skill_name) {
                added.push(skill_name.clone());
                debug!("Discovered new skill: {} for agent {}", skill_name, agent);
            }
        }

        if options.auto_remove_orphans {
            for installed in registry.list_for_agent(agent) {
                if !discovered.contains(&installed.skill.slug) {
                    removed.push(installed.skill.slug.clone());
                    debug!(
                        "Skill no longer present: {} for agent {}",
                        installed.skill.slug, agent
                    );
                }
            }
        }

        Ok((added, removed, updated))
    }

    fn discover_skills_in_directory(&self, dir: &PathBuf) -> RimuruResult<Vec<String>> {
        let mut skills = Vec::new();

        if !dir.exists() {
            return Ok(skills);
        }

        let entries = std::fs::read_dir(dir)
            .map_err(|e| RimuruError::IoError(format!("Failed to read skills directory: {}", e)))?;

        for entry in entries {
            let entry = entry.map_err(|e| RimuruError::IoError(e.to_string()))?;
            let path = entry.path();

            if path.is_dir() {
                let skill_md = path.join("SKILL.md");
                let skill_json = path.join("skill.json");

                if skill_md.exists() || skill_json.exists() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        skills.push(name.to_string());
                    }
                }
            } else if path.extension().map(|e| e == "md").unwrap_or(false) {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    skills.push(stem.to_string());
                }
            }
        }

        Ok(skills)
    }

    async fn check_marketplace_updates(&self) -> RimuruResult<Vec<SkillUpdate>> {
        let updates = Vec::new();

        if let Err(e) = self.bridge.sync().await {
            warn!("Marketplace sync failed: {}", e);
        }

        Ok(updates)
    }

    pub async fn check_skill_updates(&self, skill_name: &str) -> RimuruResult<Option<SkillUpdate>> {
        let registry = self.bridge.config_manager().registry();

        let installed = registry.get(skill_name).ok_or_else(|| {
            RimuruError::SkillNotFound(format!("Skill '{}' is not installed", skill_name))
        })?;

        let current_version = installed
            .skill
            .version
            .clone()
            .unwrap_or_else(|| "0.0.0".to_string());

        match self.bridge.get_skill_details(skill_name).await {
            Ok(marketplace_skill) => {
                let latest_version = marketplace_skill
                    .version
                    .clone()
                    .unwrap_or_else(|| "0.0.0".to_string());

                if current_version != latest_version {
                    Ok(Some(SkillUpdate::new(
                        skill_name,
                        &current_version,
                        &latest_version,
                    )))
                } else {
                    Ok(None)
                }
            }
            Err(e) => {
                debug!("Could not check updates for {}: {}", skill_name, e);
                Ok(None)
            }
        }
    }

    pub async fn get_compatibility_changes(
        &self,
        skill_name: &str,
    ) -> RimuruResult<Vec<CompatibilityChange>> {
        let changes = Vec::new();

        let registry = self.bridge.config_manager().registry();
        let _installed = registry.get(skill_name).ok_or_else(|| {
            RimuruError::SkillNotFound(format!("Skill '{}' is not installed", skill_name))
        })?;

        Ok(changes)
    }

    pub async fn get_all_updates(&self) -> RimuruResult<Vec<SkillUpdate>> {
        let mut updates = Vec::new();
        let registry = self.bridge.config_manager().registry();

        for installed in registry.list() {
            if let Ok(Some(update)) = self.check_skill_updates(&installed.skill.slug).await {
                updates.push(update);
            }
        }

        Ok(updates)
    }

    pub fn get_sync_status(&self) -> SyncStatus {
        let registry = self.bridge.config_manager().registry();
        let config = self.bridge.config_manager().config();

        SyncStatus {
            last_synced: Some(registry.last_updated),
            auto_sync_enabled: config.auto_sync,
            sync_interval_secs: config.sync_interval_secs,
            total_installed: registry.count(),
            needs_sync: self.needs_sync(registry),
        }
    }

    fn needs_sync(&self, registry: &InstalledSkillsRegistry) -> bool {
        let config = self.bridge.config_manager().config();
        let elapsed = Utc::now()
            .signed_duration_since(registry.last_updated)
            .num_seconds() as u64;

        elapsed > config.sync_interval_secs
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatus {
    pub last_synced: Option<DateTime<Utc>>,
    pub auto_sync_enabled: bool,
    pub sync_interval_secs: u64,
    pub total_installed: usize,
    pub needs_sync: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncProgress {
    pub current_agent: Option<SkillKitAgent>,
    pub agents_processed: usize,
    pub total_agents: usize,
    pub skills_discovered: usize,
    pub phase: SyncPhase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncPhase {
    DetectingAgents,
    ScanningSkills,
    CheckingUpdates,
    ApplyingChanges,
    Completed,
    Failed,
}

impl std::fmt::Display for SyncPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DetectingAgents => write!(f, "Detecting agents"),
            Self::ScanningSkills => write!(f, "Scanning skills"),
            Self::CheckingUpdates => write!(f, "Checking updates"),
            Self::ApplyingChanges => write!(f, "Applying changes"),
            Self::Completed => write!(f, "Completed"),
            Self::Failed => write!(f, "Failed"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_result_empty() {
        let result = SyncResult::empty();
        assert!(!result.has_changes());
        assert!(result.is_success());
        assert_eq!(result.total_changes(), 0);
    }

    #[test]
    fn test_sync_result_with_changes() {
        let mut result = SyncResult::empty();
        result.skills_added.push("skill-1".to_string());
        result.skills_updated.push("skill-2".to_string());

        assert!(result.has_changes());
        assert!(result.is_success());
        assert_eq!(result.total_changes(), 2);
    }

    #[test]
    fn test_sync_result_with_errors() {
        let mut result = SyncResult::empty();
        result.errors.push(SyncError::new("test error"));

        assert!(!result.has_changes());
        assert!(!result.is_success());
    }

    #[test]
    fn test_sync_error_creation() {
        let error = SyncError::new("generic error");
        assert!(error.skill_name.is_none());
        assert!(error.agent.is_none());
        assert!(!error.recoverable);

        let skill_error = SyncError::for_skill("test-skill", "skill error");
        assert_eq!(skill_error.skill_name, Some("test-skill".to_string()));
        assert!(skill_error.recoverable);

        let agent_error = SyncError::for_agent(SkillKitAgent::ClaudeCode, "agent error");
        assert_eq!(agent_error.agent, Some(SkillKitAgent::ClaudeCode));
        assert!(agent_error.recoverable);
    }

    #[test]
    fn test_sync_options_default() {
        let options = SyncOptions::default();
        assert!(options.check_updates);
        assert!(options.detect_agents);
        assert!(!options.auto_remove_orphans);
        assert!(!options.force_refresh);
        assert!(options.agents_to_sync.is_none());
    }

    #[test]
    fn test_skill_update() {
        let update = SkillUpdate::new("test-skill", "1.0.0", "1.1.0");
        assert!(update.update_available);
        assert!(!update.breaking_changes);

        let no_update = SkillUpdate::new("test-skill", "1.0.0", "1.0.0");
        assert!(!no_update.update_available);
    }

    #[test]
    fn test_compatibility_change_type_display() {
        assert_eq!(CompatibilityChangeType::Added.to_string(), "Added");
        assert_eq!(CompatibilityChangeType::Removed.to_string(), "Removed");
        assert_eq!(
            CompatibilityChangeType::Deprecated.to_string(),
            "Deprecated"
        );
        assert_eq!(CompatibilityChangeType::Breaking.to_string(), "Breaking");
    }

    #[test]
    fn test_sync_phase_display() {
        assert_eq!(SyncPhase::DetectingAgents.to_string(), "Detecting agents");
        assert_eq!(SyncPhase::ScanningSkills.to_string(), "Scanning skills");
        assert_eq!(SyncPhase::CheckingUpdates.to_string(), "Checking updates");
        assert_eq!(SyncPhase::ApplyingChanges.to_string(), "Applying changes");
        assert_eq!(SyncPhase::Completed.to_string(), "Completed");
        assert_eq!(SyncPhase::Failed.to_string(), "Failed");
    }

    #[test]
    fn test_sync_status_default_values() {
        let status = SyncStatus {
            last_synced: None,
            auto_sync_enabled: true,
            sync_interval_secs: 3600,
            total_installed: 0,
            needs_sync: true,
        };

        assert!(status.last_synced.is_none());
        assert!(status.auto_sync_enabled);
        assert!(status.needs_sync);
    }
}
