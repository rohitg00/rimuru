use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexConfig {
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub api_key_env_var: String,
    pub organization_env_var: String,
    pub auto_detect: bool,
}

impl Default for CodexConfig {
    fn default() -> Self {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));

        Self {
            config_dir: home_dir.join(".codex"),
            data_dir: home_dir.join(".codex"),
            api_key_env_var: "OPENAI_API_KEY".to_string(),
            organization_env_var: "OPENAI_ORG_ID".to_string(),
            auto_detect: true,
        }
    }
}

impl CodexConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config_dir(mut self, path: PathBuf) -> Self {
        self.config_dir = path;
        self
    }

    pub fn with_data_dir(mut self, path: PathBuf) -> Self {
        self.data_dir = path;
        self
    }

    pub fn config_file_path(&self) -> PathBuf {
        self.config_dir.join("config.json")
    }

    pub fn instructions_file_path(&self) -> PathBuf {
        self.config_dir.join("instructions.md")
    }

    pub fn sessions_dir(&self) -> PathBuf {
        self.data_dir.join("sessions")
    }

    pub fn history_file(&self) -> PathBuf {
        self.data_dir.join("history.json")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CodexSettings {
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub approval_mode: Option<String>,
    #[serde(default)]
    pub sandbox_permissions: Option<CodexSandboxPermissions>,
    #[serde(default)]
    pub notify: Option<NotifySettings>,
    #[serde(default)]
    pub history: Option<HistorySettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CodexSandboxPermissions {
    #[serde(default)]
    pub read_file: bool,
    #[serde(default)]
    pub write_file: bool,
    #[serde(default)]
    pub network: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NotifySettings {
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HistorySettings {
    #[serde(default)]
    pub max_sessions: Option<i32>,
    #[serde(default)]
    pub persistence: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = CodexConfig::default();
        assert!(config.config_dir.ends_with(".codex"));
        assert!(config.auto_detect);
        assert_eq!(config.api_key_env_var, "OPENAI_API_KEY");
    }

    #[test]
    fn test_builder_pattern() {
        let config = CodexConfig::new()
            .with_config_dir(PathBuf::from("/custom/config"))
            .with_data_dir(PathBuf::from("/custom/data"));

        assert_eq!(config.config_dir, PathBuf::from("/custom/config"));
        assert_eq!(config.data_dir, PathBuf::from("/custom/data"));
    }

    #[test]
    fn test_path_helpers() {
        let config = CodexConfig::new().with_config_dir(PathBuf::from("/test/.codex"));

        assert_eq!(
            config.config_file_path(),
            PathBuf::from("/test/.codex/config.json")
        );
        assert_eq!(
            config.instructions_file_path(),
            PathBuf::from("/test/.codex/instructions.md")
        );
    }
}
