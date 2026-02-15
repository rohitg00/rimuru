use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenCodeConfig {
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub auto_detect: bool,
}

impl Default for OpenCodeConfig {
    fn default() -> Self {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));

        Self {
            config_dir: home_dir.join(".opencode"),
            data_dir: home_dir.join(".opencode").join("data"),
            auto_detect: true,
        }
    }
}

impl OpenCodeConfig {
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

    pub fn sessions_dir(&self) -> PathBuf {
        self.data_dir.join("sessions")
    }

    pub fn state_file_path(&self) -> PathBuf {
        self.data_dir.join("state.json")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OpenCodeSettings {
    #[serde(default)]
    pub default_provider: Option<String>,
    #[serde(default)]
    pub default_model: Option<String>,
    #[serde(default)]
    pub providers: Vec<ProviderConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub name: String,
    #[serde(default)]
    pub api_key_env: Option<String>,
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default)]
    pub models: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = OpenCodeConfig::default();
        assert!(config.config_dir.ends_with(".opencode"));
        assert!(config.auto_detect);
    }

    #[test]
    fn test_builder_pattern() {
        let config = OpenCodeConfig::new()
            .with_config_dir(PathBuf::from("/custom/path"))
            .with_data_dir(PathBuf::from("/custom/data"));

        assert_eq!(config.config_dir, PathBuf::from("/custom/path"));
        assert_eq!(config.data_dir, PathBuf::from("/custom/data"));
    }

    #[test]
    fn test_paths() {
        let config = OpenCodeConfig::new().with_config_dir(PathBuf::from("/test/.opencode"));

        assert_eq!(
            config.config_file_path(),
            PathBuf::from("/test/.opencode/config.json")
        );
    }
}
