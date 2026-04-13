use async_trait::async_trait;
use std::path::PathBuf;

use chrono::Utc;
use serde_json::Value;
use tracing::{debug, warn};
use uuid::Uuid;

use super::{AdapterCore, AgentAdapter};
use crate::error::RimuruError;
use crate::models::{Agent, AgentStatus, AgentType, Session, SessionStatus};

type Result<T> = std::result::Result<T, RimuruError>;

fn gemini_binary_on_path() -> bool {
    let Some(path_var) = std::env::var_os("PATH") else {
        return false;
    };
    let exe = if cfg!(windows) {
        "gemini.exe"
    } else {
        "gemini"
    };
    std::env::split_paths(&path_var).any(|dir| dir.join(exe).is_file())
}

pub struct GeminiCliAdapter {
    config_path: PathBuf,
    connected: bool,
    agent_id: Uuid,
}

impl GeminiCliAdapter {
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
        let config_path = if home.join(".gemini").exists() {
            home.join(".gemini")
        } else {
            home.join(".config/gemini")
        };

        Self {
            config_path,
            connected: false,
            agent_id: Uuid::new_v4(),
        }
    }

    fn settings_file(&self) -> PathBuf {
        self.config_path.join("settings.json")
    }

    fn sessions_dir(&self) -> PathBuf {
        self.config_path.join("sessions")
    }

    fn history_file(&self) -> PathBuf {
        self.config_path.join("history.jsonl")
    }

    fn read_settings(&self) -> Result<Value> {
        let path = self.settings_file();
        if !path.exists() {
            return Ok(serde_json::json!({}));
        }
        let content = std::fs::read_to_string(&path)?;
        let val: Value = serde_json::from_str(&content)?;
        Ok(val)
    }

    fn detect_cli_version(&self) -> Option<String> {
        let output = std::process::Command::new("gemini")
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
            let ext = path.extension().and_then(|e| e.to_str());
            if matches!(ext, Some("json") | Some("jsonl")) {
                files.push(path);
            }
        }
        Ok(files)
    }

    fn parse_session_file(&self, path: &PathBuf) -> Result<Session> {
        let content = std::fs::read_to_string(path)?;
        let mut session = Session::new(self.agent_id, AgentType::GeminiCli);

        let ext = path.extension().and_then(|e| e.to_str());
        if ext == Some("json") {
            let data: Value = serde_json::from_str(&content)?;

            if let Some(id) = data
                .get("sessionId")
                .or_else(|| data.get("id"))
                .and_then(|v| v.as_str())
                && let Ok(parsed) = uuid::Uuid::parse_str(id)
            {
                session.id = parsed;
            }

            if let Some(ts) = data
                .get("startedAt")
                .or_else(|| data.get("created_at"))
                .and_then(|v| v.as_str())
                && let Ok(dt) = chrono::DateTime::parse_from_rfc3339(ts)
            {
                session.started_at = dt.with_timezone(&Utc);
            }

            if let Some(model) = data.get("model").and_then(|m| m.as_str()) {
                session.model = Some(model.to_string());
            }

            if let Some(project) = data
                .get("projectPath")
                .or_else(|| data.get("project_path"))
                .or_else(|| data.get("workspace"))
                .and_then(|p| p.as_str())
            {
                session.project_path = Some(project.to_string());
            }

            if let Some(turns) = data
                .get("turns")
                .or_else(|| data.get("messages"))
                .and_then(|v| v.as_array())
            {
                session.messages = turns.len() as u64;
                for turn in turns {
                    let usage = turn.get("usage").or_else(|| turn.get("usageMetadata"));
                    if let Some(u) = usage {
                        let inp = u
                            .get("promptTokenCount")
                            .or_else(|| u.get("input_tokens"))
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0);
                        let out = u
                            .get("candidatesTokenCount")
                            .or_else(|| u.get("output_tokens"))
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0);
                        session.input_tokens += inp;
                        session.output_tokens += out;
                    }
                    if session.model.is_none()
                        && let Some(m) = turn.get("model").and_then(|v| v.as_str())
                    {
                        session.model = Some(m.to_string());
                    }
                }
            }
        } else {
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
                        .get("sessionId")
                        .or_else(|| entry.get("session_id"))
                        .and_then(|v| v.as_str())
                {
                    session_id_found = Some(sid.to_string());
                }

                if first_ts.is_none()
                    && let Some(ts) = entry
                        .get("timestamp")
                        .or_else(|| entry.get("createdAt"))
                        .and_then(|v| v.as_str())
                {
                    first_ts = Some(ts.to_string());
                }

                let usage = entry.get("usage").or_else(|| entry.get("usageMetadata"));
                if let Some(u) = usage {
                    let inp = u
                        .get("promptTokenCount")
                        .or_else(|| u.get("input_tokens"))
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    let out = u
                        .get("candidatesTokenCount")
                        .or_else(|| u.get("output_tokens"))
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

        if let Ok(metadata) = std::fs::metadata(path)
            && let Ok(modified) = metadata.modified()
        {
            let elapsed = modified.elapsed().unwrap_or_default();
            if elapsed.as_secs() > 3600 {
                session.status = SessionStatus::Completed;
                session.ended_at = Some(chrono::DateTime::<Utc>::from(modified));
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

            let mut session = Session::new(self.agent_id, AgentType::GeminiCli);

            if let Some(sid) = entry
                .get("sessionId")
                .or_else(|| entry.get("session_id"))
                .or_else(|| entry.get("id"))
                .and_then(|v| v.as_str())
                && let Ok(parsed) = uuid::Uuid::parse_str(sid)
            {
                session.id = parsed;
            }

            if let Some(ts) = entry
                .get("timestamp")
                .or_else(|| entry.get("createdAt"))
                .or_else(|| entry.get("created_at"))
                .and_then(|v| v.as_str())
                && let Ok(dt) = chrono::DateTime::parse_from_rfc3339(ts)
            {
                session.started_at = dt.with_timezone(&Utc);
            }

            if let Some(end_ts) = entry
                .get("endedAt")
                .or_else(|| entry.get("ended_at"))
                .and_then(|v| v.as_str())
                && let Ok(dt) = chrono::DateTime::parse_from_rfc3339(end_ts)
            {
                session.ended_at = Some(dt.with_timezone(&Utc));
            }

            if let Some(model) = entry.get("model").and_then(|m| m.as_str()) {
                session.model = Some(model.to_string());
            }
            let usage = entry.get("usage").or_else(|| entry.get("usageMetadata"));
            if let Some(u) = usage {
                let inp = u
                    .get("promptTokenCount")
                    .or_else(|| u.get("input_tokens"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let out = u
                    .get("candidatesTokenCount")
                    .or_else(|| u.get("output_tokens"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                session.input_tokens = inp;
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
        // Rates from https://ai.google.dev/gemini-api/docs/pricing (USD per
        // 1M tokens, text/image/video paid tier). Order matters: the more
        // specific "flash-lite" variants must match before the broader
        // "flash" arms.
        let (input_rate, output_rate) = match model {
            m if m.contains("2.5-pro") => (1.25, 10.00),
            m if m.contains("2.5-flash-lite") => (0.10, 0.40),
            m if m.contains("2.5-flash") => (0.30, 2.50),
            m if m.contains("2.0-flash-lite") => (0.075, 0.30),
            m if m.contains("2.0-flash") => (0.15, 0.60),
            m if m.contains("1.5-pro") => (1.25, 5.00),
            m if m.contains("1.5-flash-8b") => (0.0375, 0.15),
            m if m.contains("1.5-flash") => (0.075, 0.30),
            _ => (0.30, 2.50),
        };
        let input_cost = (input_tokens as f64 / 1_000_000.0) * input_rate;
        let output_cost = (output_tokens as f64 / 1_000_000.0) * output_rate;
        input_cost + output_cost
    }
}

impl Default for GeminiCliAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AgentAdapter for GeminiCliAdapter {
    fn agent_type(&self) -> AgentType {
        AgentType::GeminiCli
    }

    fn is_installed(&self) -> bool {
        self.config_path.exists() || gemini_binary_on_path()
    }

    fn detect_version(&self) -> Option<String> {
        self.detect_cli_version()
    }

    async fn connect(&mut self) -> Result<()> {
        if !self.is_installed() {
            return Err(RimuruError::Adapter(format!(
                "Gemini CLI is not installed ({} not found and `gemini` not on PATH)",
                self.config_path.display()
            )));
        }
        self.connected = true;
        debug!("Connected to Gemini CLI adapter");
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.connected = false;
        debug!("Disconnected from Gemini CLI adapter");
        Ok(())
    }

    async fn get_status(&self) -> Result<Value> {
        let settings = self
            .read_settings()
            .unwrap_or_else(|_| serde_json::json!({}));
        let session_files = self.scan_sessions().unwrap_or_default();

        Ok(serde_json::json!({
            "agent_type": "gemini_cli",
            "installed": self.is_installed(),
            "connected": self.connected,
            "config_path": self.config_path.to_string_lossy(),
            "session_files": session_files.len(),
            "has_settings": settings != serde_json::json!({}),
            "has_history": self.history_file().exists(),
            "version": self.detect_cli_version(),
        }))
    }

    async fn get_info(&self) -> Result<Agent> {
        let mut agent = Agent::new(AgentType::GeminiCli, "Gemini CLI".into());
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
            "sessions_dir": self.sessions_dir().to_string_lossy(),
        });

        Ok(agent)
    }

    async fn get_sessions(&self) -> Result<Vec<Session>> {
        use std::collections::HashSet;

        let mut sessions = Vec::new();
        let mut seen_ids: HashSet<uuid::Uuid> = HashSet::new();

        let session_files = self.scan_sessions()?;
        for file in &session_files {
            match self.parse_session_file(file) {
                Ok(s) => {
                    seen_ids.insert(s.id);
                    sessions.push(s);
                }
                Err(e) => warn!(
                    "Failed to parse Gemini CLI session {}: {}",
                    file.display(),
                    e
                ),
            }
        }

        match self.parse_history() {
            Ok(history_sessions) => {
                for s in history_sessions {
                    if seen_ids.insert(s.id) {
                        sessions.push(s);
                    }
                }
            }
            Err(e) => warn!("Failed to parse Gemini CLI history: {}", e),
        }

        sessions.sort_by(|a, b| b.started_at.cmp(&a.started_at));
        Ok(sessions)
    }

    async fn health_check(&self) -> Result<bool> {
        Ok(self.is_installed() && self.connected)
    }
}

impl AdapterCore for GeminiCliAdapter {
    fn adapter_type_name(&self) -> &'static str {
        "gemini_cli"
    }

    fn supported_models(&self) -> Vec<String> {
        vec![
            "gemini-2.5-pro".into(),
            "gemini-2.5-flash".into(),
            "gemini-2.0-flash".into(),
            "gemini-1.5-pro".into(),
            "gemini-1.5-flash".into(),
            "gemini-1.5-flash-8b".into(),
        ]
    }

    fn estimate_cost_for_model(&self, model: &str, input_tokens: u64, output_tokens: u64) -> f64 {
        Self::estimate_cost(model, input_tokens, output_tokens)
    }
}
