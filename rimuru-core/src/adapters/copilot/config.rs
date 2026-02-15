use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum CopilotProduct {
    #[default]
    Individual,
    Business,
    Enterprise,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopilotConfig {
    pub vscode_extensions_dir: PathBuf,
    pub jetbrains_plugins_dir: PathBuf,
    pub cli_config_dir: PathBuf,
    pub github_copilot_dir: PathBuf,
    pub auto_detect: bool,
    pub product: CopilotProduct,
}

impl Default for CopilotConfig {
    fn default() -> Self {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));

        #[cfg(target_os = "macos")]
        let vscode_extensions_dir = home_dir.join(".vscode/extensions");
        #[cfg(target_os = "linux")]
        let vscode_extensions_dir = home_dir.join(".vscode/extensions");
        #[cfg(target_os = "windows")]
        let vscode_extensions_dir = home_dir.join(".vscode/extensions");

        #[cfg(target_os = "macos")]
        let jetbrains_plugins_dir = home_dir.join("Library/Application Support/JetBrains");
        #[cfg(target_os = "linux")]
        let jetbrains_plugins_dir = home_dir.join(".local/share/JetBrains");
        #[cfg(target_os = "windows")]
        let jetbrains_plugins_dir = home_dir.join("AppData/Roaming/JetBrains");

        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        let vscode_extensions_dir = home_dir.join(".vscode/extensions");
        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        let jetbrains_plugins_dir = home_dir.join(".local/share/JetBrains");

        Self {
            vscode_extensions_dir,
            jetbrains_plugins_dir,
            cli_config_dir: home_dir.join(".config/github-copilot"),
            github_copilot_dir: home_dir.join(".config/github-copilot"),
            auto_detect: true,
            product: CopilotProduct::Individual,
        }
    }
}

impl CopilotConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_vscode_extensions_dir(mut self, path: PathBuf) -> Self {
        self.vscode_extensions_dir = path;
        self
    }

    pub fn with_jetbrains_plugins_dir(mut self, path: PathBuf) -> Self {
        self.jetbrains_plugins_dir = path;
        self
    }

    pub fn with_cli_config_dir(mut self, path: PathBuf) -> Self {
        self.cli_config_dir = path;
        self
    }

    pub fn with_github_copilot_dir(mut self, path: PathBuf) -> Self {
        self.github_copilot_dir = path;
        self
    }

    pub fn with_product(mut self, product: CopilotProduct) -> Self {
        self.product = product;
        self
    }

    pub fn hosts_file(&self) -> PathBuf {
        self.github_copilot_dir.join("hosts.json")
    }

    pub fn apps_file(&self) -> PathBuf {
        self.github_copilot_dir.join("apps.json")
    }

    pub fn versions_file(&self) -> PathBuf {
        self.github_copilot_dir.join("versions.json")
    }

    pub fn usage_cache_dir(&self) -> PathBuf {
        self.github_copilot_dir.join("usage")
    }

    pub fn find_vscode_copilot_extension(&self) -> Option<PathBuf> {
        if !self.vscode_extensions_dir.exists() {
            return None;
        }

        let entries = std::fs::read_dir(&self.vscode_extensions_dir).ok()?;
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("github.copilot-") && !name.contains("chat") {
                return Some(entry.path());
            }
        }
        None
    }

    pub fn find_vscode_copilot_chat_extension(&self) -> Option<PathBuf> {
        if !self.vscode_extensions_dir.exists() {
            return None;
        }

        let entries = std::fs::read_dir(&self.vscode_extensions_dir).ok()?;
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("github.copilot-chat-") {
                return Some(entry.path());
            }
        }
        None
    }

    pub fn find_jetbrains_copilot_plugin(&self) -> Option<PathBuf> {
        if !self.jetbrains_plugins_dir.exists() {
            return None;
        }

        let entries = std::fs::read_dir(&self.jetbrains_plugins_dir).ok()?;
        for entry in entries.flatten() {
            let ide_path = entry.path();
            if ide_path.is_dir() {
                let plugins_path = ide_path.join("plugins/github-copilot");
                if plugins_path.exists() {
                    return Some(plugins_path);
                }
            }
        }
        None
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CopilotHostsConfig {
    #[serde(default)]
    pub hosts: Vec<CopilotHost>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopilotHost {
    #[serde(default)]
    pub host: Option<String>,
    #[serde(default)]
    pub oauth_token: Option<String>,
    #[serde(default)]
    pub user: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CopilotAppsConfig {
    #[serde(default)]
    pub vscode: Option<CopilotAppEntry>,
    #[serde(default)]
    pub jetbrains: Option<CopilotAppEntry>,
    #[serde(default)]
    pub neovim: Option<CopilotAppEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopilotAppEntry {
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub telemetry_enabled: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = CopilotConfig::default();
        assert!(config.auto_detect);
        assert_eq!(config.product, CopilotProduct::Individual);
    }

    #[test]
    fn test_builder_pattern() {
        let config = CopilotConfig::new()
            .with_vscode_extensions_dir(PathBuf::from("/custom/vscode"))
            .with_jetbrains_plugins_dir(PathBuf::from("/custom/jetbrains"))
            .with_cli_config_dir(PathBuf::from("/custom/cli"))
            .with_github_copilot_dir(PathBuf::from("/custom/copilot"))
            .with_product(CopilotProduct::Business);

        assert_eq!(
            config.vscode_extensions_dir,
            PathBuf::from("/custom/vscode")
        );
        assert_eq!(
            config.jetbrains_plugins_dir,
            PathBuf::from("/custom/jetbrains")
        );
        assert_eq!(config.cli_config_dir, PathBuf::from("/custom/cli"));
        assert_eq!(config.github_copilot_dir, PathBuf::from("/custom/copilot"));
        assert_eq!(config.product, CopilotProduct::Business);
    }

    #[test]
    fn test_path_helpers() {
        let config =
            CopilotConfig::new().with_github_copilot_dir(PathBuf::from("/test/.config/copilot"));

        assert_eq!(
            config.hosts_file(),
            PathBuf::from("/test/.config/copilot/hosts.json")
        );
        assert_eq!(
            config.apps_file(),
            PathBuf::from("/test/.config/copilot/apps.json")
        );
        assert_eq!(
            config.versions_file(),
            PathBuf::from("/test/.config/copilot/versions.json")
        );
    }

    #[test]
    fn test_copilot_product_default() {
        let product = CopilotProduct::default();
        assert_eq!(product, CopilotProduct::Individual);
    }
}
