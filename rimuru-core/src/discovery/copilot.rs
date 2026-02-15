use async_trait::async_trait;
use std::path::PathBuf;
use std::process::Command;
use tracing::{debug, info};

use crate::error::RimuruResult;
use crate::models::AgentType;

use super::{AgentDiscovery, AgentInstallation};

pub struct CopilotDiscovery {
    custom_config_dir: Option<PathBuf>,
    custom_vscode_extensions_dir: Option<PathBuf>,
    custom_jetbrains_plugins_dir: Option<PathBuf>,
}

impl CopilotDiscovery {
    pub fn new() -> Self {
        Self {
            custom_config_dir: None,
            custom_vscode_extensions_dir: None,
            custom_jetbrains_plugins_dir: None,
        }
    }

    pub fn with_config_dir(mut self, dir: PathBuf) -> Self {
        self.custom_config_dir = Some(dir);
        self
    }

    pub fn with_vscode_extensions_dir(mut self, dir: PathBuf) -> Self {
        self.custom_vscode_extensions_dir = Some(dir);
        self
    }

    pub fn with_jetbrains_plugins_dir(mut self, dir: PathBuf) -> Self {
        self.custom_jetbrains_plugins_dir = Some(dir);
        self
    }

    fn home_dir() -> Option<PathBuf> {
        dirs::home_dir()
    }

    fn check_config_exists(&self, path: &PathBuf) -> bool {
        path.exists() && path.is_dir()
    }

    fn check_hosts_file(&self, config_dir: &PathBuf) -> bool {
        let hosts_file = config_dir.join("hosts.json");
        hosts_file.exists()
    }

    fn check_apps_file(&self, config_dir: &PathBuf) -> bool {
        let apps_file = config_dir.join("apps.json");
        apps_file.exists()
    }

    fn default_vscode_extensions_dir() -> Option<PathBuf> {
        Self::home_dir().map(|home| home.join(".vscode").join("extensions"))
    }

    #[cfg(target_os = "macos")]
    fn default_jetbrains_plugins_dir() -> Option<PathBuf> {
        Self::home_dir().map(|home| {
            home.join("Library")
                .join("Application Support")
                .join("JetBrains")
        })
    }

    #[cfg(target_os = "linux")]
    fn default_jetbrains_plugins_dir() -> Option<PathBuf> {
        Self::home_dir().map(|home| home.join(".local").join("share").join("JetBrains"))
    }

    #[cfg(target_os = "windows")]
    fn default_jetbrains_plugins_dir() -> Option<PathBuf> {
        Self::home_dir().map(|home| home.join("AppData").join("Roaming").join("JetBrains"))
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    fn default_jetbrains_plugins_dir() -> Option<PathBuf> {
        Self::home_dir().map(|home| home.join(".local").join("share").join("JetBrains"))
    }

    fn find_vscode_copilot_extension(&self) -> Option<PathBuf> {
        let extensions_dir = self
            .custom_vscode_extensions_dir
            .clone()
            .or_else(Self::default_vscode_extensions_dir)?;

        if !extensions_dir.exists() {
            return None;
        }

        let entries = std::fs::read_dir(&extensions_dir).ok()?;
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("github.copilot-") {
                return Some(entry.path());
            }
        }
        None
    }

    fn find_jetbrains_copilot_plugin(&self) -> Option<PathBuf> {
        let plugins_dir = self
            .custom_jetbrains_plugins_dir
            .clone()
            .or_else(Self::default_jetbrains_plugins_dir)?;

        if !plugins_dir.exists() {
            return None;
        }

        let entries = std::fs::read_dir(&plugins_dir).ok()?;
        for entry in entries.flatten() {
            let ide_path = entry.path();
            if ide_path.is_dir() {
                let plugins_path = ide_path.join("plugins").join("github-copilot");
                if plugins_path.exists() {
                    return Some(plugins_path);
                }
            }
        }
        None
    }

    fn find_executable(&self) -> Option<PathBuf> {
        for exe_name in self.executable_names() {
            if let Ok(output) = Command::new("which").arg(exe_name).output() {
                if output.status.success() {
                    let path_str = String::from_utf8_lossy(&output.stdout);
                    let path = PathBuf::from(path_str.trim());
                    if path.exists() {
                        return Some(path);
                    }
                }
            }

            #[cfg(target_os = "windows")]
            if let Ok(output) = Command::new("where").arg(exe_name).output() {
                if output.status.success() {
                    let path_str = String::from_utf8_lossy(&output.stdout);
                    if let Some(first_line) = path_str.lines().next() {
                        let path = PathBuf::from(first_line.trim());
                        if path.exists() {
                            return Some(path);
                        }
                    }
                }
            }
        }
        None
    }

    fn parse_version_from_output(&self, output: &str) -> Option<String> {
        for line in output.lines() {
            let line = line.trim();

            if line.starts_with("Version:") || line.starts_with("version:") {
                if let Some(version) = line.split(':').nth(1) {
                    return Some(version.trim().to_string());
                }
            }

            if line.starts_with('v') && line.len() > 1 {
                let version = line.trim_start_matches('v');
                if version
                    .chars()
                    .next()
                    .map(|c| c.is_ascii_digit())
                    .unwrap_or(false)
                {
                    return Some(version.to_string());
                }
            }

            if line
                .chars()
                .next()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
            {
                return Some(line.to_string());
            }

            if line.starts_with("gh copilot") || line.starts_with("copilot") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                for part in parts {
                    let version = part.trim_start_matches('v');
                    if version
                        .chars()
                        .next()
                        .map(|c| c.is_ascii_digit())
                        .unwrap_or(false)
                    {
                        return Some(version.to_string());
                    }
                }
            }
        }
        None
    }

    fn parse_vscode_extension_version(&self, extension_path: &PathBuf) -> Option<String> {
        let dir_name = extension_path.file_name()?.to_string_lossy();
        if let Some(version_part) = dir_name.strip_prefix("github.copilot-") {
            let version = version_part.split('-').next()?;
            return Some(version.to_string());
        }
        None
    }

    fn is_configured(&self, config_dir: &PathBuf) -> bool {
        self.check_hosts_file(config_dir) || self.check_apps_file(config_dir)
    }
}

impl Default for CopilotDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AgentDiscovery for CopilotDiscovery {
    fn agent_type(&self) -> AgentType {
        AgentType::Copilot
    }

    async fn is_installed(&self) -> bool {
        if let Some(ref custom_dir) = self.custom_config_dir {
            return self.check_config_exists(custom_dir);
        }

        for path in self.default_config_locations() {
            if self.check_config_exists(&path) {
                debug!("Found Copilot config at {:?}", path);
                return true;
            }
        }

        if self.find_vscode_copilot_extension().is_some() {
            debug!("Found Copilot VS Code extension");
            return true;
        }

        if self.find_jetbrains_copilot_plugin().is_some() {
            debug!("Found Copilot JetBrains plugin");
            return true;
        }

        if self.find_executable().is_some() {
            debug!("Found Copilot CLI (gh copilot)");
            return true;
        }

        false
    }

    async fn discover(&self) -> RimuruResult<Option<AgentInstallation>> {
        let (config_dir, check_fallback) = if let Some(ref custom_dir) = self.custom_config_dir {
            if self.check_config_exists(custom_dir) {
                (Some(custom_dir.clone()), false)
            } else {
                (None, false)
            }
        } else {
            let found = self
                .default_config_locations()
                .into_iter()
                .find(|p| self.check_config_exists(p));
            (found, true)
        };

        let Some(config_dir) = config_dir else {
            if check_fallback {
                let vscode_ext = self.find_vscode_copilot_extension();
                let jetbrains_plugin = self.find_jetbrains_copilot_plugin();
                let executable = self.find_executable();

                if vscode_ext.is_some() || jetbrains_plugin.is_some() || executable.is_some() {
                    let home = Self::home_dir().unwrap_or_else(|| PathBuf::from("."));
                    let default_config = home.join(".config").join("github-copilot");

                    let version = if let Some(ref ext_path) = vscode_ext {
                        self.parse_vscode_extension_version(ext_path)
                    } else {
                        self.get_version().await.ok().flatten()
                    };

                    let mut install = AgentInstallation::new(
                        AgentType::Copilot,
                        "github-copilot",
                        default_config,
                    )
                    .with_configured(false);

                    if let Some(exe) = executable {
                        install = install.with_executable(exe);
                    }

                    if let Some(ref ext) = vscode_ext {
                        install = install.with_data_dir(ext.clone());
                    }

                    if let Some(v) = version {
                        install = install.with_version(v);
                    }

                    info!("Discovered GitHub Copilot via IDE extension/CLI (not yet configured)");
                    return Ok(Some(install));
                }
            }
            return Ok(None);
        };

        let executable = self.find_executable();
        let is_configured = self.is_configured(&config_dir);
        let version = self.get_version().await.ok().flatten();

        let vscode_ext = self.find_vscode_copilot_extension();
        let data_dir = vscode_ext.or_else(|| {
            let usage_dir = config_dir.join("usage");
            if usage_dir.exists() {
                Some(usage_dir)
            } else {
                Some(config_dir.clone())
            }
        });

        let mut install =
            AgentInstallation::new(AgentType::Copilot, "github-copilot", config_dir.clone())
                .with_configured(is_configured);

        if let Some(exe) = executable {
            install = install.with_executable(exe);
        }
        if let Some(data) = data_dir {
            install = install.with_data_dir(data);
        }
        if let Some(v) = version {
            install = install.with_version(v);
        }

        info!(
            "Discovered GitHub Copilot installation at {:?} (configured: {})",
            config_dir, is_configured
        );

        Ok(Some(install))
    }

    async fn get_version(&self) -> RimuruResult<Option<String>> {
        if let Some(ext_path) = self.find_vscode_copilot_extension() {
            if let Some(version) = self.parse_vscode_extension_version(&ext_path) {
                return Ok(Some(version));
            }
        }

        for exe_name in &["gh", "github-copilot-cli"] {
            if let Ok(output) = Command::new(exe_name)
                .args(["copilot", "--version"])
                .output()
            {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    if let Some(version) = self.parse_version_from_output(&stdout) {
                        return Ok(Some(version));
                    }
                }
            }
        }
        Ok(None)
    }

    fn default_config_locations(&self) -> Vec<PathBuf> {
        let mut locations = Vec::new();

        if let Some(home) = Self::home_dir() {
            locations.push(home.join(".config").join("github-copilot"));
        }

        if let Ok(xdg_config) = std::env::var("XDG_CONFIG_HOME") {
            locations.push(PathBuf::from(xdg_config).join("github-copilot"));
        }

        #[cfg(target_os = "macos")]
        if let Some(home) = Self::home_dir() {
            locations.push(
                home.join("Library")
                    .join("Application Support")
                    .join("GitHub Copilot"),
            );
        }

        #[cfg(target_os = "windows")]
        if let Ok(appdata) = std::env::var("APPDATA") {
            locations.push(PathBuf::from(appdata).join("GitHub Copilot"));
        }

        #[cfg(target_os = "windows")]
        if let Ok(localappdata) = std::env::var("LOCALAPPDATA") {
            locations.push(PathBuf::from(localappdata).join("GitHub Copilot"));
        }

        locations
    }

    fn executable_names(&self) -> Vec<&'static str> {
        vec!["gh", "github-copilot-cli"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_copilot_discovery_new() {
        let discovery = CopilotDiscovery::new();
        assert_eq!(discovery.agent_type(), AgentType::Copilot);
        assert!(discovery.custom_config_dir.is_none());
    }

    #[test]
    fn test_copilot_discovery_with_config_dir() {
        let discovery = CopilotDiscovery::new()
            .with_config_dir(PathBuf::from("/custom/.config/github-copilot"));

        assert_eq!(
            discovery.custom_config_dir,
            Some(PathBuf::from("/custom/.config/github-copilot"))
        );
    }

    #[test]
    fn test_copilot_discovery_with_vscode_dir() {
        let discovery = CopilotDiscovery::new()
            .with_vscode_extensions_dir(PathBuf::from("/custom/.vscode/extensions"));

        assert_eq!(
            discovery.custom_vscode_extensions_dir,
            Some(PathBuf::from("/custom/.vscode/extensions"))
        );
    }

    #[test]
    fn test_copilot_discovery_with_jetbrains_dir() {
        let discovery =
            CopilotDiscovery::new().with_jetbrains_plugins_dir(PathBuf::from("/custom/JetBrains"));

        assert_eq!(
            discovery.custom_jetbrains_plugins_dir,
            Some(PathBuf::from("/custom/JetBrains"))
        );
    }

    #[test]
    fn test_default_config_locations() {
        let discovery = CopilotDiscovery::new();
        let locations = discovery.default_config_locations();

        assert!(!locations.is_empty());

        let has_copilot_dir = locations.iter().any(|p| {
            p.to_string_lossy().contains("copilot") || p.to_string_lossy().contains("Copilot")
        });
        assert!(has_copilot_dir);
    }

    #[test]
    fn test_executable_names() {
        let discovery = CopilotDiscovery::new();
        let names = discovery.executable_names();

        assert!(names.contains(&"gh"));
        assert!(names.contains(&"github-copilot-cli"));
    }

    #[tokio::test]
    async fn test_discover_with_temp_config() {
        let temp_dir = tempdir().unwrap();
        let config_dir = temp_dir.path().to_path_buf();

        let discovery = CopilotDiscovery::new().with_config_dir(config_dir.clone());

        let result = discovery.discover().await.unwrap();
        assert!(result.is_some());

        let install = result.unwrap();
        assert_eq!(install.agent_type, AgentType::Copilot);
        assert_eq!(install.name, "github-copilot");
        assert_eq!(install.config_dir, config_dir);
    }

    #[tokio::test]
    async fn test_discover_with_hosts_file() {
        let temp_dir = tempdir().unwrap();
        let config_dir = temp_dir.path().to_path_buf();

        std::fs::write(
            config_dir.join("hosts.json"),
            r#"{"github.com": {"oauth_token": "test"}}"#,
        )
        .unwrap();

        let discovery = CopilotDiscovery::new().with_config_dir(config_dir.clone());

        let result = discovery.discover().await.unwrap();
        assert!(result.is_some());

        let install = result.unwrap();
        assert!(install.is_configured);
    }

    #[tokio::test]
    async fn test_discover_with_apps_file() {
        let temp_dir = tempdir().unwrap();
        let config_dir = temp_dir.path().to_path_buf();

        std::fs::write(
            config_dir.join("apps.json"),
            r#"{"vscode": {"enabled": true}}"#,
        )
        .unwrap();

        let discovery = CopilotDiscovery::new().with_config_dir(config_dir.clone());

        let result = discovery.discover().await.unwrap();
        assert!(result.is_some());

        let install = result.unwrap();
        assert!(install.is_configured);
    }

    #[tokio::test]
    async fn test_is_installed_with_temp_config() {
        let temp_dir = tempdir().unwrap();
        let config_dir = temp_dir.path().to_path_buf();

        let discovery = CopilotDiscovery::new().with_config_dir(config_dir);

        assert!(discovery.is_installed().await);
    }

    #[tokio::test]
    async fn test_is_not_installed() {
        let discovery = CopilotDiscovery::new()
            .with_config_dir(PathBuf::from("/nonexistent/path/.config/github-copilot"));

        assert!(!discovery.is_installed().await);
    }

    #[test]
    fn test_parse_version_from_output() {
        let discovery = CopilotDiscovery::new();

        assert_eq!(
            discovery.parse_version_from_output("Version: 1.0.0"),
            Some("1.0.0".to_string())
        );

        assert_eq!(
            discovery.parse_version_from_output("version: 2.1.3"),
            Some("2.1.3".to_string())
        );

        assert_eq!(
            discovery.parse_version_from_output("v1.2.3"),
            Some("1.2.3".to_string())
        );

        assert_eq!(
            discovery.parse_version_from_output("1.0.0"),
            Some("1.0.0".to_string())
        );
    }

    #[test]
    fn test_parse_vscode_extension_version() {
        let discovery = CopilotDiscovery::new();

        let path = PathBuf::from("/home/user/.vscode/extensions/github.copilot-1.234.0");
        assert_eq!(
            discovery.parse_vscode_extension_version(&path),
            Some("1.234.0".to_string())
        );

        let path2 = PathBuf::from("/home/user/.vscode/extensions/github.copilot-chat-0.22.0");
        assert_eq!(
            discovery.parse_vscode_extension_version(&path2),
            Some("chat".to_string())
        );
    }

    #[test]
    fn test_check_config_exists() {
        let temp_dir = tempdir().unwrap();
        let discovery = CopilotDiscovery::new();

        assert!(discovery.check_config_exists(&temp_dir.path().to_path_buf()));

        assert!(!discovery.check_config_exists(&PathBuf::from("/nonexistent/path")));
    }

    #[test]
    fn test_check_hosts_file() {
        let temp_dir = tempdir().unwrap();
        let config_dir = temp_dir.path().to_path_buf();
        let discovery = CopilotDiscovery::new();

        assert!(!discovery.check_hosts_file(&config_dir));

        std::fs::write(config_dir.join("hosts.json"), "{}").unwrap();
        assert!(discovery.check_hosts_file(&config_dir));
    }

    #[test]
    fn test_check_apps_file() {
        let temp_dir = tempdir().unwrap();
        let config_dir = temp_dir.path().to_path_buf();
        let discovery = CopilotDiscovery::new();

        assert!(!discovery.check_apps_file(&config_dir));

        std::fs::write(config_dir.join("apps.json"), "{}").unwrap();
        assert!(discovery.check_apps_file(&config_dir));
    }

    #[test]
    fn test_find_vscode_extension_with_custom_dir() {
        let temp_dir = tempdir().unwrap();
        let extensions_dir = temp_dir.path().to_path_buf();
        std::fs::create_dir_all(extensions_dir.join("github.copilot-1.234.0")).unwrap();

        let discovery = CopilotDiscovery::new().with_vscode_extensions_dir(extensions_dir.clone());

        let result = discovery.find_vscode_copilot_extension();
        assert!(result.is_some());
        assert!(result
            .unwrap()
            .to_string_lossy()
            .contains("github.copilot-1.234.0"));
    }

    #[test]
    fn test_find_vscode_extension_not_found() {
        let temp_dir = tempdir().unwrap();
        let extensions_dir = temp_dir.path().to_path_buf();

        let discovery = CopilotDiscovery::new().with_vscode_extensions_dir(extensions_dir);

        let result = discovery.find_vscode_copilot_extension();
        assert!(result.is_none());
    }
}
