use async_trait::async_trait;
use std::path::PathBuf;
use std::process::Command;
use tracing::{debug, info};

use crate::error::RimuruResult;
use crate::models::AgentType;

use super::{AgentDiscovery, AgentInstallation};

pub struct CursorDiscovery {
    custom_config_dir: Option<PathBuf>,
    custom_app_data_dir: Option<PathBuf>,
}

impl CursorDiscovery {
    pub fn new() -> Self {
        Self {
            custom_config_dir: None,
            custom_app_data_dir: None,
        }
    }

    pub fn with_config_dir(mut self, dir: PathBuf) -> Self {
        self.custom_config_dir = Some(dir);
        self
    }

    pub fn with_app_data_dir(mut self, dir: PathBuf) -> Self {
        self.custom_app_data_dir = Some(dir);
        self
    }

    fn home_dir() -> Option<PathBuf> {
        dirs::home_dir()
    }

    fn check_config_exists(&self, path: &PathBuf) -> bool {
        path.exists() && path.is_dir()
    }

    fn check_settings_file(&self, config_dir: &PathBuf) -> bool {
        let settings_file = config_dir.join("User").join("settings.json");
        let settings_alt = config_dir.join("settings.json");
        settings_file.exists() || settings_alt.exists()
    }

    fn check_state_db(&self, app_data_dir: &PathBuf) -> bool {
        let state_db = app_data_dir
            .join("User")
            .join("globalStorage")
            .join("state.vscdb");
        state_db.exists()
    }

    #[cfg(target_os = "macos")]
    fn check_app_installation() -> bool {
        PathBuf::from("/Applications/Cursor.app").exists()
    }

    #[cfg(target_os = "linux")]
    fn check_app_installation() -> bool {
        let common_paths = [
            "/usr/share/applications/cursor.desktop",
            "/opt/Cursor/cursor",
            "/usr/bin/cursor",
        ];
        common_paths.iter().any(|p| PathBuf::from(p).exists())
    }

    #[cfg(target_os = "windows")]
    fn check_app_installation() -> bool {
        if let Some(local_app_data) = dirs::data_local_dir() {
            let cursor_path = local_app_data
                .join("Programs")
                .join("cursor")
                .join("Cursor.exe");
            if cursor_path.exists() {
                return true;
            }
        }
        false
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    fn check_app_installation() -> bool {
        false
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

        #[cfg(target_os = "macos")]
        {
            let app_path = PathBuf::from("/Applications/Cursor.app/Contents/MacOS/Cursor");
            if app_path.exists() {
                return Some(app_path);
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
                let parts: Vec<&str> = line.split_whitespace().collect();
                if !parts.is_empty() {
                    let version = parts[0].trim_start_matches('v');
                    return Some(version.to_string());
                }
            }

            if line.to_lowercase().contains("cursor") {
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

    #[cfg(target_os = "macos")]
    fn default_app_data_dir() -> Option<PathBuf> {
        Self::home_dir().map(|home| {
            home.join("Library")
                .join("Application Support")
                .join("Cursor")
        })
    }

    #[cfg(target_os = "linux")]
    fn default_app_data_dir() -> Option<PathBuf> {
        dirs::config_dir().map(|config| config.join("Cursor"))
    }

    #[cfg(target_os = "windows")]
    fn default_app_data_dir() -> Option<PathBuf> {
        dirs::data_dir().map(|data| data.join("Cursor"))
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    fn default_app_data_dir() -> Option<PathBuf> {
        Self::home_dir().map(|home| home.join(".cursor"))
    }

    fn has_chat_history(&self, app_data_dir: &PathBuf) -> bool {
        let chat_dir = app_data_dir
            .join("User")
            .join("globalStorage")
            .join("cursor.chat");
        chat_dir.exists()
    }

    fn has_composer_history(&self, app_data_dir: &PathBuf) -> bool {
        let composer_dir = app_data_dir
            .join("User")
            .join("globalStorage")
            .join("cursor.composer");
        composer_dir.exists()
    }
}

impl Default for CursorDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AgentDiscovery for CursorDiscovery {
    fn agent_type(&self) -> AgentType {
        AgentType::Cursor
    }

    async fn is_installed(&self) -> bool {
        if let Some(ref custom_dir) = self.custom_config_dir {
            return self.check_config_exists(custom_dir);
        }

        if let Some(ref custom_app_dir) = self.custom_app_data_dir {
            if self.check_config_exists(custom_app_dir) {
                return true;
            }
        }

        for path in self.default_config_locations() {
            if self.check_config_exists(&path) {
                debug!("Found Cursor config at {:?}", path);
                return true;
            }
        }

        if Self::check_app_installation() {
            debug!("Found Cursor app installation");
            return true;
        }

        if self.find_executable().is_some() {
            debug!("Found Cursor executable");
            return true;
        }

        false
    }

    async fn discover(&self) -> RimuruResult<Option<AgentInstallation>> {
        let use_system_defaults =
            self.custom_config_dir.is_none() && self.custom_app_data_dir.is_none();

        let app_data_dir = if use_system_defaults {
            Self::default_app_data_dir()
        } else {
            self.custom_app_data_dir.clone()
        };

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
            (
                found.or_else(|| app_data_dir.clone().filter(|p| self.check_config_exists(p))),
                use_system_defaults,
            )
        };

        let Some(config_dir) = config_dir else {
            if check_exe_fallback
                && (Self::check_app_installation() || self.find_executable().is_some())
            {
                let default_config = Self::default_app_data_dir()
                    .unwrap_or_else(|| PathBuf::from(".").join(".cursor"));

                let executable = self.find_executable();
                let version = self.get_version().await.ok().flatten();

                let mut install =
                    AgentInstallation::new(AgentType::Cursor, "cursor", default_config)
                        .with_configured(false);

                if let Some(exe) = executable {
                    install = install.with_executable(exe);
                }
                if let Some(v) = version {
                    install = install.with_version(v);
                }

                info!("Discovered Cursor via app installation (not yet configured)");
                return Ok(Some(install));
            }
            return Ok(None);
        };

        let executable = self.find_executable();
        let effective_app_data = app_data_dir.as_ref().unwrap_or(&config_dir);

        let is_configured = self.check_settings_file(&config_dir)
            || self.check_state_db(effective_app_data)
            || self.has_chat_history(effective_app_data)
            || self.has_composer_history(effective_app_data);

        let version = self.get_version().await.ok().flatten();

        let data_dir = if self.has_chat_history(effective_app_data) {
            Some(
                effective_app_data
                    .join("User")
                    .join("globalStorage")
                    .join("cursor.chat"),
            )
        } else if self.has_composer_history(effective_app_data) {
            Some(
                effective_app_data
                    .join("User")
                    .join("globalStorage")
                    .join("cursor.composer"),
            )
        } else {
            Some(config_dir.clone())
        };

        let mut install = AgentInstallation::new(AgentType::Cursor, "cursor", config_dir.clone())
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
            "Discovered Cursor installation at {:?} (configured: {})",
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

        #[cfg(target_os = "macos")]
        {
            let plist_path = PathBuf::from("/Applications/Cursor.app/Contents/Info.plist");
            if plist_path.exists() {
                if let Ok(output) = Command::new("defaults")
                    .args([
                        "read",
                        "/Applications/Cursor.app/Contents/Info",
                        "CFBundleShortVersionString",
                    ])
                    .output()
                {
                    if output.status.success() {
                        let version = String::from_utf8_lossy(&output.stdout);
                        let version = version.trim();
                        if !version.is_empty() {
                            return Ok(Some(version.to_string()));
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    fn default_config_locations(&self) -> Vec<PathBuf> {
        let mut locations = Vec::new();

        #[cfg(target_os = "macos")]
        if let Some(home) = Self::home_dir() {
            locations.push(
                home.join("Library")
                    .join("Application Support")
                    .join("Cursor"),
            );
            locations.push(home.join(".cursor"));
        }

        #[cfg(target_os = "linux")]
        {
            if let Some(config_dir) = dirs::config_dir() {
                locations.push(config_dir.join("Cursor"));
            }
            if let Some(home) = Self::home_dir() {
                locations.push(home.join(".cursor"));
                locations.push(home.join(".config").join("Cursor"));
            }
        }

        #[cfg(target_os = "windows")]
        {
            if let Ok(appdata) = std::env::var("APPDATA") {
                locations.push(PathBuf::from(appdata).join("Cursor"));
            }
            if let Ok(localappdata) = std::env::var("LOCALAPPDATA") {
                locations.push(PathBuf::from(localappdata).join("Cursor"));
            }
            if let Some(home) = Self::home_dir() {
                locations.push(home.join(".cursor"));
            }
        }

        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        if let Some(home) = Self::home_dir() {
            locations.push(home.join(".cursor"));
        }

        if let Ok(xdg_config) = std::env::var("XDG_CONFIG_HOME") {
            locations.push(PathBuf::from(xdg_config).join("Cursor"));
        }

        locations
    }

    fn executable_names(&self) -> Vec<&'static str> {
        vec!["cursor", "Cursor"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_cursor_discovery_new() {
        let discovery = CursorDiscovery::new();
        assert_eq!(discovery.agent_type(), AgentType::Cursor);
        assert!(discovery.custom_config_dir.is_none());
    }

    #[test]
    fn test_cursor_discovery_with_config_dir() {
        let discovery = CursorDiscovery::new().with_config_dir(PathBuf::from("/custom/.cursor"));

        assert_eq!(
            discovery.custom_config_dir,
            Some(PathBuf::from("/custom/.cursor"))
        );
    }

    #[test]
    fn test_cursor_discovery_with_app_data_dir() {
        let discovery = CursorDiscovery::new().with_app_data_dir(PathBuf::from("/custom/Cursor"));

        assert_eq!(
            discovery.custom_app_data_dir,
            Some(PathBuf::from("/custom/Cursor"))
        );
    }

    #[test]
    fn test_default_config_locations() {
        let discovery = CursorDiscovery::new();
        let locations = discovery.default_config_locations();

        assert!(!locations.is_empty());

        let has_cursor_dir = locations.iter().any(|p| {
            p.to_string_lossy().contains("cursor") || p.to_string_lossy().contains("Cursor")
        });
        assert!(has_cursor_dir);
    }

    #[test]
    fn test_executable_names() {
        let discovery = CursorDiscovery::new();
        let names = discovery.executable_names();

        assert!(names.contains(&"cursor"));
        assert!(names.contains(&"Cursor"));
    }

    #[tokio::test]
    async fn test_discover_with_temp_config() {
        let temp_dir = tempdir().unwrap();
        let config_dir = temp_dir.path().to_path_buf();

        let discovery = CursorDiscovery::new().with_config_dir(config_dir.clone());

        let result = discovery.discover().await.unwrap();
        assert!(result.is_some());

        let install = result.unwrap();
        assert_eq!(install.agent_type, AgentType::Cursor);
        assert_eq!(install.name, "cursor");
        assert_eq!(install.config_dir, config_dir);
    }

    #[tokio::test]
    async fn test_discover_with_settings_file() {
        let temp_dir = tempdir().unwrap();
        let config_dir = temp_dir.path().to_path_buf();
        let user_dir = config_dir.join("User");
        std::fs::create_dir_all(&user_dir).unwrap();

        std::fs::write(user_dir.join("settings.json"), "{}").unwrap();

        let discovery = CursorDiscovery::new().with_config_dir(config_dir.clone());

        let result = discovery.discover().await.unwrap();
        assert!(result.is_some());

        let install = result.unwrap();
        assert!(install.is_configured);
    }

    #[tokio::test]
    async fn test_discover_with_chat_history() {
        let temp_dir = tempdir().unwrap();
        let config_dir = temp_dir.path().to_path_buf();
        let chat_dir = config_dir
            .join("User")
            .join("globalStorage")
            .join("cursor.chat");
        std::fs::create_dir_all(&chat_dir).unwrap();

        let discovery = CursorDiscovery::new()
            .with_config_dir(config_dir.clone())
            .with_app_data_dir(config_dir.clone());

        let result = discovery.discover().await.unwrap();
        assert!(result.is_some());

        let install = result.unwrap();
        assert!(install.is_configured);
        assert!(install.data_dir.is_some());
        assert!(install
            .data_dir
            .unwrap()
            .to_string_lossy()
            .contains("cursor.chat"));
    }

    #[tokio::test]
    async fn test_discover_with_composer_history() {
        let temp_dir = tempdir().unwrap();
        let config_dir = temp_dir.path().to_path_buf();
        let composer_dir = config_dir
            .join("User")
            .join("globalStorage")
            .join("cursor.composer");
        std::fs::create_dir_all(&composer_dir).unwrap();

        let discovery = CursorDiscovery::new()
            .with_config_dir(config_dir.clone())
            .with_app_data_dir(config_dir.clone());

        let result = discovery.discover().await.unwrap();
        assert!(result.is_some());

        let install = result.unwrap();
        assert!(install.is_configured);
    }

    #[tokio::test]
    async fn test_is_installed_with_temp_config() {
        let temp_dir = tempdir().unwrap();
        let config_dir = temp_dir.path().to_path_buf();

        let discovery = CursorDiscovery::new().with_config_dir(config_dir);

        assert!(discovery.is_installed().await);
    }

    #[tokio::test]
    async fn test_is_not_installed() {
        let discovery =
            CursorDiscovery::new().with_config_dir(PathBuf::from("/nonexistent/path/.cursor"));

        assert!(!discovery.is_installed().await);
    }

    #[test]
    fn test_parse_version_from_output() {
        let discovery = CursorDiscovery::new();

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
            discovery.parse_version_from_output("Cursor 0.45.0\nBuild: 12345"),
            Some("0.45.0".to_string())
        );
    }

    #[test]
    fn test_check_config_exists() {
        let temp_dir = tempdir().unwrap();
        let discovery = CursorDiscovery::new();

        assert!(discovery.check_config_exists(&temp_dir.path().to_path_buf()));

        assert!(!discovery.check_config_exists(&PathBuf::from("/nonexistent/path")));
    }

    #[test]
    fn test_check_settings_file() {
        let temp_dir = tempdir().unwrap();
        let config_dir = temp_dir.path().to_path_buf();
        let user_dir = config_dir.join("User");
        std::fs::create_dir_all(&user_dir).unwrap();
        let discovery = CursorDiscovery::new();

        assert!(!discovery.check_settings_file(&config_dir));

        std::fs::write(user_dir.join("settings.json"), "{}").unwrap();
        assert!(discovery.check_settings_file(&config_dir));
    }

    #[test]
    fn test_check_settings_file_alt() {
        let temp_dir = tempdir().unwrap();
        let config_dir = temp_dir.path().to_path_buf();
        let discovery = CursorDiscovery::new();

        std::fs::write(config_dir.join("settings.json"), "{}").unwrap();
        assert!(discovery.check_settings_file(&config_dir));
    }

    #[test]
    fn test_has_chat_history() {
        let temp_dir = tempdir().unwrap();
        let app_data_dir = temp_dir.path().to_path_buf();
        let discovery = CursorDiscovery::new();

        assert!(!discovery.has_chat_history(&app_data_dir));

        let chat_dir = app_data_dir
            .join("User")
            .join("globalStorage")
            .join("cursor.chat");
        std::fs::create_dir_all(&chat_dir).unwrap();
        assert!(discovery.has_chat_history(&app_data_dir));
    }

    #[test]
    fn test_has_composer_history() {
        let temp_dir = tempdir().unwrap();
        let app_data_dir = temp_dir.path().to_path_buf();
        let discovery = CursorDiscovery::new();

        assert!(!discovery.has_composer_history(&app_data_dir));

        let composer_dir = app_data_dir
            .join("User")
            .join("globalStorage")
            .join("cursor.composer");
        std::fs::create_dir_all(&composer_dir).unwrap();
        assert!(discovery.has_composer_history(&app_data_dir));
    }
}
