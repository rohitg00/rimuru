use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use super::ThemeManager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    #[serde(default = "default_theme_name")]
    pub theme: String,
}

fn default_theme_name() -> String {
    "Tokyo Night".to_string()
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            theme: default_theme_name(),
        }
    }
}

pub struct ThemeLoader {
    config_path: PathBuf,
}

impl ThemeLoader {
    pub fn new() -> Self {
        let config_path = Self::default_config_path();
        Self { config_path }
    }

    pub fn with_path(config_path: PathBuf) -> Self {
        Self { config_path }
    }

    fn default_config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("rimuru")
            .join("theme.toml")
    }

    pub fn load(&self) -> Result<ThemeConfig> {
        if !self.config_path.exists() {
            return Ok(ThemeConfig::default());
        }

        let contents = fs::read_to_string(&self.config_path)
            .with_context(|| format!("Failed to read theme config from {:?}", self.config_path))?;

        toml::from_str(&contents)
            .with_context(|| format!("Failed to parse theme config from {:?}", self.config_path))
    }

    pub fn save(&self, config: &ThemeConfig) -> Result<()> {
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory {:?}", parent))?;
        }

        let contents =
            toml::to_string_pretty(config).context("Failed to serialize theme config")?;

        fs::write(&self.config_path, contents)
            .with_context(|| format!("Failed to write theme config to {:?}", self.config_path))?;

        Ok(())
    }

    pub fn save_theme_name(&self, theme_name: &str) -> Result<()> {
        let config = ThemeConfig {
            theme: theme_name.to_string(),
        };
        self.save(&config)
    }

    pub fn load_theme_name(&self) -> String {
        self.load()
            .map(|c| c.theme)
            .unwrap_or_else(|_| default_theme_name())
    }

    pub fn initialize_theme_manager(&self) -> ThemeManager {
        let mut manager = ThemeManager::new();
        let theme_name = self.load_theme_name();

        if !manager.set_theme_by_name(&theme_name) {
            tracing::warn!(
                "Theme '{}' not found, using default '{}'",
                theme_name,
                manager.current_theme().name()
            );
        }

        manager
    }
}

impl Default for ThemeLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = ThemeConfig::default();
        assert_eq!(config.theme, "Tokyo Night");
    }

    #[test]
    fn test_load_nonexistent_returns_default() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("nonexistent.toml");
        let loader = ThemeLoader::with_path(config_path);

        let config = loader.load().unwrap();
        assert_eq!(config.theme, "Tokyo Night");
    }

    #[test]
    fn test_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("theme.toml");
        let loader = ThemeLoader::with_path(config_path);

        let config = ThemeConfig {
            theme: "Dracula".to_string(),
        };
        loader.save(&config).unwrap();

        let loaded = loader.load().unwrap();
        assert_eq!(loaded.theme, "Dracula");
    }

    #[test]
    fn test_save_theme_name() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("theme.toml");
        let loader = ThemeLoader::with_path(config_path);

        loader.save_theme_name("Nord").unwrap();

        let loaded = loader.load_theme_name();
        assert_eq!(loaded, "Nord");
    }

    #[test]
    fn test_initialize_theme_manager_with_valid_theme() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("theme.toml");
        let loader = ThemeLoader::with_path(config_path);

        loader.save_theme_name("Catppuccin Mocha").unwrap();
        let manager = loader.initialize_theme_manager();

        assert_eq!(manager.current_theme().name(), "Catppuccin Mocha");
    }

    #[test]
    fn test_initialize_theme_manager_with_invalid_theme() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("theme.toml");
        let loader = ThemeLoader::with_path(config_path);

        loader.save_theme_name("NonExistent Theme").unwrap();
        let manager = loader.initialize_theme_manager();

        assert_eq!(manager.current_theme().name(), "Tokyo Night");
    }
}
