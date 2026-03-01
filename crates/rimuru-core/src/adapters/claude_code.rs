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

pub struct ClaudeCodeAdapter {
    config_path: PathBuf,
    connected: bool,
    agent_id: Uuid,
}

impl ClaudeCodeAdapter {
    pub fn new() -> Self {
        let config_path = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join(".claude");

        Self {
            config_path,
            connected: false,
            agent_id: Uuid::new_v4(),
        }
    }

    fn projects_dir(&self) -> PathBuf {
        self.config_path.join("projects")
    }

    fn settings_path(&self) -> PathBuf {
        self.config_path.join("settings.json")
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
        let output = std::process::Command::new("claude")
            .arg("--version")
            .output()
            .ok()?;
        if output.status.success() {
            let ver = String::from_utf8_lossy(&output.stdout).trim().to_string();
            Some(ver)
        } else {
            None
        }
    }

    fn scan_session_files(&self) -> Result<Vec<PathBuf>> {
        let projects = self.projects_dir();
        if !projects.exists() {
            return Ok(vec![]);
        }
        let mut session_files = Vec::new();
        Self::walk_for_jsonl(&projects, &mut session_files)?;
        Ok(session_files)
    }

    fn walk_for_jsonl(dir: &PathBuf, results: &mut Vec<PathBuf>) -> Result<()> {
        if !dir.is_dir() {
            return Ok(());
        }
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "jsonl" {
                        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                            if uuid::Uuid::parse_str(stem).is_ok() {
                                results.push(path);
                            }
                        }
                    }
                }
            } else if path.is_dir() {
                let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if dir_name != "subagents" {
                    Self::walk_for_jsonl(&path, results)?;
                }
            }
        }
        Ok(())
    }

    fn parse_session_jsonl(&self, jsonl_path: &PathBuf) -> Result<Session> {
        let content = std::fs::read_to_string(jsonl_path)?;

        let mut session = Session::new(self.agent_id, AgentType::ClaudeCode);

        let project_dir = jsonl_path
            .parent()
            .map(|p| p.file_name().and_then(|n| n.to_str()).unwrap_or(""))
            .unwrap_or("");
        session.project_path = Some(project_dir.to_string());

        let mut msg_count: u64 = 0;
        let mut total_input: u64 = 0;
        let mut total_output: u64 = 0;
        let mut total_cache_read: u64 = 0;
        let mut total_cache_write: u64 = 0;
        let mut last_model: Option<String> = None;
        let mut session_id_found: Option<String> = None;
        let mut first_timestamp: Option<String> = None;
        let mut last_timestamp: Option<String> = None;

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let entry: Value = match serde_json::from_str(line) {
                Ok(v) => v,
                Err(_) => continue,
            };

            if let Some(ts) = entry.get("timestamp").and_then(|v| v.as_str()) {
                if first_timestamp.is_none() {
                    first_timestamp = Some(ts.to_string());
                }
                last_timestamp = Some(ts.to_string());
            }

            if session_id_found.is_none() {
                if let Some(sid) = entry.get("sessionId").and_then(|v| v.as_str()) {
                    session_id_found = Some(sid.to_string());
                }
            }

            if let Some(msg) = entry.get("message") {
                if msg.get("role").and_then(|r| r.as_str()) == Some("assistant") {
                    msg_count += 1;
                }

                if let Some(model) = msg.get("model").and_then(|m| m.as_str()) {
                    last_model = Some(model.to_string());
                }

                if let Some(usage) = msg.get("usage") {
                    if let Some(inp) = usage.get("input_tokens").and_then(|v| v.as_u64()) {
                        total_input += inp;
                    }
                    if let Some(out) = usage.get("output_tokens").and_then(|v| v.as_u64()) {
                        total_output += out;
                    }
                    if let Some(cr) = usage.get("cache_read_input_tokens").and_then(|v| v.as_u64())
                    {
                        total_cache_read += cr;
                    }
                    if let Some(cw) = usage
                        .get("cache_creation_input_tokens")
                        .and_then(|v| v.as_u64())
                    {
                        total_cache_write += cw;
                    }
                }
            }

            let entry_type = entry.get("type").and_then(|t| t.as_str()).unwrap_or("");
            if entry_type == "human" || entry_type == "user" {
                msg_count += 1;
            }
        }

        if let Some(ref sid) = session_id_found {
            if let Ok(parsed) = uuid::Uuid::parse_str(sid) {
                session.id = parsed;
            }
        } else if let Some(stem) = jsonl_path.file_stem().and_then(|s| s.to_str()) {
            if let Ok(parsed) = uuid::Uuid::parse_str(stem) {
                session.id = parsed;
            }
        }

        if let Some(ref ts) = first_timestamp {
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(ts) {
                session.started_at = dt.with_timezone(&Utc);
            }
        }

        session.messages = msg_count;
        session.input_tokens = total_input;
        session.output_tokens = total_output;
        session.total_tokens = total_input + total_output + total_cache_read + total_cache_write;
        session.model = last_model.clone();

        if let Some(ref model) = last_model {
            session.total_cost = Self::estimate_cost_full(
                model,
                total_input,
                total_output,
                total_cache_read,
                total_cache_write,
            );
        }

        if let Ok(metadata) = std::fs::metadata(jsonl_path) {
            if let Ok(modified) = metadata.modified() {
                let elapsed = modified.elapsed().unwrap_or_default();
                if elapsed.as_secs() > 3600 {
                    session.status = SessionStatus::Completed;
                    session.ended_at = Some(chrono::DateTime::<Utc>::from(modified));
                } else if let Some(ref ts) = last_timestamp {
                    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(ts) {
                        let age = Utc::now() - dt.with_timezone(&Utc);
                        if age.num_seconds() > 3600 {
                            session.status = SessionStatus::Completed;
                            session.ended_at = Some(dt.with_timezone(&Utc));
                        }
                    }
                }
            }
        }

        Ok(session)
    }

    fn estimate_cost(model: &str, input_tokens: u64, output_tokens: u64) -> f64 {
        Self::estimate_cost_full(model, input_tokens, output_tokens, 0, 0)
    }

    fn estimate_cost_full(
        model: &str,
        input_tokens: u64,
        output_tokens: u64,
        cache_read_tokens: u64,
        cache_write_tokens: u64,
    ) -> f64 {
        let (input_rate, output_rate, cache_read_rate, cache_write_rate) = match model {
            m if m.contains("opus") => (15.0, 75.0, 1.50, 18.75),
            m if m.contains("sonnet") => (3.0, 15.0, 0.30, 3.75),
            m if m.contains("haiku") => (0.25, 1.25, 0.03, 0.30),
            _ => (3.0, 15.0, 0.30, 3.75),
        };
        let per_m = |tokens: u64, rate: f64| (tokens as f64 / 1_000_000.0) * rate;
        per_m(input_tokens, input_rate)
            + per_m(output_tokens, output_rate)
            + per_m(cache_read_tokens, cache_read_rate)
            + per_m(cache_write_tokens, cache_write_rate)
    }
}

impl Default for ClaudeCodeAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AgentAdapter for ClaudeCodeAdapter {
    fn agent_type(&self) -> AgentType {
        AgentType::ClaudeCode
    }

    fn is_installed(&self) -> bool {
        self.config_path.exists()
    }

    fn detect_version(&self) -> Option<String> {
        let output = std::process::Command::new("claude")
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
                "Claude Code is not installed (~/.claude not found)".into(),
            ));
        }
        self.connected = true;
        debug!("Connected to Claude Code adapter");
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.connected = false;
        debug!("Disconnected from Claude Code adapter");
        Ok(())
    }

    async fn get_status(&self) -> Result<Value> {
        let settings = self.read_settings().unwrap_or_else(|_| serde_json::json!({}));
        let sessions = self.scan_session_files().unwrap_or_default();

        Ok(serde_json::json!({
            "agent_type": "claude_code",
            "installed": self.is_installed(),
            "connected": self.connected,
            "config_path": self.config_path.to_string_lossy(),
            "session_count": sessions.len(),
            "has_settings": settings != serde_json::json!({}),
            "version": self.detect_version(),
        }))
    }

    async fn get_info(&self) -> Result<Agent> {
        let mut agent = Agent::new(AgentType::ClaudeCode, "Claude Code".into());
        agent.id = self.agent_id;
        agent.config_path = Some(self.config_path.to_string_lossy().to_string());
        agent.version = self.detect_version();
        agent.status = if self.connected {
            AgentStatus::Connected
        } else {
            AgentStatus::Disconnected
        };
        agent.last_seen = Some(Utc::now());

        if let Ok(sessions) = self.scan_session_files() {
            agent.session_count = sessions.len() as u64;
        }

        let settings = self.read_settings().unwrap_or_else(|_| serde_json::json!({}));
        agent.metadata = serde_json::json!({
            "settings": settings,
            "projects_dir": self.projects_dir().to_string_lossy(),
        });

        Ok(agent)
    }

    async fn get_sessions(&self) -> Result<Vec<Session>> {
        let dirs = self.scan_session_files()?;
        let mut sessions = Vec::new();
        for dir in &dirs {
            match self.parse_session_jsonl(dir) {
                Ok(s) => sessions.push(s),
                Err(e) => warn!("Failed to parse session at {}: {}", dir.display(), e),
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
impl CostTracker for ClaudeCodeAdapter {
    async fn get_usage(&self) -> Result<Value> {
        let sessions = self.get_sessions().await?;
        let total_input: u64 = sessions.iter().map(|s| s.input_tokens).sum();
        let total_output: u64 = sessions.iter().map(|s| s.output_tokens).sum();
        let total_cost: f64 = sessions.iter().map(|s| s.total_cost).sum();

        Ok(serde_json::json!({
            "agent_type": "claude_code",
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
            "claude-opus-4-6".into(),
            "claude-sonnet-4-20250514".into(),
            "claude-haiku-3-5-20241022".into(),
            "claude-3-5-sonnet-20241022".into(),
            "claude-3-opus-20240229".into(),
        ]
    }

    async fn get_total_cost(&self) -> Result<f64> {
        let sessions = self.get_sessions().await?;
        Ok(sessions.iter().map(|s| s.total_cost).sum())
    }
}

#[async_trait]
impl SessionMonitor for ClaudeCodeAdapter {
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
