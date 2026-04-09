use async_trait::async_trait;
use std::path::PathBuf;

use chrono::Utc;
use serde_json::Value;
use tracing::{debug, warn};
use uuid::Uuid;

use super::{AdapterCore, AgentAdapter};
use crate::error::RimuruError;
use crate::models::{
    Agent, AgentStatus, AgentType, ContextBreakdown, Session, SessionStatus, ToolCallRecord,
    TurnRecord,
};

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

    pub fn find_session_file(&self, session_id: &str) -> Option<PathBuf> {
        let files = self.scan_session_files().ok()?;
        files.into_iter().find(|p| {
            p.file_stem()
                .and_then(|s| s.to_str())
                .is_some_and(|stem| stem == session_id)
        })
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
                if let Some(ext) = path.extension()
                    && ext == "jsonl"
                    && let Some(stem) = path.file_stem().and_then(|s| s.to_str())
                    && uuid::Uuid::parse_str(stem).is_ok()
                {
                    results.push(path);
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
        let (session, _breakdown) = self.parse_session_jsonl_full(jsonl_path)?;
        Ok(session)
    }

    pub fn parse_session_jsonl_full(
        &self,
        jsonl_path: &PathBuf,
    ) -> Result<(Session, ContextBreakdown)> {
        let content = std::fs::read_to_string(jsonl_path)?;

        let mut session = Session::new(self.agent_id, AgentType::ClaudeCode);
        let mut breakdown = ContextBreakdown::new(session.id);

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
        let mut turn_index: u32 = 0;

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let entry: Value = match serde_json::from_str(line) {
                Ok(v) => v,
                Err(_) => continue,
            };

            let timestamp = entry
                .get("timestamp")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            if let Some(ref ts) = timestamp {
                if first_timestamp.is_none() {
                    first_timestamp = Some(ts.clone());
                }
                last_timestamp = Some(ts.clone());
            }

            if session_id_found.is_none()
                && let Some(sid) = entry.get("sessionId").and_then(|v| v.as_str())
            {
                session_id_found = Some(sid.to_string());
            }

            if let Some(msg) = entry.get("message") {
                let role = msg
                    .get("role")
                    .and_then(|r| r.as_str())
                    .unwrap_or("unknown");
                let model = msg.get("model").and_then(|m| m.as_str()).map(String::from);

                if role == "assistant" {
                    msg_count += 1;
                }
                if let Some(ref m) = model {
                    last_model = Some(m.clone());
                }

                let mut turn_input: u64 = 0;
                let mut turn_output: u64 = 0;
                let mut turn_cache_read: u64 = 0;
                let mut turn_cache_write: u64 = 0;

                if let Some(usage) = msg.get("usage") {
                    turn_input = usage
                        .get("input_tokens")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    turn_output = usage
                        .get("output_tokens")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    turn_cache_read = usage
                        .get("cache_read_input_tokens")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);
                    turn_cache_write = usage
                        .get("cache_creation_input_tokens")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0);

                    total_input += turn_input;
                    total_output += turn_output;
                    total_cache_read += turn_cache_read;
                    total_cache_write += turn_cache_write;
                }

                let mut tool_calls = Vec::new();
                let mut content_type = "text".to_string();

                if let Some(content_arr) = msg.get("content").and_then(|c| c.as_array()) {
                    for block in content_arr {
                        let block_type =
                            block.get("type").and_then(|t| t.as_str()).unwrap_or("text");

                        match block_type {
                            "tool_use" => {
                                content_type = "tool_use".to_string();
                                let tool_name = block
                                    .get("name")
                                    .and_then(|n| n.as_str())
                                    .unwrap_or("unknown")
                                    .to_string();
                                let tool_id =
                                    block.get("id").and_then(|i| i.as_str()).map(String::from);
                                let input_est = block
                                    .get("input")
                                    .map(|v| v.to_string().len() as u64 / 4)
                                    .unwrap_or(0);

                                Self::classify_tool_tokens(
                                    &tool_name,
                                    input_est,
                                    0,
                                    &mut breakdown,
                                );

                                tool_calls.push(ToolCallRecord {
                                    tool_name,
                                    tool_id,
                                    input_tokens_estimate: input_est,
                                    output_tokens_estimate: 0,
                                });
                            }
                            "tool_result" => {
                                content_type = "tool_result".to_string();
                                let output_est = block
                                    .get("content")
                                    .map(|v| v.to_string().len() as u64 / 4)
                                    .unwrap_or(0);

                                let tool_use_id = block
                                    .get("tool_use_id")
                                    .or_else(|| block.get("id"))
                                    .and_then(|v| v.as_str());

                                let matched = tool_use_id.and_then(|tid| {
                                    tool_calls
                                        .iter_mut()
                                        .find(|tc| tc.tool_id.as_deref() == Some(tid))
                                });

                                if let Some(tc) = matched {
                                    tc.output_tokens_estimate = output_est;
                                    Self::classify_tool_tokens(
                                        &tc.tool_name,
                                        0,
                                        output_est,
                                        &mut breakdown,
                                    );
                                } else if let Some(last_tool) = tool_calls.last_mut() {
                                    last_tool.output_tokens_estimate = output_est;
                                    Self::classify_tool_tokens(
                                        &last_tool.tool_name,
                                        0,
                                        output_est,
                                        &mut breakdown,
                                    );
                                } else {
                                    breakdown.tool_result_tokens += output_est;
                                }
                            }
                            "text" => {
                                let text_est = block
                                    .get("text")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.len() as u64 / 4)
                                    .unwrap_or(0);

                                match role {
                                    "user" | "human" => breakdown.user_tokens += text_est,
                                    "assistant" => breakdown.assistant_tokens += text_est,
                                    "system" => breakdown.system_prompt_tokens += text_est,
                                    _ => breakdown.conversation_tokens += text_est,
                                }
                            }
                            _ => {}
                        }
                    }
                } else {
                    match role {
                        "user" | "human" => breakdown.user_tokens += turn_input,
                        "assistant" => breakdown.assistant_tokens += turn_output,
                        "system" => breakdown.system_prompt_tokens += turn_input,
                        _ => {}
                    }
                }

                if turn_input > 0 || turn_output > 0 {
                    let estimated_from_content: u64 = tool_calls
                        .iter()
                        .map(|tc| tc.input_tokens_estimate + tc.output_tokens_estimate)
                        .sum::<u64>();

                    let actual_total = turn_input + turn_output;
                    if actual_total > estimated_from_content {
                        let unattributed = actual_total - estimated_from_content;
                        match role {
                            "assistant" => breakdown.conversation_tokens += unattributed,
                            "user" | "human" => breakdown.user_tokens += unattributed,
                            "system" => breakdown.system_prompt_tokens += unattributed,
                            _ => breakdown.conversation_tokens += unattributed,
                        }
                    }

                    breakdown.turns.push(TurnRecord {
                        turn_index,
                        role: role.to_string(),
                        model: model.clone(),
                        input_tokens: turn_input,
                        output_tokens: turn_output,
                        cache_read: turn_cache_read,
                        cache_write: turn_cache_write,
                        tool_calls,
                        timestamp: timestamp.clone(),
                        content_type,
                    });
                    turn_index += 1;
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
                breakdown.session_id = parsed;
            }
        } else if let Some(stem) = jsonl_path.file_stem().and_then(|s| s.to_str())
            && let Ok(parsed) = uuid::Uuid::parse_str(stem)
        {
            session.id = parsed;
            breakdown.session_id = parsed;
        }

        if let Some(ref ts) = first_timestamp
            && let Ok(dt) = chrono::DateTime::parse_from_rfc3339(ts)
        {
            session.started_at = dt.with_timezone(&Utc);
        }

        session.messages = msg_count;
        session.input_tokens = total_input;
        session.output_tokens = total_output;
        session.total_tokens = total_input + total_output + total_cache_read + total_cache_write;
        session.model = last_model.clone();

        breakdown.total_tokens = session.total_tokens;
        breakdown.cache_read_tokens = total_cache_read;
        breakdown.cache_write_tokens = total_cache_write;

        let turns_json = serde_json::to_value(&breakdown.turns).unwrap_or_default();
        session.metadata = serde_json::json!({"turns": turns_json});

        if let Some(ref model) = last_model {
            session.total_cost = Self::estimate_cost_full(
                model,
                total_input,
                total_output,
                total_cache_read,
                total_cache_write,
            );
        }

        if let Ok(metadata) = std::fs::metadata(jsonl_path)
            && let Ok(modified) = metadata.modified()
        {
            let elapsed = modified.elapsed().unwrap_or_default();
            if elapsed.as_secs() > 3600 {
                session.status = SessionStatus::Completed;
                session.ended_at = Some(chrono::DateTime::<Utc>::from(modified));
            } else if let Some(ref ts) = last_timestamp
                && let Ok(dt) = chrono::DateTime::parse_from_rfc3339(ts)
            {
                let age = Utc::now() - dt.with_timezone(&Utc);
                if age.num_seconds() > 3600 {
                    session.status = SessionStatus::Completed;
                    session.ended_at = Some(dt.with_timezone(&Utc));
                }
            }
        }

        Ok((session, breakdown))
    }

    fn classify_tool_tokens(
        tool_name: &str,
        input_est: u64,
        output_est: u64,
        breakdown: &mut ContextBreakdown,
    ) {
        let total = input_est + output_est;
        match tool_name {
            "Read" | "read_file" | "ReadFile" => breakdown.file_read_tokens += total,
            "Bash" | "bash" | "execute_command" => breakdown.bash_output_tokens += total,
            "Grep" | "grep" | "search" | "Glob" | "glob" => breakdown.file_read_tokens += total,
            "Edit" | "Write" | "edit_file" | "write_file" => breakdown.file_read_tokens += total,
            name if name.starts_with("mcp_") || name.contains("::") => {
                breakdown.mcp_tokens += total
            }
            _ => breakdown.tool_result_tokens += total,
        }
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
        // Pricing from Claude Code source: src/utils/modelCost.ts
        // https://platform.claude.com/docs/en/about-claude/pricing
        let (input_rate, output_rate, cache_read_rate, cache_write_rate) = match model {
            m if m.contains("opus-4-6") || m.contains("opus-4-5") => (5.0, 25.0, 0.50, 6.25),
            m if m.contains("opus-4-1") || m.contains("opus-4") => (15.0, 75.0, 1.50, 18.75),
            m if m.contains("opus") => (5.0, 25.0, 0.50, 6.25),
            m if m.contains("sonnet") => (3.0, 15.0, 0.30, 3.75),
            m if m.contains("haiku-4-5") => (1.0, 5.0, 0.10, 1.25),
            m if m.contains("haiku") => (0.80, 4.0, 0.08, 1.0),
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
        let settings = self
            .read_settings()
            .unwrap_or_else(|_| serde_json::json!({}));
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

        let settings = self
            .read_settings()
            .unwrap_or_else(|_| serde_json::json!({}));
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

impl AdapterCore for ClaudeCodeAdapter {
    fn adapter_type_name(&self) -> &'static str {
        "claude_code"
    }

    fn supported_models(&self) -> Vec<String> {
        vec![
            "claude-opus-4-6".into(),
            "claude-sonnet-4-20250514".into(),
            "claude-haiku-3-5-20241022".into(),
            "claude-3-5-sonnet-20241022".into(),
            "claude-3-opus-20240229".into(),
        ]
    }

    fn estimate_cost_for_model(&self, model: &str, input_tokens: u64, output_tokens: u64) -> f64 {
        Self::estimate_cost(model, input_tokens, output_tokens)
    }
}
