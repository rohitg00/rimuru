use async_trait::async_trait;
use std::path::PathBuf;

use chrono::Utc;
use serde_json::Value;
use tracing::{debug, warn};
use uuid::Uuid;

use super::{AdapterCore, AgentAdapter, binary_on_path};
use crate::error::RimuruError;
use crate::models::{Agent, AgentStatus, AgentType, Session};

type Result<T> = std::result::Result<T, RimuruError>;

/// Windsurf (formerly Codeium) adapter.
///
/// Windsurf stores user state under `~/.windsurf/` on newer installs
/// and `~/.codeium/` on legacy ones. Conversation history lives in
/// `<config>/conversations/<uuid>.json` or `.jsonl`. Settings are in
/// `<config>/settings.json`. Token usage isn't always present (older
/// installs don't emit it) — when it's missing the session falls
/// through cost estimation with zero tokens, which matches how the
/// Cursor and Goose adapters handle their own quirky fixtures.
pub struct WindsurfAdapter {
    config_path: PathBuf,
    connected: bool,
    agent_id: Uuid,
}

impl WindsurfAdapter {
    pub fn new() -> Self {
        // No home dir → empty base; joined candidates won't match
        // any real path, so is_installed() correctly reports false.
        let home = dirs::home_dir().unwrap_or_default();
        let candidates = [
            home.join(".windsurf"),
            home.join(".codeium"),
            home.join(".config/windsurf"),
        ];
        let config_path = candidates
            .iter()
            .find(|p| p.exists())
            .cloned()
            .unwrap_or_else(|| candidates[0].clone());

        Self {
            config_path,
            connected: false,
            agent_id: Uuid::new_v4(),
        }
    }

    fn settings_file(&self) -> PathBuf {
        self.config_path.join("settings.json")
    }

    fn conversations_dir(&self) -> PathBuf {
        let primary = self.config_path.join("conversations");
        if primary.exists() {
            return primary;
        }
        // Legacy Codeium installs put history under `chat_history`
        let legacy = self.config_path.join("chat_history");
        if legacy.exists() { legacy } else { primary }
    }

    fn detect_cli_version(&self) -> Option<String> {
        for binary in ["windsurf", "codeium"] {
            if let Ok(out) = std::process::Command::new(binary).arg("--version").output()
                && out.status.success()
            {
                return Some(String::from_utf8_lossy(&out.stdout).trim().to_string());
            }
        }
        None
    }

    fn read_settings(&self) -> Result<Value> {
        let path = self.settings_file();
        if !path.exists() {
            return Ok(serde_json::json!({}));
        }
        let content = std::fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({})))
    }

    fn scan_conversations(&self) -> Result<Vec<PathBuf>> {
        let dir = self.conversations_dir();
        if !dir.exists() {
            return Ok(vec![]);
        }
        let mut files = Vec::new();
        for entry in std::fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            let ext = path.extension().and_then(|e| e.to_str());
            if matches!(ext, Some("json") | Some("jsonl")) {
                files.push(path);
            }
        }
        Ok(files)
    }

    fn parse_conversation_file(&self, path: &PathBuf) -> Result<Session> {
        let content = std::fs::read_to_string(path)?;
        let mut session = Session::new(self.agent_id, AgentType::Windsurf);

        let ext = path.extension().and_then(|e| e.to_str());
        if ext == Some("json") {
            let data: Value = serde_json::from_str(&content)?;

            if let Some(id) = data
                .get("conversationId")
                .or_else(|| data.get("id"))
                .and_then(|v| v.as_str())
                && let Ok(parsed) = uuid::Uuid::parse_str(id)
            {
                session.id = parsed;
            }

            if let Some(ts) = data
                .get("createdAt")
                .or_else(|| data.get("created_at"))
                .or_else(|| data.get("startedAt"))
                .and_then(|v| v.as_str())
                && let Ok(dt) = chrono::DateTime::parse_from_rfc3339(ts)
            {
                session.started_at = dt.with_timezone(&Utc);
            }

            if let Some(model) = data.get("model").and_then(|m| m.as_str()) {
                session.model = Some(model.to_string());
            }

            if let Some(workspace) = data
                .get("workspace")
                .or_else(|| data.get("project_path"))
                .and_then(|v| v.as_str())
            {
                session.project_path = Some(workspace.to_string());
            }

            if let Some(messages) = data
                .get("messages")
                .or_else(|| data.get("turns"))
                .and_then(|v| v.as_array())
            {
                session.messages = messages.len() as u64;
                for msg in messages {
                    if let Some(usage) = msg.get("usage") {
                        let inp = usage
                            .get("input_tokens")
                            .or_else(|| usage.get("prompt_tokens"))
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0);
                        let out = usage
                            .get("output_tokens")
                            .or_else(|| usage.get("completion_tokens"))
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0);
                        session.input_tokens += inp;
                        session.output_tokens += out;
                    }
                    if session.model.is_none()
                        && let Some(m) = msg.get("model").and_then(|v| v.as_str())
                    {
                        session.model = Some(m.to_string());
                    }
                }
            }
        } else {
            // JSONL: one event per line.
            let mut msg_count: u64 = 0;
            let mut total_input: u64 = 0;
            let mut total_output: u64 = 0;
            let mut last_model: Option<String> = None;
            let mut session_id_found: Option<String> = None;
            let mut first_ts: Option<String> = None;

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

                if session_id_found.is_none()
                    && let Some(sid) = entry
                        .get("conversationId")
                        .or_else(|| entry.get("conversation_id"))
                        .or_else(|| entry.get("id"))
                        .and_then(|v| v.as_str())
                {
                    session_id_found = Some(sid.to_string());
                }
                if first_ts.is_none()
                    && let Some(ts) = entry
                        .get("timestamp")
                        .or_else(|| entry.get("created_at"))
                        .and_then(|v| v.as_str())
                {
                    first_ts = Some(ts.to_string());
                }

                if let Some(usage) = entry.get("usage") {
                    let inp = usage
                        .get("input_tokens")
                        .or_else(|| usage.get("prompt_tokens"))
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    let out = usage
                        .get("output_tokens")
                        .or_else(|| usage.get("completion_tokens"))
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    total_input += inp;
                    total_output += out;
                }
                if let Some(model) = entry.get("model").and_then(|m| m.as_str()) {
                    last_model = Some(model.to_string());
                }
            }

            session.messages = msg_count;
            session.input_tokens = total_input;
            session.output_tokens = total_output;
            session.model = last_model;
            if let Some(sid) = session_id_found
                && let Ok(parsed) = uuid::Uuid::parse_str(&sid)
            {
                session.id = parsed;
            }
            if let Some(ts) = first_ts
                && let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&ts)
            {
                session.started_at = dt.with_timezone(&Utc);
            }
        }

        session.total_tokens = session.input_tokens + session.output_tokens;
        if let Some(ref model) = session.model {
            session.total_cost =
                Self::estimate_cost(model, session.input_tokens, session.output_tokens);
        }
        Ok(session)
    }

    fn estimate_cost(model: &str, input_tokens: u64, output_tokens: u64) -> f64 {
        // Windsurf bundles its own Cascade models alongside passthrough
        // OpenAI / Anthropic. Prices below cover the surfaces we know;
        // unknown models fall back to a mid-tier Cascade rate.
        let (input_rate, output_rate) = match model {
            m if m.contains("cascade-base") => (1.50, 6.00),
            m if m.contains("cascade") => (3.00, 15.00),
            m if m.contains("gpt-4") => (5.00, 15.00),
            m if m.contains("opus") => (15.00, 75.00),
            m if m.contains("sonnet") => (3.00, 15.00),
            m if m.contains("haiku") => (0.80, 4.00),
            _ => (3.00, 15.00),
        };
        let input = input_tokens as f64 / 1_000_000.0 * input_rate;
        let output = output_tokens as f64 / 1_000_000.0 * output_rate;
        input + output
    }
}

impl Default for WindsurfAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AgentAdapter for WindsurfAdapter {
    fn agent_type(&self) -> AgentType {
        AgentType::Windsurf
    }

    fn is_installed(&self) -> bool {
        self.config_path.exists() || binary_on_path(&["windsurf", "codeium"])
    }

    fn detect_version(&self) -> Option<String> {
        self.detect_cli_version()
    }

    async fn connect(&mut self) -> Result<()> {
        if !self.is_installed() {
            return Err(RimuruError::Adapter(format!(
                "Windsurf is not installed ({} not found and `windsurf`/`codeium` not on PATH)",
                self.config_path.display()
            )));
        }
        self.connected = true;
        debug!("Connected to Windsurf adapter");
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.connected = false;
        Ok(())
    }

    async fn get_status(&self) -> Result<Value> {
        let conversation_files = self.scan_conversations().unwrap_or_default();
        let settings = self
            .read_settings()
            .unwrap_or_else(|_| serde_json::json!({}));

        Ok(serde_json::json!({
            "agent_type": "windsurf",
            "installed": self.is_installed(),
            "connected": self.connected,
            "config_path": self.config_path.to_string_lossy(),
            "conversation_files": conversation_files.len(),
            "has_settings": settings != serde_json::json!({}),
            "version": self.detect_cli_version(),
        }))
    }

    async fn get_info(&self) -> Result<Agent> {
        let mut agent = Agent::new(AgentType::Windsurf, "Windsurf".into());
        agent.id = self.agent_id;
        agent.config_path = Some(self.config_path.to_string_lossy().to_string());
        agent.version = self.detect_cli_version();
        agent.status = if self.connected {
            AgentStatus::Connected
        } else {
            AgentStatus::Disconnected
        };
        agent.last_seen = Some(Utc::now());
        agent.session_count = self
            .get_sessions()
            .await
            .map(|s| s.len() as u64)
            .unwrap_or(0);

        let settings = self
            .read_settings()
            .unwrap_or_else(|_| serde_json::json!({}));
        agent.metadata = serde_json::json!({
            "settings": settings,
            "conversations_dir": self.conversations_dir().to_string_lossy(),
        });
        Ok(agent)
    }

    async fn get_sessions(&self) -> Result<Vec<Session>> {
        use std::collections::HashSet;

        let mut sessions = Vec::new();
        let mut seen: HashSet<uuid::Uuid> = HashSet::new();

        let files = self.scan_conversations()?;
        for file in &files {
            match self.parse_conversation_file(file) {
                Ok(s) => {
                    if seen.insert(s.id) {
                        sessions.push(s);
                    }
                }
                Err(e) => warn!(
                    "Failed to parse Windsurf conversation {}: {}",
                    file.display(),
                    e
                ),
            }
        }

        sessions.sort_by(|a, b| b.started_at.cmp(&a.started_at));
        Ok(sessions)
    }

    async fn health_check(&self) -> Result<bool> {
        Ok(self.is_installed() && self.connected)
    }
}

impl AdapterCore for WindsurfAdapter {
    fn adapter_type_name(&self) -> &'static str {
        "windsurf"
    }

    fn supported_models(&self) -> Vec<String> {
        vec![
            "windsurf-cascade".into(),
            "windsurf-cascade-base".into(),
            "gpt-4o".into(),
            "claude-sonnet-4".into(),
            "claude-opus-4".into(),
        ]
    }

    fn estimate_cost_for_model(&self, model: &str, input_tokens: u64, output_tokens: u64) -> f64 {
        Self::estimate_cost(model, input_tokens, output_tokens)
    }
}
