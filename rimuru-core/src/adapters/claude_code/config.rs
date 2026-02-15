use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeCodeConfig {
    pub config_dir: PathBuf,
    pub api_key_path: Option<PathBuf>,
    pub projects_dir: PathBuf,
    pub auto_detect: bool,
}

impl Default for ClaudeCodeConfig {
    fn default() -> Self {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));

        Self {
            config_dir: home_dir.join(".claude"),
            api_key_path: None,
            projects_dir: home_dir.join(".claude").join("projects"),
            auto_detect: true,
        }
    }
}

impl ClaudeCodeConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config_dir(mut self, path: PathBuf) -> Self {
        self.config_dir = path;
        self
    }

    pub fn with_api_key_path(mut self, path: PathBuf) -> Self {
        self.api_key_path = Some(path);
        self
    }

    pub fn with_projects_dir(mut self, path: PathBuf) -> Self {
        self.projects_dir = path;
        self
    }

    pub fn settings_path(&self) -> PathBuf {
        self.config_dir.join("settings.json")
    }

    pub fn settings_local_path(&self) -> PathBuf {
        self.config_dir.join("settings.local.json")
    }

    pub fn api_key_file(&self) -> PathBuf {
        self.api_key_path
            .clone()
            .unwrap_or_else(|| self.config_dir.join(".credentials.json"))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeCodeSettings {
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub theme: Option<String>,
    #[serde(default, rename = "customApiKeyResponsibility")]
    pub custom_api_key_responsibility: Option<String>,
    #[serde(default, rename = "hasCompletedOnboarding")]
    pub has_completed_onboarding: Option<bool>,
    #[serde(default, rename = "preferredNotifChannel")]
    pub preferred_notif_channel: Option<String>,
    #[serde(default)]
    pub permissions: Option<ClaudeCodePermissions>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClaudeCodePermissions {
    #[serde(default)]
    pub allow_read: bool,
    #[serde(default)]
    pub allow_write: bool,
    #[serde(default)]
    pub allow_bash: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ClaudeCodeConfig::default();
        assert!(config.config_dir.ends_with(".claude"));
        assert!(config.api_key_path.is_none());
        assert!(config.auto_detect);
    }

    #[test]
    fn test_builder_pattern() {
        let config = ClaudeCodeConfig::new()
            .with_config_dir(PathBuf::from("/custom/path"))
            .with_api_key_path(PathBuf::from("/custom/api_key.json"));

        assert_eq!(config.config_dir, PathBuf::from("/custom/path"));
        assert_eq!(
            config.api_key_path,
            Some(PathBuf::from("/custom/api_key.json"))
        );
    }

    #[test]
    fn test_settings_paths() {
        let config = ClaudeCodeConfig::new().with_config_dir(PathBuf::from("/test/.claude"));

        assert_eq!(
            config.settings_path(),
            PathBuf::from("/test/.claude/settings.json")
        );
        assert_eq!(
            config.settings_local_path(),
            PathBuf::from("/test/.claude/settings.local.json")
        );
    }
}
