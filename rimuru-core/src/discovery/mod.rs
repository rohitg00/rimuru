mod claude_code;
mod codex;
mod copilot;
mod cursor;
mod goose;
mod opencode;
mod scanner;

pub use claude_code::ClaudeCodeDiscovery;
pub use codex::CodexDiscovery;
pub use copilot::CopilotDiscovery;
pub use cursor::CursorDiscovery;
pub use goose::GooseDiscovery;
pub use opencode::OpenCodeDiscovery;
pub use scanner::{AgentScanner, DetectionMethod, DiscoveredAgent, ScanOptions, ScanResult};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::error::RimuruResult;
use crate::models::AgentType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInstallation {
    pub agent_type: AgentType,
    pub name: String,
    pub version: Option<String>,
    pub config_dir: PathBuf,
    pub data_dir: Option<PathBuf>,
    pub executable_path: Option<PathBuf>,
    pub is_configured: bool,
}

impl AgentInstallation {
    pub fn new(agent_type: AgentType, name: &str, config_dir: PathBuf) -> Self {
        Self {
            agent_type,
            name: name.to_string(),
            version: None,
            config_dir,
            data_dir: None,
            executable_path: None,
            is_configured: false,
        }
    }

    pub fn with_version(mut self, version: String) -> Self {
        self.version = Some(version);
        self
    }

    pub fn with_data_dir(mut self, data_dir: PathBuf) -> Self {
        self.data_dir = Some(data_dir);
        self
    }

    pub fn with_executable(mut self, path: PathBuf) -> Self {
        self.executable_path = Some(path);
        self
    }

    pub fn with_configured(mut self, configured: bool) -> Self {
        self.is_configured = configured;
        self
    }
}

#[async_trait]
pub trait AgentDiscovery: Send + Sync {
    fn agent_type(&self) -> AgentType;

    async fn is_installed(&self) -> bool;

    async fn discover(&self) -> RimuruResult<Option<AgentInstallation>>;

    async fn get_version(&self) -> RimuruResult<Option<String>>;

    fn default_config_locations(&self) -> Vec<PathBuf>;

    fn executable_names(&self) -> Vec<&'static str>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_installation_new() {
        let install = AgentInstallation::new(
            AgentType::ClaudeCode,
            "claude-code",
            PathBuf::from("/home/user/.claude"),
        );

        assert_eq!(install.agent_type, AgentType::ClaudeCode);
        assert_eq!(install.name, "claude-code");
        assert_eq!(install.config_dir, PathBuf::from("/home/user/.claude"));
        assert!(install.version.is_none());
        assert!(install.data_dir.is_none());
        assert!(install.executable_path.is_none());
        assert!(!install.is_configured);
    }

    #[test]
    fn test_agent_installation_builder() {
        let install = AgentInstallation::new(
            AgentType::OpenCode,
            "opencode",
            PathBuf::from("/home/user/.opencode"),
        )
        .with_version("1.0.0".to_string())
        .with_data_dir(PathBuf::from("/home/user/.opencode/data"))
        .with_executable(PathBuf::from("/usr/local/bin/opencode"))
        .with_configured(true);

        assert_eq!(install.version, Some("1.0.0".to_string()));
        assert_eq!(
            install.data_dir,
            Some(PathBuf::from("/home/user/.opencode/data"))
        );
        assert_eq!(
            install.executable_path,
            Some(PathBuf::from("/usr/local/bin/opencode"))
        );
        assert!(install.is_configured);
    }
}
