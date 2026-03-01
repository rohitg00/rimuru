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

pub struct CursorAdapter {
    config_path: PathBuf,
    connected: bool,
    agent_id: Uuid,
}

impl CursorAdapter {
    pub fn new() -> Self {
        let config_path = Self::default_config_path();
        Self {
            config_path,
            connected: false,
            agent_id: Uuid::new_v4(),
        }
    }

    fn default_config_path() -> PathBuf {
        #[cfg(target_os = "macos")]
        {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("/tmp"))
                .join("Library/Application Support/Cursor")
        }
        #[cfg(target_os = "linux")]
        {
            dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("/tmp"))
                .join("Cursor")
        }
        #[cfg(target_os = "windows")]
        {
            dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("C:\\temp"))
                .join("Cursor")
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("/tmp"))
                .join(".cursor")
        }
    }

    fn user_dir(&self) -> PathBuf {
        self.config_path.join("User")
    }

    fn settings_path(&self) -> PathBuf {
        self.user_dir().join("settings.json")
    }

    fn storage_path(&self) -> PathBuf {
        self.user_dir().join("globalStorage")
    }

    fn cursor_state_path(&self) -> PathBuf {
        self.storage_path().join("state.vscdb")
    }

    fn read_settings(&self) -> Result<Value> {
        let path = self.settings_path();
        if !path.exists() {
            return Ok(serde_json::json!({}));
        }
        let content = std::fs::read_to_string(&path)?;
        let val: Value = serde_json::from_str(&content)?;
        Ok(val)
    }

    fn detect_version(&self) -> Option<String> {
        let output = std::process::Command::new("cursor")
            .arg("--version")
            .output()
            .ok()?;
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            text.lines().next().map(|l| l.trim().to_string())
        } else {
            None
        }
    }

    fn scan_workspaces(&self) -> Result<Vec<PathBuf>> {
        let storage = self.storage_path();
        if !storage.exists() {
            return Ok(vec![]);
        }
        let mut workspaces = Vec::new();
        for entry in std::fs::read_dir(&storage)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let workspace_json = path.join("workspace.json");
                if workspace_json.exists() {
                    workspaces.push(path);
                }
            }
        }
        Ok(workspaces)
    }

    fn parse_workspace_session(&self, workspace_dir: &PathBuf) -> Result<Session> {
        let workspace_json = workspace_dir.join("workspace.json");
        let mut session = Session::new(self.agent_id, AgentType::Cursor);

        if workspace_json.exists() {
            let content = std::fs::read_to_string(&workspace_json)?;
            let data: Value = serde_json::from_str(&content)?;

            if let Some(folder) = data.get("folder").and_then(|f| f.as_str()) {
                session.project_path = Some(folder.to_string());
            }
        }

        if let Ok(metadata) = std::fs::metadata(&workspace_json) {
            if let Ok(modified) = metadata.modified() {
                let elapsed = modified.elapsed().unwrap_or_default();
                if elapsed.as_secs() > 3600 {
                    session.status = SessionStatus::Completed;
                    session.ended_at = Some(chrono::DateTime::<Utc>::from(modified));
                }
            }
        }

        session.model = Some("cursor-default".into());
        session.metadata = serde_json::json!({
            "workspace_dir": workspace_dir.to_string_lossy(),
        });

        Ok(session)
    }

    fn estimate_cost(model: &str, input_tokens: u64, output_tokens: u64) -> f64 {
        let (input_rate, output_rate) = match model {
            m if m.contains("gpt-4") || m.contains("gpt4") => (30.0, 60.0),
            m if m.contains("gpt-3.5") => (0.5, 1.5),
            m if m.contains("claude") && m.contains("opus") => (15.0, 75.0),
            m if m.contains("claude") && m.contains("sonnet") => (3.0, 15.0),
            m if m.contains("cursor-small") => (0.5, 1.5),
            _ => (3.0, 15.0),
        };
        let input_cost = (input_tokens as f64 / 1_000_000.0) * input_rate;
        let output_cost = (output_tokens as f64 / 1_000_000.0) * output_rate;
        input_cost + output_cost
    }
}

impl Default for CursorAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AgentAdapter for CursorAdapter {
    fn agent_type(&self) -> AgentType {
        AgentType::Cursor
    }

    fn is_installed(&self) -> bool {
        self.config_path.exists()
    }

    fn detect_version(&self) -> Option<String> {
        let output = std::process::Command::new("cursor")
            .arg("--version")
            .output()
            .ok()?;
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            text.lines().next().map(|l| l.trim().to_string())
        } else {
            None
        }
    }

    async fn connect(&mut self) -> Result<()> {
        if !self.is_installed() {
            return Err(RimuruError::Adapter(
                "Cursor is not installed (config directory not found)".into(),
            ));
        }
        self.connected = true;
        debug!("Connected to Cursor adapter");
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.connected = false;
        debug!("Disconnected from Cursor adapter");
        Ok(())
    }

    async fn get_status(&self) -> Result<Value> {
        let workspaces = self.scan_workspaces().unwrap_or_default();
        let has_state_db = self.cursor_state_path().exists();

        Ok(serde_json::json!({
            "agent_type": "cursor",
            "installed": self.is_installed(),
            "connected": self.connected,
            "config_path": self.config_path.to_string_lossy(),
            "workspace_count": workspaces.len(),
            "has_state_db": has_state_db,
            "version": self.detect_version(),
        }))
    }

    async fn get_info(&self) -> Result<Agent> {
        let mut agent = Agent::new(AgentType::Cursor, "Cursor".into());
        agent.id = self.agent_id;
        agent.config_path = Some(self.config_path.to_string_lossy().to_string());
        agent.version = self.detect_version();
        agent.status = if self.connected {
            AgentStatus::Connected
        } else {
            AgentStatus::Disconnected
        };
        agent.last_seen = Some(Utc::now());

        if let Ok(workspaces) = self.scan_workspaces() {
            agent.session_count = workspaces.len() as u64;
        }

        let settings = self.read_settings().unwrap_or_else(|_| serde_json::json!({}));
        agent.metadata = serde_json::json!({
            "settings_keys": settings.as_object().map(|o| o.keys().cloned().collect::<Vec<_>>()).unwrap_or_default(),
            "storage_path": self.storage_path().to_string_lossy(),
        });

        Ok(agent)
    }

    async fn get_sessions(&self) -> Result<Vec<Session>> {
        let workspaces = self.scan_workspaces()?;
        let mut sessions = Vec::new();
        for ws in &workspaces {
            match self.parse_workspace_session(ws) {
                Ok(s) => sessions.push(s),
                Err(e) => warn!("Failed to parse Cursor workspace at {}: {}", ws.display(), e),
            }
        }
        sessions.sort_by(|a, b| b.started_at.cmp(&a.started_at));
        Ok(sessions)
    }

    async fn health_check(&self) -> Result<bool> {
        Ok(self.is_installed() && self.connected)
    }
}

#[async_trait]
impl CostTracker for CursorAdapter {
    async fn get_usage(&self) -> Result<Value> {
        let sessions = self.get_sessions().await?;
        let total_cost: f64 = sessions.iter().map(|s| s.total_cost).sum();

        Ok(serde_json::json!({
            "agent_type": "cursor",
            "total_sessions": sessions.len(),
            "estimated_total_cost": total_cost,
            "note": "Cursor uses subscription pricing; per-token cost is estimated",
        }))
    }

    async fn calculate_cost(&self, model: &str, input_tokens: u64, output_tokens: u64) -> Result<f64> {
        Ok(Self::estimate_cost(model, input_tokens, output_tokens))
    }

    fn get_supported_models(&self) -> Vec<String> {
        vec![
            "gpt-4".into(),
            "gpt-4-turbo".into(),
            "gpt-3.5-turbo".into(),
            "claude-3-5-sonnet".into(),
            "cursor-small".into(),
        ]
    }

    async fn get_total_cost(&self) -> Result<f64> {
        let sessions = self.get_sessions().await?;
        Ok(sessions.iter().map(|s| s.total_cost).sum())
    }
}

#[async_trait]
impl SessionMonitor for CursorAdapter {
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
