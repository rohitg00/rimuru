use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};

use crate::error::RimuruResult;
use crate::models::AgentType;

use super::{
    AgentDiscovery, AgentInstallation, ClaudeCodeDiscovery, CodexDiscovery, CopilotDiscovery,
    CursorDiscovery, GooseDiscovery, OpenCodeDiscovery,
};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScanOptions {
    pub include_unconfigured: bool,
    pub agent_types: Option<Vec<AgentType>>,
    pub skip_executable_check: bool,
}

impl ScanOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn include_unconfigured(mut self, include: bool) -> Self {
        self.include_unconfigured = include;
        self
    }

    pub fn with_agent_types(mut self, types: Vec<AgentType>) -> Self {
        self.agent_types = Some(types);
        self
    }

    pub fn skip_executable_check(mut self, skip: bool) -> Self {
        self.skip_executable_check = skip;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredAgent {
    pub installation: AgentInstallation,
    pub detection_method: DetectionMethod,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DetectionMethod {
    ConfigDirectory,
    Executable,
    Both,
}

impl DiscoveredAgent {
    pub fn new(installation: AgentInstallation, method: DetectionMethod) -> Self {
        Self {
            installation,
            detection_method: method,
        }
    }

    pub fn is_fully_detected(&self) -> bool {
        self.detection_method == DetectionMethod::Both
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScanResult {
    pub discovered: Vec<DiscoveredAgent>,
    pub errors: Vec<String>,
    pub scanned_types: Vec<AgentType>,
}

impl ScanResult {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_discovered(&mut self, agent: DiscoveredAgent) {
        self.discovered.push(agent);
    }

    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
    }

    pub fn add_scanned_type(&mut self, agent_type: AgentType) {
        if !self.scanned_types.contains(&agent_type) {
            self.scanned_types.push(agent_type);
        }
    }

    pub fn agents_count(&self) -> usize {
        self.discovered.len()
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub fn get_by_type(&self, agent_type: AgentType) -> Vec<&DiscoveredAgent> {
        self.discovered
            .iter()
            .filter(|a| a.installation.agent_type == agent_type)
            .collect()
    }

    pub fn configured_only(&self) -> Vec<&DiscoveredAgent> {
        self.discovered
            .iter()
            .filter(|a| a.installation.is_configured)
            .collect()
    }

    pub fn into_installations(self) -> Vec<AgentInstallation> {
        self.discovered
            .into_iter()
            .map(|a| a.installation)
            .collect()
    }
}

pub struct AgentScanner {
    discoveries: HashMap<AgentType, Box<dyn AgentDiscovery>>,
}

impl AgentScanner {
    pub fn new() -> Self {
        let mut discoveries: HashMap<AgentType, Box<dyn AgentDiscovery>> = HashMap::new();

        discoveries.insert(AgentType::ClaudeCode, Box::new(ClaudeCodeDiscovery::new()));
        discoveries.insert(AgentType::Codex, Box::new(CodexDiscovery::new()));
        discoveries.insert(AgentType::Copilot, Box::new(CopilotDiscovery::new()));
        discoveries.insert(AgentType::Cursor, Box::new(CursorDiscovery::new()));
        discoveries.insert(AgentType::Goose, Box::new(GooseDiscovery::new()));
        discoveries.insert(AgentType::OpenCode, Box::new(OpenCodeDiscovery::new()));

        Self { discoveries }
    }

    pub fn with_discovery<D: AgentDiscovery + 'static>(mut self, discovery: D) -> Self {
        self.discoveries
            .insert(discovery.agent_type(), Box::new(discovery));
        self
    }

    pub fn supported_types(&self) -> Vec<AgentType> {
        self.discoveries.keys().cloned().collect()
    }

    pub async fn scan(&self, options: &ScanOptions) -> RimuruResult<ScanResult> {
        let mut result = ScanResult::new();

        let types_to_scan: Vec<AgentType> = if let Some(ref types) = options.agent_types {
            types.clone()
        } else {
            self.supported_types()
        };

        info!("Starting agent scan for {:?}", types_to_scan);

        for agent_type in types_to_scan {
            result.add_scanned_type(agent_type);

            let Some(discovery) = self.discoveries.get(&agent_type) else {
                warn!("No discovery implementation for {:?}", agent_type);
                continue;
            };

            debug!("Scanning for {:?}", agent_type);

            match discovery.discover().await {
                Ok(Some(installation)) => {
                    if !installation.is_configured && !options.include_unconfigured {
                        debug!(
                            "Skipping unconfigured {:?} installation",
                            installation.agent_type
                        );
                        continue;
                    }

                    let method = match (
                        installation.executable_path.is_some(),
                        installation.is_configured,
                    ) {
                        (true, true) => DetectionMethod::Both,
                        (true, false) => DetectionMethod::Executable,
                        (false, _) => DetectionMethod::ConfigDirectory,
                    };

                    info!("Discovered {:?} via {:?}", installation.agent_type, method);

                    result.add_discovered(DiscoveredAgent::new(installation, method));
                }
                Ok(None) => {
                    debug!("No {:?} installation found", agent_type);
                }
                Err(e) => {
                    let error_msg = format!("Error scanning for {:?}: {}", agent_type, e);
                    warn!("{}", error_msg);
                    result.add_error(error_msg);
                }
            }
        }

        info!(
            "Scan complete: found {} agent(s), {} error(s)",
            result.agents_count(),
            result.errors.len()
        );

        Ok(result)
    }

    pub async fn scan_all(&self) -> RimuruResult<ScanResult> {
        let options = ScanOptions::new().include_unconfigured(true);
        self.scan(&options).await
    }

    pub async fn scan_configured_only(&self) -> RimuruResult<ScanResult> {
        let options = ScanOptions::new().include_unconfigured(false);
        self.scan(&options).await
    }

    pub async fn scan_type(&self, agent_type: AgentType) -> RimuruResult<Option<DiscoveredAgent>> {
        let options = ScanOptions::new()
            .with_agent_types(vec![agent_type])
            .include_unconfigured(true);

        let result = self.scan(&options).await?;

        Ok(result.discovered.into_iter().next())
    }

    pub async fn is_any_installed(&self) -> bool {
        for discovery in self.discoveries.values() {
            if discovery.is_installed().await {
                return true;
            }
        }
        false
    }

    pub async fn get_installed_types(&self) -> Vec<AgentType> {
        let mut installed = Vec::new();

        for (agent_type, discovery) in &self.discoveries {
            if discovery.is_installed().await {
                installed.push(*agent_type);
            }
        }

        installed
    }
}

impl Default for AgentScanner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn test_scan_options_default() {
        let options = ScanOptions::default();
        assert!(!options.include_unconfigured);
        assert!(options.agent_types.is_none());
        assert!(!options.skip_executable_check);
    }

    #[test]
    fn test_scan_options_builder() {
        let options = ScanOptions::new()
            .include_unconfigured(true)
            .with_agent_types(vec![AgentType::ClaudeCode])
            .skip_executable_check(true);

        assert!(options.include_unconfigured);
        assert_eq!(options.agent_types, Some(vec![AgentType::ClaudeCode]));
        assert!(options.skip_executable_check);
    }

    #[test]
    fn test_discovered_agent_new() {
        let installation = AgentInstallation::new(
            AgentType::ClaudeCode,
            "claude-code",
            PathBuf::from("/home/user/.claude"),
        );

        let discovered = DiscoveredAgent::new(installation, DetectionMethod::ConfigDirectory);

        assert_eq!(
            discovered.detection_method,
            DetectionMethod::ConfigDirectory
        );
        assert!(!discovered.is_fully_detected());
    }

    #[test]
    fn test_discovered_agent_fully_detected() {
        let installation = AgentInstallation::new(
            AgentType::OpenCode,
            "opencode",
            PathBuf::from("/home/user/.opencode"),
        )
        .with_executable(PathBuf::from("/usr/bin/opencode"))
        .with_configured(true);

        let discovered = DiscoveredAgent::new(installation, DetectionMethod::Both);

        assert!(discovered.is_fully_detected());
    }

    #[test]
    fn test_scan_result_new() {
        let result = ScanResult::new();
        assert!(result.discovered.is_empty());
        assert!(result.errors.is_empty());
        assert!(result.scanned_types.is_empty());
    }

    #[test]
    fn test_scan_result_add_discovered() {
        let mut result = ScanResult::new();

        let installation = AgentInstallation::new(
            AgentType::ClaudeCode,
            "claude-code",
            PathBuf::from("/home/user/.claude"),
        );
        let discovered = DiscoveredAgent::new(installation, DetectionMethod::ConfigDirectory);

        result.add_discovered(discovered);

        assert_eq!(result.agents_count(), 1);
    }

    #[test]
    fn test_scan_result_add_error() {
        let mut result = ScanResult::new();
        result.add_error("Test error".to_string());

        assert!(result.has_errors());
        assert_eq!(result.errors.len(), 1);
    }

    #[test]
    fn test_scan_result_add_scanned_type() {
        let mut result = ScanResult::new();
        result.add_scanned_type(AgentType::ClaudeCode);
        result.add_scanned_type(AgentType::ClaudeCode);
        result.add_scanned_type(AgentType::OpenCode);

        assert_eq!(result.scanned_types.len(), 2);
    }

    #[test]
    fn test_scan_result_get_by_type() {
        let mut result = ScanResult::new();

        let claude_install = AgentInstallation::new(
            AgentType::ClaudeCode,
            "claude-code",
            PathBuf::from("/home/user/.claude"),
        );
        let opencode_install = AgentInstallation::new(
            AgentType::OpenCode,
            "opencode",
            PathBuf::from("/home/user/.opencode"),
        );

        result.add_discovered(DiscoveredAgent::new(
            claude_install,
            DetectionMethod::ConfigDirectory,
        ));
        result.add_discovered(DiscoveredAgent::new(
            opencode_install,
            DetectionMethod::ConfigDirectory,
        ));

        let claude_agents = result.get_by_type(AgentType::ClaudeCode);
        assert_eq!(claude_agents.len(), 1);

        let opencode_agents = result.get_by_type(AgentType::OpenCode);
        assert_eq!(opencode_agents.len(), 1);
    }

    #[test]
    fn test_scan_result_configured_only() {
        let mut result = ScanResult::new();

        let configured = AgentInstallation::new(
            AgentType::ClaudeCode,
            "claude-code",
            PathBuf::from("/home/user/.claude"),
        )
        .with_configured(true);

        let unconfigured = AgentInstallation::new(
            AgentType::OpenCode,
            "opencode",
            PathBuf::from("/home/user/.opencode"),
        )
        .with_configured(false);

        result.add_discovered(DiscoveredAgent::new(
            configured,
            DetectionMethod::ConfigDirectory,
        ));
        result.add_discovered(DiscoveredAgent::new(
            unconfigured,
            DetectionMethod::ConfigDirectory,
        ));

        let configured_agents = result.configured_only();
        assert_eq!(configured_agents.len(), 1);
    }

    #[test]
    fn test_scan_result_into_installations() {
        let mut result = ScanResult::new();

        let installation = AgentInstallation::new(
            AgentType::ClaudeCode,
            "claude-code",
            PathBuf::from("/home/user/.claude"),
        );
        result.add_discovered(DiscoveredAgent::new(
            installation,
            DetectionMethod::ConfigDirectory,
        ));

        let installations = result.into_installations();
        assert_eq!(installations.len(), 1);
        assert_eq!(installations[0].name, "claude-code");
    }

    #[test]
    fn test_agent_scanner_new() {
        let scanner = AgentScanner::new();
        let types = scanner.supported_types();

        assert!(types.contains(&AgentType::ClaudeCode));
        assert!(types.contains(&AgentType::Codex));
        assert!(types.contains(&AgentType::Copilot));
        assert!(types.contains(&AgentType::Cursor));
        assert!(types.contains(&AgentType::Goose));
        assert!(types.contains(&AgentType::OpenCode));
        assert_eq!(types.len(), 6);
    }

    #[tokio::test]
    async fn test_agent_scanner_scan_with_temp_dirs() {
        let claude_dir = tempdir().unwrap();
        let codex_dir = tempdir().unwrap();
        let copilot_dir = tempdir().unwrap();
        let cursor_dir = tempdir().unwrap();
        let opencode_dir = tempdir().unwrap();

        std::fs::write(claude_dir.path().join("settings.json"), "{}").unwrap();
        std::fs::write(codex_dir.path().join("config.json"), "{}").unwrap();
        std::fs::write(copilot_dir.path().join("hosts.json"), "{}").unwrap();
        let cursor_user_dir = cursor_dir.path().join("User");
        std::fs::create_dir_all(&cursor_user_dir).unwrap();
        std::fs::write(cursor_user_dir.join("settings.json"), "{}").unwrap();
        std::fs::write(opencode_dir.path().join("config.json"), "{}").unwrap();

        let scanner = AgentScanner::new()
            .with_discovery(
                ClaudeCodeDiscovery::new().with_config_dir(claude_dir.path().to_path_buf()),
            )
            .with_discovery(CodexDiscovery::new().with_config_dir(codex_dir.path().to_path_buf()))
            .with_discovery(
                CopilotDiscovery::new().with_config_dir(copilot_dir.path().to_path_buf()),
            )
            .with_discovery(CursorDiscovery::new().with_config_dir(cursor_dir.path().to_path_buf()))
            .with_discovery(
                OpenCodeDiscovery::new().with_config_dir(opencode_dir.path().to_path_buf()),
            );

        let options = ScanOptions::new().include_unconfigured(false);
        let result = scanner.scan(&options).await.unwrap();

        assert_eq!(result.agents_count(), 5);
        assert!(!result.has_errors());
    }

    #[tokio::test]
    async fn test_agent_scanner_scan_all() {
        let claude_dir = tempdir().unwrap();

        let scanner = AgentScanner::new()
            .with_discovery(
                ClaudeCodeDiscovery::new().with_config_dir(claude_dir.path().to_path_buf()),
            )
            .with_discovery(
                CodexDiscovery::new().with_config_dir(PathBuf::from("/nonexistent/.codex")),
            )
            .with_discovery(
                CopilotDiscovery::new()
                    .with_config_dir(PathBuf::from("/nonexistent/.config/github-copilot")),
            )
            .with_discovery(
                CursorDiscovery::new().with_config_dir(PathBuf::from("/nonexistent/.cursor")),
            )
            .with_discovery(
                GooseDiscovery::new().with_config_dir(PathBuf::from("/nonexistent/.config/goose")),
            )
            .with_discovery(
                OpenCodeDiscovery::new().with_config_dir(PathBuf::from("/nonexistent/.opencode")),
            );

        let result = scanner.scan_all().await.unwrap();

        assert!(result.agents_count() >= 1);
        assert_eq!(result.scanned_types.len(), 6);
    }

    #[tokio::test]
    async fn test_agent_scanner_scan_type() {
        let claude_dir = tempdir().unwrap();

        let scanner = AgentScanner::new().with_discovery(
            ClaudeCodeDiscovery::new().with_config_dir(claude_dir.path().to_path_buf()),
        );

        let result = scanner.scan_type(AgentType::ClaudeCode).await.unwrap();

        assert!(result.is_some());
        let discovered = result.unwrap();
        assert_eq!(discovered.installation.agent_type, AgentType::ClaudeCode);
    }

    #[tokio::test]
    async fn test_agent_scanner_scan_type_not_found() {
        let scanner = AgentScanner::new().with_discovery(
            ClaudeCodeDiscovery::new().with_config_dir(PathBuf::from("/nonexistent/.claude")),
        );

        let result = scanner.scan_type(AgentType::ClaudeCode).await.unwrap();

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_agent_scanner_is_any_installed() {
        let claude_dir = tempdir().unwrap();

        let scanner = AgentScanner::new().with_discovery(
            ClaudeCodeDiscovery::new().with_config_dir(claude_dir.path().to_path_buf()),
        );

        assert!(scanner.is_any_installed().await);
    }

    #[tokio::test]
    async fn test_agent_scanner_get_installed_types() {
        let claude_dir = tempdir().unwrap();
        let opencode_dir = tempdir().unwrap();

        let scanner = AgentScanner::new()
            .with_discovery(
                ClaudeCodeDiscovery::new().with_config_dir(claude_dir.path().to_path_buf()),
            )
            .with_discovery(
                OpenCodeDiscovery::new().with_config_dir(opencode_dir.path().to_path_buf()),
            );

        let installed = scanner.get_installed_types().await;

        assert!(installed.contains(&AgentType::ClaudeCode));
        assert!(installed.contains(&AgentType::OpenCode));
    }

    #[tokio::test]
    async fn test_agent_scanner_scan_configured_only() {
        let claude_dir = tempdir().unwrap();
        let codex_dir = tempdir().unwrap();
        let copilot_dir = tempdir().unwrap();
        let cursor_dir = tempdir().unwrap();
        let goose_dir = tempdir().unwrap();
        let opencode_dir = tempdir().unwrap();

        std::fs::write(claude_dir.path().join("settings.json"), "{}").unwrap();

        let scanner = AgentScanner::new()
            .with_discovery(
                ClaudeCodeDiscovery::new().with_config_dir(claude_dir.path().to_path_buf()),
            )
            .with_discovery(CodexDiscovery::new().with_config_dir(codex_dir.path().to_path_buf()))
            .with_discovery(
                CopilotDiscovery::new().with_config_dir(copilot_dir.path().to_path_buf()),
            )
            .with_discovery(CursorDiscovery::new().with_config_dir(cursor_dir.path().to_path_buf()))
            .with_discovery(GooseDiscovery::new().with_config_dir(goose_dir.path().to_path_buf()))
            .with_discovery(
                OpenCodeDiscovery::new().with_config_dir(opencode_dir.path().to_path_buf()),
            );

        let result = scanner.scan_configured_only().await.unwrap();

        assert_eq!(result.agents_count(), 1);
        assert_eq!(
            result.discovered[0].installation.agent_type,
            AgentType::ClaudeCode
        );
    }

    #[tokio::test]
    async fn test_agent_scanner_filter_by_agent_type() {
        let claude_dir = tempdir().unwrap();
        let opencode_dir = tempdir().unwrap();

        let scanner = AgentScanner::new()
            .with_discovery(
                ClaudeCodeDiscovery::new().with_config_dir(claude_dir.path().to_path_buf()),
            )
            .with_discovery(
                OpenCodeDiscovery::new().with_config_dir(opencode_dir.path().to_path_buf()),
            );

        let options = ScanOptions::new()
            .with_agent_types(vec![AgentType::OpenCode])
            .include_unconfigured(true);

        let result = scanner.scan(&options).await.unwrap();

        assert_eq!(result.scanned_types, vec![AgentType::OpenCode]);
        assert!(result
            .discovered
            .iter()
            .all(|a| a.installation.agent_type == AgentType::OpenCode));
    }

    #[test]
    fn test_detection_method_equality() {
        assert_eq!(
            DetectionMethod::ConfigDirectory,
            DetectionMethod::ConfigDirectory
        );
        assert_eq!(DetectionMethod::Executable, DetectionMethod::Executable);
        assert_eq!(DetectionMethod::Both, DetectionMethod::Both);
        assert_ne!(
            DetectionMethod::ConfigDirectory,
            DetectionMethod::Executable
        );
    }
}
