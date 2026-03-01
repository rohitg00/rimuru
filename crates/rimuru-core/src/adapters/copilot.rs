use std::path::PathBuf;

use async_trait::async_trait;
use chrono::Utc;
use serde_json::Value;
use tracing::{debug, warn};
use uuid::Uuid;

use super::{AgentAdapter, CostTracker, SessionMonitor};
use crate::error::RimuruError;
use crate::models::{Agent, AgentStatus, AgentType, Session, SessionStatus};

type Result<T> = std::result::Result<T, RimuruError>;

pub struct CopilotAdapter {
    config_path: PathBuf,
    vscode_path: PathBuf,
    connected: bool,
    agent_id: Uuid,
}

impl CopilotAdapter {
    pub fn new() -> Self {
        let (config_path, vscode_path) = Self::default_paths();
        Self {
            config_path,
            vscode_path,
            connected: false,
            agent_id: Uuid::new_v4(),
        }
    }

    fn default_paths() -> (PathBuf, PathBuf) {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));

        #[cfg(target_os = "macos")]
        let vscode_path = home.join("Library/Application Support/Code");
        #[cfg(target_os = "linux")]
        let vscode_path = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("Code");
        #[cfg(target_os = "windows")]
        let vscode_path = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("C:\\temp"))
            .join("Code");
        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        let vscode_path = home.join(".vscode");

        let config_path = home.join(".config/github-copilot");

        (config_path, vscode_path)
    }

    fn extensions_dir(&self) -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join(".vscode/extensions")
    }

    fn copilot_extension_dir(&self) -> Option<PathBuf> {
        let ext_dir = self.extensions_dir();
        if !ext_dir.exists() {
            return None;
        }
        if let Ok(entries) = std::fs::read_dir(&ext_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("github.copilot-") && !name.contains("chat") {
                    return Some(entry.path());
                }
            }
        }
        None
    }

    fn copilot_chat_extension_dir(&self) -> Option<PathBuf> {
        let ext_dir = self.extensions_dir();
        if !ext_dir.exists() {
            return None;
        }
        if let Ok(entries) = std::fs::read_dir(&ext_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("github.copilot-chat-") {
                    return Some(entry.path());
                }
            }
        }
        None
    }

    fn storage_dir(&self) -> PathBuf {
        self.vscode_path
            .join("User/globalStorage/github.copilot-chat")
    }

    fn read_chat_history(&self) -> Result<Vec<Value>> {
        let storage = self.storage_dir();
        if !storage.exists() {
            return Ok(vec![]);
        }

        let mut conversations = Vec::new();

        let history_dir = storage.join("history");
        if history_dir.exists() {
            for entry in std::fs::read_dir(&history_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("json") {
                    match std::fs::read_to_string(&path) {
                        Ok(content) => {
                            if let Ok(val) = serde_json::from_str::<Value>(&content) {
                                conversations.push(val);
                            }
                        }
                        Err(e) => warn!("Failed to read Copilot chat file {}: {}", path.display(), e),
                    }
                }
            }
        }

        Ok(conversations)
    }

    fn detect_version(&self) -> Option<String> {
        let ext = self.copilot_extension_dir()?;
        let package_json = ext.join("package.json");
        if package_json.exists() {
            let content = std::fs::read_to_string(&package_json).ok()?;
            let pkg: Value = serde_json::from_str(&content).ok()?;
            pkg.get("version")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        } else {
            None
        }
    }

    fn estimate_cost(model: &str, input_tokens: u64, output_tokens: u64) -> f64 {
        let (input_rate, output_rate) = match model {
            m if m.contains("gpt-4o") => (2.5, 10.0),
            m if m.contains("gpt-4") => (30.0, 60.0),
            m if m.contains("gpt-3.5") => (0.5, 1.5),
            m if m.contains("claude") => (3.0, 15.0),
            _ => (2.5, 10.0),
        };
        let input_cost = (input_tokens as f64 / 1_000_000.0) * input_rate;
        let output_cost = (output_tokens as f64 / 1_000_000.0) * output_rate;
        input_cost + output_cost
    }
}

impl Default for CopilotAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AgentAdapter for CopilotAdapter {
    fn agent_type(&self) -> AgentType {
        AgentType::Copilot
    }

    fn is_installed(&self) -> bool {
        self.copilot_extension_dir().is_some() || self.config_path.exists()
    }

    fn detect_version(&self) -> Option<String> {
        let ext = self.copilot_extension_dir()?;
        let package_json = ext.join("package.json");
        if package_json.exists() {
            let content = std::fs::read_to_string(&package_json).ok()?;
            let pkg: serde_json::Value = serde_json::from_str(&content).ok()?;
            pkg.get("version")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        } else {
            None
        }
    }

    async fn connect(&mut self) -> Result<()> {
        if !self.is_installed() {
            return Err(RimuruError::Adapter(
                "GitHub Copilot is not installed (no VS Code extension found)".into(),
            ));
        }
        self.connected = true;
        debug!("Connected to GitHub Copilot adapter");
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.connected = false;
        debug!("Disconnected from GitHub Copilot adapter");
        Ok(())
    }

    async fn get_status(&self) -> Result<Value> {
        let has_extension = self.copilot_extension_dir().is_some();
        let has_chat = self.copilot_chat_extension_dir().is_some();
        let conversations = self.read_chat_history().unwrap_or_default();

        Ok(serde_json::json!({
            "agent_type": "copilot",
            "installed": self.is_installed(),
            "connected": self.connected,
            "config_path": self.config_path.to_string_lossy(),
            "vscode_path": self.vscode_path.to_string_lossy(),
            "has_copilot_extension": has_extension,
            "has_chat_extension": has_chat,
            "chat_conversations": conversations.len(),
            "version": self.detect_version(),
        }))
    }

    async fn get_info(&self) -> Result<Agent> {
        let mut agent = Agent::new(AgentType::Copilot, "GitHub Copilot".into());
        agent.id = self.agent_id;
        agent.config_path = Some(self.config_path.to_string_lossy().to_string());
        agent.version = self.detect_version();
        agent.status = if self.connected {
            AgentStatus::Connected
        } else {
            AgentStatus::Disconnected
        };
        agent.last_seen = Some(Utc::now());

        let conversations = self.read_chat_history().unwrap_or_default();
        agent.session_count = conversations.len() as u64;

        agent.metadata = serde_json::json!({
            "has_copilot_extension": self.copilot_extension_dir().is_some(),
            "has_chat_extension": self.copilot_chat_extension_dir().is_some(),
            "storage_dir": self.storage_dir().to_string_lossy(),
        });

        Ok(agent)
    }

    async fn get_sessions(&self) -> Result<Vec<Session>> {
        let conversations = self.read_chat_history()?;
        let mut sessions = Vec::new();

        for conv in &conversations {
            let mut session = Session::new(self.agent_id, AgentType::Copilot);

            if let Some(turns) = conv.get("turns").and_then(|t| t.as_array()) {
                session.messages = turns.len() as u64;
            }

            if let Some(model) = conv
                .get("model")
                .or_else(|| conv.get("modelFamily"))
                .and_then(|m| m.as_str())
            {
                session.model = Some(model.to_string());
            }

            session.status = SessionStatus::Completed;
            session.metadata = serde_json::json!({
                "source": "copilot_chat_history",
            });

            sessions.push(session);
        }

        sessions.sort_by(|a, b| b.started_at.cmp(&a.started_at));
        Ok(sessions)
    }

    async fn health_check(&self) -> Result<bool> {
        Ok(self.is_installed() && self.connected)
    }
}

#[async_trait]
impl CostTracker for CopilotAdapter {
    async fn get_usage(&self) -> Result<Value> {
        let sessions = self.get_sessions().await?;
        Ok(serde_json::json!({
            "agent_type": "copilot",
            "total_sessions": sessions.len(),
            "note": "GitHub Copilot uses subscription pricing ($10-39/mo); per-token costs are not directly available",
        }))
    }

    async fn calculate_cost(&self, model: &str, input_tokens: u64, output_tokens: u64) -> Result<f64> {
        Ok(Self::estimate_cost(model, input_tokens, output_tokens))
    }

    fn get_supported_models(&self) -> Vec<String> {
        vec![
            "gpt-4o".into(),
            "gpt-4".into(),
            "gpt-3.5-turbo".into(),
            "claude-3-5-sonnet".into(),
        ]
    }

    async fn get_total_cost(&self) -> Result<f64> {
        let sessions = self.get_sessions().await?;
        Ok(sessions.iter().map(|s| s.total_cost).sum())
    }
}

#[async_trait]
impl SessionMonitor for CopilotAdapter {
    async fn get_session_history(&self) -> Result<Vec<Session>> {
        self.get_sessions().await
    }

    async fn get_session_details(&self, session_id: &str) -> Result<Option<Session>> {
        let sessions = self.get_sessions().await?;
        let target = Uuid::parse_str(session_id)
            .map_err(|e| RimuruError::Validation(format!("Invalid session ID: {}", e)))?;
        Ok(sessions.into_iter().find(|s| s.id == target))
    }

    async fn get_active_sessions(&self) -> Result<Vec<Session>> {
        let sessions = self.get_sessions().await?;
        Ok(sessions
            .into_iter()
            .filter(|s| matches!(s.status, SessionStatus::Active))
            .collect())
    }
}
