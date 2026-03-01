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

pub struct OpenCodeAdapter {
    config_path: PathBuf,
    connected: bool,
    agent_id: Uuid,
}

impl OpenCodeAdapter {
    pub fn new() -> Self {
        let config_path = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join(".opencode");

        Self {
            config_path,
            connected: false,
            agent_id: Uuid::new_v4(),
        }
    }

    fn config_file(&self) -> PathBuf {
        self.config_path.join("config.json")
    }

    fn sessions_dir(&self) -> PathBuf {
        self.config_path.join("sessions")
    }

    fn state_dir(&self) -> PathBuf {
        self.config_path.join("state")
    }

    fn history_dir(&self) -> PathBuf {
        self.config_path.join("history")
    }

    fn read_config(&self) -> Result<Value> {
        let path = self.config_file();
        if !path.exists() {
            let toml_path = self.config_path.join("config.toml");
            if toml_path.exists() {
                let content = std::fs::read_to_string(&toml_path)?;
                return Ok(serde_json::json!({
                    "format": "toml",
                    "raw": content,
                }));
            }
            return Ok(serde_json::json!({}));
        }
        let content = std::fs::read_to_string(&path)?;
        let val: Value = serde_json::from_str(&content)?;
        Ok(val)
    }

    fn detect_version(&self) -> Option<String> {
        let output = std::process::Command::new("opencode")
            .arg("--version")
            .output()
            .ok()?;
        if output.status.success() {
            Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            None
        }
    }

    fn scan_session_files(&self) -> Result<Vec<PathBuf>> {
        let mut files = Vec::new();

        for dir in [self.sessions_dir(), self.history_dir()] {
            if !dir.exists() {
                continue;
            }
            for entry in std::fs::read_dir(&dir)? {
                let entry = entry?;
                let path = entry.path();
                let ext = path.extension().and_then(|e| e.to_str());
                if ext == Some("json") || ext == Some("jsonl") {
                    files.push(path);
                }
            }
        }

        Ok(files)
    }

    fn parse_session_file(&self, path: &PathBuf) -> Result<Session> {
        let content = std::fs::read_to_string(path)?;
        let mut session = Session::new(self.agent_id, AgentType::OpenCode);

        let ext = path.extension().and_then(|e| e.to_str());

        if ext == Some("json") {
            let data: Value = serde_json::from_str(&content)?;

            if let Some(model) = data.get("model").and_then(|m| m.as_str()) {
                session.model = Some(model.to_string());
            }

            if let Some(project) = data
                .get("project_path")
                .or_else(|| data.get("cwd"))
                .and_then(|p| p.as_str())
            {
                session.project_path = Some(project.to_string());
            }

            if let Some(messages) = data.get("messages").and_then(|m| m.as_array()) {
                session.messages = messages.len() as u64;

                let mut total_input: u64 = 0;
                let mut total_output: u64 = 0;

                for msg in messages {
                    if let Some(usage) = msg.get("usage") {
                        if let Some(inp) = usage.get("input_tokens").and_then(|v| v.as_u64()) {
                            total_input += inp;
                        }
                        if let Some(out) = usage.get("output_tokens").and_then(|v| v.as_u64()) {
                            total_output += out;
                        }
                    }
                }

                session.input_tokens = total_input;
                session.output_tokens = total_output;
                session.total_tokens = total_input + total_output;
            }

            if let Some(total) = data.get("total_tokens").and_then(|t| t.as_u64()) {
                session.total_tokens = total;
            }
            if let Some(inp) = data.get("input_tokens").and_then(|t| t.as_u64()) {
                session.input_tokens = inp;
            }
            if let Some(out) = data.get("output_tokens").and_then(|t| t.as_u64()) {
                session.output_tokens = out;
            }
        } else {
            let mut msg_count: u64 = 0;
            let mut total_input: u64 = 0;
            let mut total_output: u64 = 0;
            let mut last_model: Option<String> = None;

            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                let entry: Value = match serde_json::from_str(line) {
                    Ok(v) => v,
                    Err(_) => continue,
                };

                msg_count += 1;

                if let Some(usage) = entry.get("usage") {
                    if let Some(inp) = usage.get("input_tokens").and_then(|v| v.as_u64()) {
                        total_input += inp;
                    }
                    if let Some(out) = usage.get("output_tokens").and_then(|v| v.as_u64()) {
                        total_output += out;
                    }
                }

                if let Some(model) = entry.get("model").and_then(|m| m.as_str()) {
                    last_model = Some(model.to_string());
                }
            }

            session.messages = msg_count;
            session.input_tokens = total_input;
            session.output_tokens = total_output;
            session.total_tokens = total_input + total_output;
            session.model = last_model;
        }

        if let Some(ref model) = session.model {
            session.total_cost =
                Self::estimate_cost(model, session.input_tokens, session.output_tokens);
        }

        if let Ok(metadata) = std::fs::metadata(path) {
            if let Ok(modified) = metadata.modified() {
                let elapsed = modified.elapsed().unwrap_or_default();
                if elapsed.as_secs() > 3600 {
                    session.status = SessionStatus::Completed;
                    session.ended_at = Some(chrono::DateTime::<Utc>::from(modified));
                }
            }
        }

        session.metadata = serde_json::json!({
            "source_file": path.to_string_lossy(),
        });

        Ok(session)
    }

    fn estimate_cost(model: &str, input_tokens: u64, output_tokens: u64) -> f64 {
        let (input_rate, output_rate) = match model {
            m if m.contains("claude") && m.contains("opus") => (15.0, 75.0),
            m if m.contains("claude") && m.contains("sonnet") => (3.0, 15.0),
            m if m.contains("claude") && m.contains("haiku") => (0.25, 1.25),
            m if m.contains("gpt-4o") => (2.5, 10.0),
            m if m.contains("gpt-4") => (30.0, 60.0),
            m if m.contains("gemini") => (1.25, 5.0),
            m if m.contains("deepseek") => (0.27, 1.1),
            _ => (3.0, 15.0),
        };
        let input_cost = (input_tokens as f64 / 1_000_000.0) * input_rate;
        let output_cost = (output_tokens as f64 / 1_000_000.0) * output_rate;
        input_cost + output_cost
    }
}

impl Default for OpenCodeAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AgentAdapter for OpenCodeAdapter {
    fn agent_type(&self) -> AgentType {
        AgentType::OpenCode
    }

    fn is_installed(&self) -> bool {
        self.config_path.exists()
    }

    fn detect_version(&self) -> Option<String> {
        let output = std::process::Command::new("opencode")
            .arg("--version")
            .output()
            .ok()?;
        if output.status.success() {
            Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            None
        }
    }

    async fn connect(&mut self) -> Result<()> {
        if !self.is_installed() {
            return Err(RimuruError::Adapter(
                "OpenCode is not installed (~/.opencode not found)".into(),
            ));
        }
        self.connected = true;
        debug!("Connected to OpenCode adapter");
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.connected = false;
        debug!("Disconnected from OpenCode adapter");
        Ok(())
    }

    async fn get_status(&self) -> Result<Value> {
        let session_files = self.scan_session_files().unwrap_or_default();
        let has_state = self.state_dir().exists();

        Ok(serde_json::json!({
            "agent_type": "opencode",
            "installed": self.is_installed(),
            "connected": self.connected,
            "config_path": self.config_path.to_string_lossy(),
            "session_files": session_files.len(),
            "has_state_dir": has_state,
            "has_config": self.config_file().exists(),
            "version": self.detect_version(),
        }))
    }

    async fn get_info(&self) -> Result<Agent> {
        let mut agent = Agent::new(AgentType::OpenCode, "OpenCode".into());
        agent.id = self.agent_id;
        agent.config_path = Some(self.config_path.to_string_lossy().to_string());
        agent.version = self.detect_version();
        agent.status = if self.connected {
            AgentStatus::Connected
        } else {
            AgentStatus::Disconnected
        };
        agent.last_seen = Some(Utc::now());

        let session_files = self.scan_session_files().unwrap_or_default();
        agent.session_count = session_files.len() as u64;

        let config = self.read_config().unwrap_or_else(|_| serde_json::json!({}));
        agent.metadata = serde_json::json!({
            "config": config,
            "sessions_dir": self.sessions_dir().to_string_lossy(),
            "has_state": self.state_dir().exists(),
        });

        Ok(agent)
    }

    async fn get_sessions(&self) -> Result<Vec<Session>> {
        let files = self.scan_session_files()?;
        let mut sessions = Vec::new();
        for file in &files {
            match self.parse_session_file(file) {
                Ok(s) => sessions.push(s),
                Err(e) => warn!("Failed to parse OpenCode session {}: {}", file.display(), e),
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
impl CostTracker for OpenCodeAdapter {
    async fn get_usage(&self) -> Result<Value> {
        let sessions = self.get_sessions().await?;
        let total_input: u64 = sessions.iter().map(|s| s.input_tokens).sum();
        let total_output: u64 = sessions.iter().map(|s| s.output_tokens).sum();
        let total_cost: f64 = sessions.iter().map(|s| s.total_cost).sum();

        Ok(serde_json::json!({
            "agent_type": "opencode",
            "total_sessions": sessions.len(),
            "total_input_tokens": total_input,
            "total_output_tokens": total_output,
            "total_tokens": total_input + total_output,
            "estimated_total_cost": total_cost,
        }))
    }

    async fn calculate_cost(&self, model: &str, input_tokens: u64, output_tokens: u64) -> Result<f64> {
        Ok(Self::estimate_cost(model, input_tokens, output_tokens))
    }

    fn get_supported_models(&self) -> Vec<String> {
        vec![
            "claude-3-5-sonnet".into(),
            "claude-3-opus".into(),
            "gpt-4o".into(),
            "deepseek-coder".into(),
            "gemini-2.0-flash".into(),
        ]
    }

    async fn get_total_cost(&self) -> Result<f64> {
        let sessions = self.get_sessions().await?;
        Ok(sessions.iter().map(|s| s.total_cost).sum())
    }
}

#[async_trait]
impl SessionMonitor for OpenCodeAdapter {
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
