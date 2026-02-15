use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use super::types::{InstalledSkill, SkillKitAgent, SkillKitInfo, SkillKitInstallationStatus};
use crate::error::{RimuruError, RimuruResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillKitConfig {
    pub enabled: bool,
    pub auto_sync: bool,
    pub sync_interval_secs: u64,
    pub marketplace_url: String,
    pub cache_ttl_secs: u64,
    pub default_agents: Vec<SkillKitAgent>,
}

impl Default for SkillKitConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_sync: true,
            sync_interval_secs: 3600,
            marketplace_url: "https://agenstskills.com".to_string(),
            cache_ttl_secs: 86400,
            default_agents: vec![SkillKitAgent::ClaudeCode],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SkillKitPreferences {
    #[serde(default)]
    pub last_selected_agents: Vec<SkillKitAgent>,
    #[serde(default)]
    pub favorite_skills: Vec<String>,
    #[serde(default)]
    pub hidden_skills: Vec<String>,
    #[serde(default)]
    pub symlink_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledSkillsRegistry {
    pub skills: HashMap<String, InstalledSkill>,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl Default for InstalledSkillsRegistry {
    fn default() -> Self {
        Self {
            skills: HashMap::new(),
            last_updated: chrono::Utc::now(),
        }
    }
}

impl InstalledSkillsRegistry {
    pub fn add(&mut self, skill: InstalledSkill) {
        self.skills.insert(skill.skill.slug.clone(), skill);
        self.last_updated = chrono::Utc::now();
    }

    pub fn remove(&mut self, slug: &str) -> Option<InstalledSkill> {
        let removed = self.skills.remove(slug);
        if removed.is_some() {
            self.last_updated = chrono::Utc::now();
        }
        removed
    }

    pub fn get(&self, slug: &str) -> Option<&InstalledSkill> {
        self.skills.get(slug)
    }

    pub fn list(&self) -> Vec<&InstalledSkill> {
        self.skills.values().collect()
    }

    pub fn list_for_agent(&self, agent: &SkillKitAgent) -> Vec<&InstalledSkill> {
        self.skills
            .values()
            .filter(|s| s.installed_for.contains(agent))
            .collect()
    }

    pub fn count(&self) -> usize {
        self.skills.len()
    }

    pub fn is_installed(&self, slug: &str) -> bool {
        self.skills.contains_key(slug)
    }
}

pub struct SkillKitConfigManager {
    skillkit_dir: PathBuf,
    config: SkillKitConfig,
    preferences: SkillKitPreferences,
    registry: InstalledSkillsRegistry,
}

impl SkillKitConfigManager {
    pub fn new() -> RimuruResult<Self> {
        let skillkit_dir = Self::get_skillkit_dir()?;
        let config = Self::load_config(&skillkit_dir)?;
        let preferences = Self::load_preferences(&skillkit_dir)?;
        let registry = Self::load_registry(&skillkit_dir)?;

        Ok(Self {
            skillkit_dir,
            config,
            preferences,
            registry,
        })
    }

    fn get_skillkit_dir() -> RimuruResult<PathBuf> {
        let home = dirs::home_dir()
            .ok_or_else(|| RimuruError::Config("Could not determine home directory".to_string()))?;
        Ok(home.join(".skillkit"))
    }

    pub fn get_skills_dir() -> RimuruResult<PathBuf> {
        let skillkit_dir = Self::get_skillkit_dir()?;
        Ok(skillkit_dir.join("skills"))
    }

    fn load_config(skillkit_dir: &PathBuf) -> RimuruResult<SkillKitConfig> {
        let config_path = skillkit_dir.join("config.json");
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path).map_err(|e| {
                RimuruError::Config(format!("Failed to read SkillKit config: {}", e))
            })?;
            serde_json::from_str(&content)
                .map_err(|e| RimuruError::Config(format!("Failed to parse SkillKit config: {}", e)))
        } else {
            Ok(SkillKitConfig::default())
        }
    }

    fn load_preferences(skillkit_dir: &PathBuf) -> RimuruResult<SkillKitPreferences> {
        let prefs_path = skillkit_dir.join("preferences.json");
        if prefs_path.exists() {
            let content = std::fs::read_to_string(&prefs_path).map_err(|e| {
                RimuruError::Config(format!("Failed to read SkillKit preferences: {}", e))
            })?;
            serde_json::from_str(&content).map_err(|e| {
                RimuruError::Config(format!("Failed to parse SkillKit preferences: {}", e))
            })
        } else {
            Ok(SkillKitPreferences::default())
        }
    }

    fn load_registry(skillkit_dir: &PathBuf) -> RimuruResult<InstalledSkillsRegistry> {
        let registry_path = skillkit_dir.join("installed.json");
        if registry_path.exists() {
            let content = std::fs::read_to_string(&registry_path).map_err(|e| {
                RimuruError::Config(format!("Failed to read SkillKit registry: {}", e))
            })?;
            serde_json::from_str(&content).map_err(|e| {
                RimuruError::Config(format!("Failed to parse SkillKit registry: {}", e))
            })
        } else {
            Ok(InstalledSkillsRegistry::default())
        }
    }

    pub fn save_config(&self) -> RimuruResult<()> {
        let config_path = self.skillkit_dir.join("config.json");
        std::fs::create_dir_all(&self.skillkit_dir).map_err(|e| {
            RimuruError::Config(format!("Failed to create SkillKit directory: {}", e))
        })?;
        let content = serde_json::to_string_pretty(&self.config).map_err(|e| {
            RimuruError::Config(format!("Failed to serialize SkillKit config: {}", e))
        })?;
        std::fs::write(&config_path, content)
            .map_err(|e| RimuruError::Config(format!("Failed to write SkillKit config: {}", e)))?;
        Ok(())
    }

    pub fn save_preferences(&self) -> RimuruResult<()> {
        let prefs_path = self.skillkit_dir.join("preferences.json");
        std::fs::create_dir_all(&self.skillkit_dir).map_err(|e| {
            RimuruError::Config(format!("Failed to create SkillKit directory: {}", e))
        })?;
        let content = serde_json::to_string_pretty(&self.preferences).map_err(|e| {
            RimuruError::Config(format!("Failed to serialize SkillKit preferences: {}", e))
        })?;
        std::fs::write(&prefs_path, content).map_err(|e| {
            RimuruError::Config(format!("Failed to write SkillKit preferences: {}", e))
        })?;
        Ok(())
    }

    pub fn save_registry(&self) -> RimuruResult<()> {
        let registry_path = self.skillkit_dir.join("installed.json");
        std::fs::create_dir_all(&self.skillkit_dir).map_err(|e| {
            RimuruError::Config(format!("Failed to create SkillKit directory: {}", e))
        })?;
        let content = serde_json::to_string_pretty(&self.registry).map_err(|e| {
            RimuruError::Config(format!("Failed to serialize SkillKit registry: {}", e))
        })?;
        std::fs::write(&registry_path, content).map_err(|e| {
            RimuruError::Config(format!("Failed to write SkillKit registry: {}", e))
        })?;
        Ok(())
    }

    pub fn config(&self) -> &SkillKitConfig {
        &self.config
    }

    pub fn config_mut(&mut self) -> &mut SkillKitConfig {
        &mut self.config
    }

    pub fn preferences(&self) -> &SkillKitPreferences {
        &self.preferences
    }

    pub fn preferences_mut(&mut self) -> &mut SkillKitPreferences {
        &mut self.preferences
    }

    pub fn registry(&self) -> &InstalledSkillsRegistry {
        &self.registry
    }

    pub fn registry_mut(&mut self) -> &mut InstalledSkillsRegistry {
        &mut self.registry
    }

    pub fn skillkit_dir(&self) -> &PathBuf {
        &self.skillkit_dir
    }

    pub fn enable_skill(&mut self, slug: &str) -> RimuruResult<()> {
        if let Some(skill) = self.registry.skills.get_mut(slug) {
            skill.enabled = true;
            self.save_registry()?;
        }
        Ok(())
    }

    pub fn disable_skill(&mut self, slug: &str) -> RimuruResult<()> {
        if let Some(skill) = self.registry.skills.get_mut(slug) {
            skill.enabled = false;
            self.save_registry()?;
        }
        Ok(())
    }
}

pub fn detect_skillkit_installation() -> RimuruResult<SkillKitInfo> {
    let output = std::process::Command::new("skillkit")
        .arg("--version")
        .output();

    match output {
        Ok(output) if output.status.success() => {
            let version_str = String::from_utf8_lossy(&output.stdout);
            let version = version_str.split_whitespace().last().map(String::from);

            let which_output = std::process::Command::new("which").arg("skillkit").output();

            let path = which_output.ok().and_then(|o| {
                if o.status.success() {
                    Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
                } else {
                    None
                }
            });

            let home = dirs::home_dir();
            let config_path = home.map(|h| h.join(".skillkit").to_string_lossy().to_string());

            Ok(SkillKitInfo {
                installed: true,
                version,
                status: SkillKitInstallationStatus::Installed,
                path,
                config_path,
            })
        }
        _ => Ok(SkillKitInfo {
            installed: false,
            version: None,
            status: SkillKitInstallationStatus::NotInstalled,
            path: None,
            config_path: None,
        }),
    }
}

pub fn detect_npx_skillkit() -> RimuruResult<bool> {
    let output = std::process::Command::new("npx")
        .arg("skillkit")
        .arg("--version")
        .output();

    Ok(output.map(|o| o.status.success()).unwrap_or(false))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skillkit_config_default() {
        let config = SkillKitConfig::default();
        assert!(config.enabled);
        assert!(config.auto_sync);
        assert_eq!(config.sync_interval_secs, 3600);
        assert!(!config.default_agents.is_empty());
    }

    #[test]
    fn test_installed_skills_registry() {
        let mut registry = InstalledSkillsRegistry::default();
        assert_eq!(registry.count(), 0);

        let skill = crate::skillkit::types::Skill::new("Test Skill", "A test");
        let installed =
            InstalledSkill::new(skill, "/path/to/skill", vec![SkillKitAgent::ClaudeCode]);

        registry.add(installed);
        assert_eq!(registry.count(), 1);
        assert!(registry.is_installed("test-skill"));

        let retrieved = registry.get("test-skill");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().skill.name, "Test Skill");

        let for_claude = registry.list_for_agent(&SkillKitAgent::ClaudeCode);
        assert_eq!(for_claude.len(), 1);

        let for_cursor = registry.list_for_agent(&SkillKitAgent::Cursor);
        assert_eq!(for_cursor.len(), 0);

        registry.remove("test-skill");
        assert_eq!(registry.count(), 0);
        assert!(!registry.is_installed("test-skill"));
    }

    #[test]
    fn test_skillkit_preferences_default() {
        let prefs = SkillKitPreferences::default();
        assert!(prefs.last_selected_agents.is_empty());
        assert!(prefs.favorite_skills.is_empty());
        assert!(!prefs.symlink_enabled);
    }
}
