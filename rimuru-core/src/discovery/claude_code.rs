use async_trait::async_trait;
use std::path::PathBuf;
use std::process::Command;
use tracing::{debug, info};

use crate::error::RimuruResult;
use crate::models::AgentType;

use super::{AgentDiscovery, AgentInstallation};

pub struct ClaudeCodeDiscovery {
    custom_config_dir: Option<PathBuf>,
}

impl ClaudeCodeDiscovery {
    pub fn new() -> Self {
        Self {
            custom_config_dir: None,
        }
    }

    pub fn with_config_dir(mut self, dir: PathBuf) -> Self {
        self.custom_config_dir = Some(dir);
        self
    }

    fn home_dir() -> Option<PathBuf> {
        dirs::home_dir()
    }

    fn check_config_exists(&self, path: &PathBuf) -> bool {
        path.exists() && path.is_dir()
    }

    fn check_settings_file(&self, config_dir: &PathBuf) -> bool {
        let settings_file = config_dir.join("settings.json");
        let settings_local_file = config_dir.join("settings.local.json");
        settings_file.exists() || settings_local_file.exists()
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
        }
        None
    }
}

impl Default for ClaudeCodeDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AgentDiscovery for ClaudeCodeDiscovery {
    fn agent_type(&self) -> AgentType {
        AgentType::ClaudeCode
    }

    async fn is_installed(&self) -> bool {
        if let Some(ref custom_dir) = self.custom_config_dir {
            return self.check_config_exists(custom_dir);
        }

        for path in self.default_config_locations() {
            if self.check_config_exists(&path) {
                debug!("Found Claude Code config at {:?}", path);
                return true;
            }
        }

        if self.find_executable().is_some() {
            debug!("Found Claude Code executable");
            return true;
        }

        false
    }

    async fn discover(&self) -> RimuruResult<Option<AgentInstallation>> {
        let (config_dir, check_exe_fallback) = if let Some(ref custom_dir) = self.custom_config_dir
        {
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
            if check_exe_fallback {
                if let Some(exe_path) = self.find_executable() {
                    let home = Self::home_dir().unwrap_or_else(|| PathBuf::from("."));
                    let default_config = home.join(".claude");

                    let version = self.get_version().await.ok().flatten();

                    let mut install = AgentInstallation::new(
                        AgentType::ClaudeCode,
                        "claude-code",
                        default_config,
                    )
                    .with_executable(exe_path)
                    .with_configured(false);

                    if let Some(v) = version {
                        install = install.with_version(v);
                    }

                    info!("Discovered Claude Code via executable (not yet configured)");
                    return Ok(Some(install));
                }
            }
            return Ok(None);
        };

        let executable = self.find_executable();
        let is_configured = self.check_settings_file(&config_dir);
        let version = self.get_version().await.ok().flatten();

        let projects_dir = config_dir.join("projects");
        let data_dir = if projects_dir.exists() {
            Some(projects_dir)
        } else {
            None
        };

        let mut install =
            AgentInstallation::new(AgentType::ClaudeCode, "claude-code", config_dir.clone())
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
            "Discovered Claude Code installation at {:?} (configured: {})",
            config_dir, is_configured
        );

        Ok(Some(install))
    }

    async fn get_version(&self) -> RimuruResult<Option<String>> {
        for exe_name in self.executable_names() {
            if let Ok(output) = Command::new(exe_name).arg("--version").output() {
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
            locations.push(home.join(".claude"));
        }

        if let Ok(xdg_config) = std::env::var("XDG_CONFIG_HOME") {
            locations.push(PathBuf::from(xdg_config).join("claude"));
        }

        #[cfg(target_os = "macos")]
        if let Some(home) = Self::home_dir() {
            locations.push(
                home.join("Library")
                    .join("Application Support")
                    .join("Claude"),
            );
        }

        #[cfg(target_os = "windows")]
        if let Ok(appdata) = std::env::var("APPDATA") {
            locations.push(PathBuf::from(appdata).join("Claude"));
        }

        #[cfg(target_os = "windows")]
        if let Ok(localappdata) = std::env::var("LOCALAPPDATA") {
            locations.push(PathBuf::from(localappdata).join("Claude"));
        }

        locations
    }

    fn executable_names(&self) -> Vec<&'static str> {
        vec!["claude", "claude-code"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_claude_code_discovery_new() {
        let discovery = ClaudeCodeDiscovery::new();
        assert_eq!(discovery.agent_type(), AgentType::ClaudeCode);
        assert!(discovery.custom_config_dir.is_none());
    }

    #[test]
    fn test_claude_code_discovery_with_config_dir() {
        let discovery =
            ClaudeCodeDiscovery::new().with_config_dir(PathBuf::from("/custom/.claude"));

        assert_eq!(
            discovery.custom_config_dir,
            Some(PathBuf::from("/custom/.claude"))
        );
    }

    #[test]
    fn test_default_config_locations() {
        let discovery = ClaudeCodeDiscovery::new();
        let locations = discovery.default_config_locations();

        assert!(!locations.is_empty());

        let has_claude_dir = locations.iter().any(|p| {
            p.to_string_lossy().contains(".claude") || p.to_string_lossy().contains("Claude")
        });
        assert!(has_claude_dir);
    }

    #[test]
    fn test_executable_names() {
        let discovery = ClaudeCodeDiscovery::new();
        let names = discovery.executable_names();

        assert!(names.contains(&"claude"));
        assert!(names.contains(&"claude-code"));
    }

    #[tokio::test]
    async fn test_discover_with_temp_config() {
        let temp_dir = tempdir().unwrap();
        let config_dir = temp_dir.path().to_path_buf();

        let discovery = ClaudeCodeDiscovery::new().with_config_dir(config_dir.clone());

        let result = discovery.discover().await.unwrap();
        assert!(result.is_some());

        let install = result.unwrap();
        assert_eq!(install.agent_type, AgentType::ClaudeCode);
        assert_eq!(install.name, "claude-code");
        assert_eq!(install.config_dir, config_dir);
        assert!(!install.is_configured);
    }

    #[tokio::test]
    async fn test_discover_with_settings_file() {
        let temp_dir = tempdir().unwrap();
        let config_dir = temp_dir.path().to_path_buf();

        std::fs::write(config_dir.join("settings.json"), "{}").unwrap();

        let discovery = ClaudeCodeDiscovery::new().with_config_dir(config_dir.clone());

        let result = discovery.discover().await.unwrap();
        assert!(result.is_some());

        let install = result.unwrap();
        assert!(install.is_configured);
    }

    #[tokio::test]
    async fn test_is_installed_with_temp_config() {
        let temp_dir = tempdir().unwrap();
        let config_dir = temp_dir.path().to_path_buf();

        let discovery = ClaudeCodeDiscovery::new().with_config_dir(config_dir);

        assert!(discovery.is_installed().await);
    }

    #[tokio::test]
    async fn test_is_not_installed() {
        let discovery =
            ClaudeCodeDiscovery::new().with_config_dir(PathBuf::from("/nonexistent/path/.claude"));

        assert!(!discovery.is_installed().await);
    }

    #[test]
    fn test_parse_version_from_output() {
        let discovery = ClaudeCodeDiscovery::new();

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

        assert_eq!(
            discovery.parse_version_from_output("claude-code\nv1.5.0\nBuild: 12345"),
            Some("1.5.0".to_string())
        );
    }

    #[test]
    fn test_check_config_exists() {
        let temp_dir = tempdir().unwrap();
        let discovery = ClaudeCodeDiscovery::new();

        assert!(discovery.check_config_exists(&temp_dir.path().to_path_buf()));

        assert!(!discovery.check_config_exists(&PathBuf::from("/nonexistent/path")));
    }

    #[test]
    fn test_check_settings_file() {
        let temp_dir = tempdir().unwrap();
        let config_dir = temp_dir.path().to_path_buf();
        let discovery = ClaudeCodeDiscovery::new();

        assert!(!discovery.check_settings_file(&config_dir));

        std::fs::write(config_dir.join("settings.json"), "{}").unwrap();
        assert!(discovery.check_settings_file(&config_dir));
    }

    #[test]
    fn test_check_settings_local_file() {
        let temp_dir = tempdir().unwrap();
        let config_dir = temp_dir.path().to_path_buf();
        let discovery = ClaudeCodeDiscovery::new();

        std::fs::write(config_dir.join("settings.local.json"), "{}").unwrap();
        assert!(discovery.check_settings_file(&config_dir));
    }
}
