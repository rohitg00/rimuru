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

pub struct CodexAdapter {
    config_path: PathBuf,
    connected: bool,
    agent_id: Uuid,
}

impl CodexAdapter {
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        let config_path = if home.join(".codex").exists() {
            home.join(".codex")
        } else {
            home.join(".config/codex")
        };

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

    fn history_file(&self) -> PathBuf {
        self.config_path.join("history.jsonl")
    }

    fn read_config(&self) -> Result<Value> {
        let path = self.config_file();
        if !path.exists() {
            return Ok(serde_json::json!({}));
        }
        let content = std::fs::read_to_string(&path)?;
        let val: Value = serde_json::from_str(&content)?;
        Ok(val)
    }

    fn detect_version(&self) -> Option<String> {
        let output = std::process::Command::new("codex")
            .arg("--version")
            .output()
            .ok()?;
        if output.status.success() {
            Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            None
        }
    }

    fn scan_sessions(&self) -> Result<Vec<PathBuf>> {
        let sessions_dir = self.sessions_dir();
        if !sessions_dir.exists() {
            return Ok(vec![]);
        }
        let mut files = Vec::new();
        for entry in std::fs::read_dir(&sessions_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json")
                || path.extension().and_then(|e| e.to_str()) == Some("jsonl")
            {
                files.push(path);
            }
        }
        Ok(files)
    }

    fn parse_session_file(&self, path: &PathBuf) -> Result<Session> {
        let content = std::fs::read_to_string(path)?;
        let mut session = Session::new(self.agent_id, AgentType::Codex);

        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            let data: Value = serde_json::from_str(&content)?;

            if let Some(sess) = data.get("session") {
                if let Some(id) = sess.get("id").and_then(|v| v.as_str()) {
                    if let Ok(parsed) = uuid::Uuid::parse_str(id) {
                        session.id = parsed;
                    }
                }
                if let Some(ts) = sess.get("timestamp").and_then(|v| v.as_str()) {
                    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(ts) {
                        session.started_at = dt.with_timezone(&Utc);
                    }
                }
            }

            if let Some(items) = data.get("items").and_then(|v| v.as_array()) {
                session.messages = items.len() as u64;
                for item in items {
                    if let Some(model) = item.get("model").and_then(|m| m.as_str()) {
                        session.model = Some(model.to_string());
                    }
                    if let Some(usage) = item.get("usage") {
                        if let Some(inp) = usage.get("input_tokens").and_then(|v| v.as_u64()) {
                            session.input_tokens += inp;
                        }
                        if let Some(out) = usage.get("output_tokens").and_then(|v| v.as_u64()) {
                            session.output_tokens += out;
                        }
                    }
                }
                session.total_tokens = session.input_tokens + session.output_tokens;
            }

            if let Some(model) = data.get("model").and_then(|m| m.as_str()) {
                session.model = Some(model.to_string());
            }
            if let Some(project) = data.get("project_path").and_then(|p| p.as_str()) {
                session.project_path = Some(project.to_string());
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

    fn parse_history(&self) -> Result<Vec<Session>> {
        let history = self.history_file();
        if !history.exists() {
            return Ok(vec![]);
        }
        let content = std::fs::read_to_string(&history)?;
        let mut sessions = Vec::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let entry: Value = match serde_json::from_str(line) {
                Ok(v) => v,
                Err(_) => continue,
            };

            let mut session = Session::new(self.agent_id, AgentType::Codex);

            if let Some(model) = entry.get("model").and_then(|m| m.as_str()) {
                session.model = Some(model.to_string());
            }
            if let Some(inp) = entry.get("input_tokens").and_then(|t| t.as_u64()) {
                session.input_tokens = inp;
            }
            if let Some(out) = entry.get("output_tokens").and_then(|t| t.as_u64()) {
                session.output_tokens = out;
            }
            session.total_tokens = session.input_tokens + session.output_tokens;
            session.status = SessionStatus::Completed;

            if let Some(ref model) = session.model {
                session.total_cost =
                    Self::estimate_cost(model, session.input_tokens, session.output_tokens);
            }

            sessions.push(session);
        }

        Ok(sessions)
    }

    fn estimate_cost(model: &str, input_tokens: u64, output_tokens: u64) -> f64 {
        let (input_rate, output_rate) = match model {
            m if m.contains("o4-mini") => (1.1, 4.4),
            m if m.contains("o3") => (10.0, 40.0),
            m if m.contains("gpt-4o") => (2.5, 10.0),
            m if m.contains("gpt-4") => (30.0, 60.0),
            _ => (2.5, 10.0),
        };
        let input_cost = (input_tokens as f64 / 1_000_000.0) * input_rate;
        let output_cost = (output_tokens as f64 / 1_000_000.0) * output_rate;
        input_cost + output_cost
    }
}

impl Default for CodexAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AgentAdapter for CodexAdapter {
    fn agent_type(&self) -> AgentType {
        AgentType::Codex
    }

    fn is_installed(&self) -> bool {
        self.config_path.exists()
    }

    fn detect_version(&self) -> Option<String> {
        let output = std::process::Command::new("codex")
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
                "Codex is not installed (~/.config/codex not found)".into(),
            ));
        }
        self.connected = true;
        debug!("Connected to Codex adapter");
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.connected = false;
        debug!("Disconnected from Codex adapter");
        Ok(())
    }

    async fn get_status(&self) -> Result<Value> {
        let config = self.read_config().unwrap_or_else(|_| serde_json::json!({}));
        let session_files = self.scan_sessions().unwrap_or_default();

        Ok(serde_json::json!({
            "agent_type": "codex",
            "installed": self.is_installed(),
            "connected": self.connected,
            "config_path": self.config_path.to_string_lossy(),
            "session_files": session_files.len(),
            "has_config": config != serde_json::json!({}),
            "has_history": self.history_file().exists(),
            "version": self.detect_version(),
        }))
    }

    async fn get_info(&self) -> Result<Agent> {
        let mut agent = Agent::new(AgentType::Codex, "Codex".into());
        agent.id = self.agent_id;
        agent.config_path = Some(self.config_path.to_string_lossy().to_string());
        agent.version = self.detect_version();
        agent.status = if self.connected {
            AgentStatus::Connected
        } else {
            AgentStatus::Disconnected
        };
        agent.last_seen = Some(Utc::now());

        let session_files = self.scan_sessions().unwrap_or_default();
        agent.session_count = session_files.len() as u64;

        let config = self.read_config().unwrap_or_else(|_| serde_json::json!({}));
        agent.metadata = serde_json::json!({
            "config": config,
            "sessions_dir": self.sessions_dir().to_string_lossy(),
        });

        Ok(agent)
    }

    async fn get_sessions(&self) -> Result<Vec<Session>> {
        let mut sessions = Vec::new();

        let session_files = self.scan_sessions()?;
        for file in &session_files {
            match self.parse_session_file(file) {
                Ok(s) => sessions.push(s),
                Err(e) => warn!("Failed to parse Codex session {}: {}", file.display(), e),
            }
        }

        match self.parse_history() {
            Ok(history_sessions) => sessions.extend(history_sessions),
            Err(e) => warn!("Failed to parse Codex history: {}", e),
        }

        sessions.sort_by(|a, b| b.started_at.cmp(&a.started_at));
        Ok(sessions)
    }

    async fn health_check(&self) -> Result<bool> {
        Ok(self.is_installed() && self.connected)
    }
}

#[async_trait]
impl CostTracker for CodexAdapter {
    async fn get_usage(&self) -> Result<Value> {
        let sessions = self.get_sessions().await?;
        let total_input: u64 = sessions.iter().map(|s| s.input_tokens).sum();
        let total_output: u64 = sessions.iter().map(|s| s.output_tokens).sum();
        let total_cost: f64 = sessions.iter().map(|s| s.total_cost).sum();

        Ok(serde_json::json!({
            "agent_type": "codex",
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
            "o4-mini".into(),
            "o3".into(),
            "gpt-4o".into(),
            "gpt-4-turbo".into(),
            "codex-mini-latest".into(),
        ]
    }

    async fn get_total_cost(&self) -> Result<f64> {
        let sessions = self.get_sessions().await?;
        Ok(sessions.iter().map(|s| s.total_cost).sum())
    }
}

#[async_trait]
impl SessionMonitor for CodexAdapter {
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
