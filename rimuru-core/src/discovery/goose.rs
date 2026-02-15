use async_trait::async_trait;
use std::path::PathBuf;
use std::process::Command;
use tracing::{debug, info};

use crate::error::RimuruResult;
use crate::models::AgentType;

use super::{AgentDiscovery, AgentInstallation};

pub struct GooseDiscovery {
    custom_config_dir: Option<PathBuf>,
}

impl GooseDiscovery {
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

    fn check_config_file(&self, config_dir: &PathBuf) -> bool {
        let config_yaml = config_dir.join("config.yaml");
        let config_json = config_dir.join("config.json");
        let profiles_dir = config_dir.join("profiles");

        config_yaml.exists() || config_json.exists() || profiles_dir.exists()
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

            if line.starts_with("goose") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let version = parts[1].trim_start_matches('v');
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
}

impl Default for GooseDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AgentDiscovery for GooseDiscovery {
    fn agent_type(&self) -> AgentType {
        AgentType::Goose
    }

    async fn is_installed(&self) -> bool {
        if let Some(ref custom_dir) = self.custom_config_dir {
            return self.check_config_exists(custom_dir);
        }

        for path in self.default_config_locations() {
            if self.check_config_exists(&path) {
                debug!("Found Goose config at {:?}", path);
                return true;
            }
        }

        if self.find_executable().is_some() {
            debug!("Found Goose executable");
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
                    let default_config = home.join(".config").join("goose");

                    let version = self.get_version().await.ok().flatten();

                    let mut install =
                        AgentInstallation::new(AgentType::Goose, "goose", default_config)
                            .with_executable(exe_path)
                            .with_configured(false);

                    if let Some(v) = version {
                        install = install.with_version(v);
                    }

                    info!("Discovered Goose via executable");
                    return Ok(Some(install));
                }
            }
            return Ok(None);
        };

        let executable = self.find_executable();
        let is_configured = self.check_config_file(&config_dir);
        let version = self.get_version().await.ok().flatten();

        let sessions_dir = config_dir.join("sessions");
        let data_dir = if sessions_dir.exists() {
            Some(sessions_dir)
        } else {
            Some(config_dir.clone())
        };

        let mut install = AgentInstallation::new(AgentType::Goose, "goose", config_dir.clone())
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
            "Discovered Goose installation at {:?} (configured: {})",
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

            if let Ok(output) = Command::new(exe_name).arg("version").output() {
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
            locations.push(home.join(".config").join("goose"));
            locations.push(home.join(".goose"));
        }

        if let Ok(xdg_config) = std::env::var("XDG_CONFIG_HOME") {
            locations.push(PathBuf::from(xdg_config).join("goose"));
        }

        #[cfg(target_os = "macos")]
        if let Some(home) = Self::home_dir() {
            locations.push(
                home.join("Library")
                    .join("Application Support")
                    .join("Goose"),
            );
        }

        #[cfg(target_os = "windows")]
        if let Ok(appdata) = std::env::var("APPDATA") {
            locations.push(PathBuf::from(appdata).join("Goose"));
        }

        #[cfg(target_os = "windows")]
        if let Ok(localappdata) = std::env::var("LOCALAPPDATA") {
            locations.push(PathBuf::from(localappdata).join("Goose"));
        }

        locations
    }

    fn executable_names(&self) -> Vec<&'static str> {
        vec!["goose"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_goose_discovery_new() {
        let discovery = GooseDiscovery::new();
        assert_eq!(discovery.agent_type(), AgentType::Goose);
        assert!(discovery.custom_config_dir.is_none());
    }

    #[test]
    fn test_goose_discovery_with_config_dir() {
        let discovery =
            GooseDiscovery::new().with_config_dir(PathBuf::from("/custom/.config/goose"));

        assert_eq!(
            discovery.custom_config_dir,
            Some(PathBuf::from("/custom/.config/goose"))
        );
    }

    #[test]
    fn test_default_config_locations() {
        let discovery = GooseDiscovery::new();
        let locations = discovery.default_config_locations();

        assert!(!locations.is_empty());

        let has_goose_dir = locations.iter().any(|p| {
            p.to_string_lossy().contains("goose") || p.to_string_lossy().contains("Goose")
        });
        assert!(has_goose_dir);
    }

    #[test]
    fn test_executable_names() {
        let discovery = GooseDiscovery::new();
        let names = discovery.executable_names();

        assert!(names.contains(&"goose"));
    }

    #[tokio::test]
    async fn test_discover_with_temp_config() {
        let temp_dir = tempdir().unwrap();
        let config_dir = temp_dir.path().to_path_buf();

        let discovery = GooseDiscovery::new().with_config_dir(config_dir.clone());

        let result = discovery.discover().await.unwrap();
        assert!(result.is_some());

        let install = result.unwrap();
        assert_eq!(install.agent_type, AgentType::Goose);
        assert_eq!(install.name, "goose");
        assert_eq!(install.config_dir, config_dir);
    }

    #[tokio::test]
    async fn test_discover_with_config_file() {
        let temp_dir = tempdir().unwrap();
        let config_dir = temp_dir.path().to_path_buf();

        std::fs::write(config_dir.join("config.yaml"), "default_profile: default").unwrap();

        let discovery = GooseDiscovery::new().with_config_dir(config_dir.clone());

        let result = discovery.discover().await.unwrap();
        assert!(result.is_some());

        let install = result.unwrap();
        assert!(install.is_configured);
    }

    #[tokio::test]
    async fn test_discover_with_profiles_dir() {
        let temp_dir = tempdir().unwrap();
        let config_dir = temp_dir.path().to_path_buf();
        let profiles_dir = config_dir.join("profiles");
        std::fs::create_dir_all(&profiles_dir).unwrap();

        let discovery = GooseDiscovery::new().with_config_dir(config_dir.clone());

        let result = discovery.discover().await.unwrap();
        assert!(result.is_some());

        let install = result.unwrap();
        assert!(install.is_configured);
    }

    #[tokio::test]
    async fn test_is_installed_with_temp_config() {
        let temp_dir = tempdir().unwrap();
        let config_dir = temp_dir.path().to_path_buf();

        let discovery = GooseDiscovery::new().with_config_dir(config_dir);

        assert!(discovery.is_installed().await);
    }

    #[tokio::test]
    async fn test_is_not_installed() {
        let discovery =
            GooseDiscovery::new().with_config_dir(PathBuf::from("/nonexistent/path/.config/goose"));

        assert!(!discovery.is_installed().await);
    }

    #[test]
    fn test_parse_version_from_output() {
        let discovery = GooseDiscovery::new();

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
            discovery.parse_version_from_output("goose v0.5.0\nBuild: 12345"),
            Some("0.5.0".to_string())
        );
    }

    #[test]
    fn test_check_config_exists() {
        let temp_dir = tempdir().unwrap();
        let discovery = GooseDiscovery::new();

        assert!(discovery.check_config_exists(&temp_dir.path().to_path_buf()));

        assert!(!discovery.check_config_exists(&PathBuf::from("/nonexistent/path")));
    }

    #[test]
    fn test_check_config_file_yaml() {
        let temp_dir = tempdir().unwrap();
        let config_dir = temp_dir.path().to_path_buf();
        let discovery = GooseDiscovery::new();

        assert!(!discovery.check_config_file(&config_dir));

        std::fs::write(config_dir.join("config.yaml"), "key: value").unwrap();
        assert!(discovery.check_config_file(&config_dir));
    }

    #[test]
    fn test_check_config_file_json() {
        let temp_dir = tempdir().unwrap();
        let config_dir = temp_dir.path().to_path_buf();
        let discovery = GooseDiscovery::new();

        std::fs::write(config_dir.join("config.json"), "{}").unwrap();
        assert!(discovery.check_config_file(&config_dir));
    }
}
