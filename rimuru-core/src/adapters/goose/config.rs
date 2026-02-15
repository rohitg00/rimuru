use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GooseConfig {
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub sessions_dir: PathBuf,
    pub auto_detect: bool,
}

impl Default for GooseConfig {
    fn default() -> Self {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        let config_dir = home_dir.join(".config").join("goose");

        Self {
            config_dir: config_dir.clone(),
            data_dir: config_dir.clone(),
            sessions_dir: config_dir.join("sessions"),
            auto_detect: true,
        }
    }
}

impl GooseConfig {
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

    pub fn with_sessions_dir(mut self, path: PathBuf) -> Self {
        self.sessions_dir = path;
        self
    }

    pub fn config_file_path(&self) -> PathBuf {
        self.config_dir.join("config.yaml")
    }

    pub fn profiles_dir(&self) -> PathBuf {
        self.config_dir.join("profiles")
    }

    pub fn database_path(&self) -> PathBuf {
        self.data_dir.join("goose.db")
    }

    pub fn session_history_path(&self) -> PathBuf {
        self.sessions_dir.clone()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GooseSettings {
    #[serde(default)]
    pub default_profile: Option<String>,
    #[serde(default)]
    pub provider: Option<GooseProviderConfig>,
    #[serde(default)]
    pub extensions: Option<Vec<GooseExtensionConfig>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GooseProviderConfig {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub api_key_env: Option<String>,
    #[serde(default)]
    pub base_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GooseExtensionConfig {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub config: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GooseProfile {
    pub name: String,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub extensions: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = GooseConfig::default();
        assert!(config.config_dir.ends_with("goose"));
        assert!(config.auto_detect);
    }

    #[test]
    fn test_builder_pattern() {
        let config = GooseConfig::new()
            .with_config_dir(PathBuf::from("/custom/config"))
            .with_data_dir(PathBuf::from("/custom/data"))
            .with_sessions_dir(PathBuf::from("/custom/sessions"));

        assert_eq!(config.config_dir, PathBuf::from("/custom/config"));
        assert_eq!(config.data_dir, PathBuf::from("/custom/data"));
        assert_eq!(config.sessions_dir, PathBuf::from("/custom/sessions"));
    }

    #[test]
    fn test_path_helpers() {
        let config = GooseConfig::new().with_config_dir(PathBuf::from("/test/.config/goose"));

        assert_eq!(
            config.config_file_path(),
            PathBuf::from("/test/.config/goose/config.yaml")
        );
        assert_eq!(
            config.profiles_dir(),
            PathBuf::from("/test/.config/goose/profiles")
        );
    }

    #[test]
    fn test_database_path() {
        let config = GooseConfig::new().with_data_dir(PathBuf::from("/data/goose"));

        assert_eq!(
            config.database_path(),
            PathBuf::from("/data/goose/goose.db")
        );
    }
}
