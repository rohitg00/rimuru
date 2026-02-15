use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorConfig {
    pub app_data_dir: PathBuf,
    pub config_dir: PathBuf,
    pub logs_dir: PathBuf,
    pub extensions_dir: PathBuf,
    pub auto_detect: bool,
}

impl Default for CursorConfig {
    fn default() -> Self {
        let (app_data_dir, config_dir, logs_dir, extensions_dir) = Self::default_paths();

        Self {
            app_data_dir,
            config_dir,
            logs_dir,
            extensions_dir,
            auto_detect: true,
        }
    }
}

impl CursorConfig {
    pub fn new() -> Self {
        Self::default()
    }

    fn default_paths() -> (PathBuf, PathBuf, PathBuf, PathBuf) {
        #[cfg(target_os = "macos")]
        {
            let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
            let app_support = home_dir.join("Library").join("Application Support");
            let app_data_dir = app_support.join("Cursor");
            let config_dir = app_data_dir.join("User");
            let logs_dir = app_data_dir.join("logs");
            let extensions_dir = home_dir.join(".cursor").join("extensions");
            (app_data_dir, config_dir, logs_dir, extensions_dir)
        }

        #[cfg(target_os = "linux")]
        {
            let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
            let config_home = dirs::config_dir().unwrap_or_else(|| home_dir.join(".config"));
            let app_data_dir = config_home.join("Cursor");
            let config_dir = app_data_dir.join("User");
            let logs_dir = app_data_dir.join("logs");
            let extensions_dir = home_dir.join(".cursor").join("extensions");
            (app_data_dir, config_dir, logs_dir, extensions_dir)
        }

        #[cfg(target_os = "windows")]
        {
            let app_data = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
            let app_data_dir = app_data.join("Programs").join("cursor");
            let roaming = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
            let config_dir = roaming.join("Cursor").join("User");
            let logs_dir = roaming.join("Cursor").join("logs");
            let extensions_dir = dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".cursor")
                .join("extensions");
            (app_data_dir, config_dir, logs_dir, extensions_dir)
        }

        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
            let app_data_dir = home_dir.join(".cursor");
            let config_dir = app_data_dir.join("config");
            let logs_dir = app_data_dir.join("logs");
            let extensions_dir = home_dir.join(".cursor").join("extensions");
            (app_data_dir, config_dir, logs_dir, extensions_dir)
        }
    }

    pub fn with_app_data_dir(mut self, path: PathBuf) -> Self {
        self.app_data_dir = path;
        self
    }

    pub fn with_config_dir(mut self, path: PathBuf) -> Self {
        self.config_dir = path;
        self
    }

    pub fn with_logs_dir(mut self, path: PathBuf) -> Self {
        self.logs_dir = path;
        self
    }

    pub fn with_extensions_dir(mut self, path: PathBuf) -> Self {
        self.extensions_dir = path;
        self
    }

    pub fn settings_path(&self) -> PathBuf {
        self.config_dir.join("settings.json")
    }

    pub fn keybindings_path(&self) -> PathBuf {
        self.config_dir.join("keybindings.json")
    }

    pub fn state_db_path(&self) -> PathBuf {
        self.app_data_dir
            .join("User")
            .join("globalStorage")
            .join("state.vscdb")
    }

    pub fn workspace_storage_path(&self) -> PathBuf {
        self.app_data_dir.join("User").join("workspaceStorage")
    }

    pub fn chat_history_path(&self) -> PathBuf {
        self.app_data_dir
            .join("User")
            .join("globalStorage")
            .join("cursor.chat")
    }

    pub fn composer_history_path(&self) -> PathBuf {
        self.app_data_dir
            .join("User")
            .join("globalStorage")
            .join("cursor.composer")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CursorSettings {
    #[serde(default)]
    pub cursor_tab: Option<CursorTabSettings>,
    #[serde(default)]
    pub cursor_chat: Option<CursorChatSettings>,
    #[serde(default)]
    pub cursor_composer: Option<CursorComposerSettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CursorTabSettings {
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CursorChatSettings {
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default, rename = "defaultProvider")]
    pub default_provider: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CursorComposerSettings {
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub model: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum CursorTier {
    #[default]
    Free,
    Pro,
    Business,
}

impl CursorTier {
    pub fn monthly_cost(&self) -> f64 {
        match self {
            CursorTier::Free => 0.0,
            CursorTier::Pro => 20.0,
            CursorTier::Business => 40.0,
        }
    }

    pub fn requests_included(&self) -> Option<i64> {
        match self {
            CursorTier::Free => Some(2000),
            CursorTier::Pro => Some(500),
            CursorTier::Business => Some(500),
        }
    }

    pub fn premium_requests_included(&self) -> Option<i64> {
        match self {
            CursorTier::Free => None,
            CursorTier::Pro => Some(500),
            CursorTier::Business => Some(500),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = CursorConfig::default();
        assert!(config.auto_detect);
    }

    #[test]
    fn test_builder_pattern() {
        let config = CursorConfig::new()
            .with_app_data_dir(PathBuf::from("/custom/app"))
            .with_config_dir(PathBuf::from("/custom/config"))
            .with_logs_dir(PathBuf::from("/custom/logs"))
            .with_extensions_dir(PathBuf::from("/custom/extensions"));

        assert_eq!(config.app_data_dir, PathBuf::from("/custom/app"));
        assert_eq!(config.config_dir, PathBuf::from("/custom/config"));
        assert_eq!(config.logs_dir, PathBuf::from("/custom/logs"));
        assert_eq!(config.extensions_dir, PathBuf::from("/custom/extensions"));
    }

    #[test]
    fn test_path_helpers() {
        let config = CursorConfig::new().with_config_dir(PathBuf::from("/test/config"));

        assert_eq!(
            config.settings_path(),
            PathBuf::from("/test/config/settings.json")
        );
        assert_eq!(
            config.keybindings_path(),
            PathBuf::from("/test/config/keybindings.json")
        );
    }

    #[test]
    fn test_cursor_tier_costs() {
        assert_eq!(CursorTier::Free.monthly_cost(), 0.0);
        assert_eq!(CursorTier::Pro.monthly_cost(), 20.0);
        assert_eq!(CursorTier::Business.monthly_cost(), 40.0);
    }

    #[test]
    fn test_cursor_tier_requests() {
        assert_eq!(CursorTier::Free.requests_included(), Some(2000));
        assert_eq!(CursorTier::Pro.requests_included(), Some(500));
        assert_eq!(CursorTier::Pro.premium_requests_included(), Some(500));
    }
}
