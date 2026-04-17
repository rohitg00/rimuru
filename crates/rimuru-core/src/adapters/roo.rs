use async_trait::async_trait;
use std::path::PathBuf;

use chrono::Utc;
use serde_json::Value;
use tracing::{debug, warn};
use uuid::Uuid;

use super::cline_base::{
    canonical_extension_storage, find_extension_storage, parse_task_dir, scan_task_dirs,
};
use super::{AdapterCore, AgentAdapter};
use crate::error::RimuruError;
use crate::models::{Agent, AgentStatus, AgentType, Session};

type Result<T> = std::result::Result<T, RimuruError>;

const EXTENSION_ID: &str = "rooveterinaryinc.roo-cline";

/// Roo Code is a Cline fork. Storage layout matches the parent
/// project — see `cline_base` — so this adapter is a thin wrapper
/// that swaps the extension ID and the agent type. Cost estimation
/// reuses the same Claude rates because Roo defaults to Anthropic.
pub struct RooAdapter {
    config_path: PathBuf,
    connected: bool,
    agent_id: Uuid,
}

impl RooAdapter {
    pub fn new() -> Self {
        // See cline.rs for the rationale: cline_base::scan_task_dirs
        // expects `<config_path>/tasks/...`, so the fallback must be
        // a globalStorage-shape path even when nothing matches today.
        let config_path = find_extension_storage(EXTENSION_ID)
            .unwrap_or_else(|| canonical_extension_storage(EXTENSION_ID));

        Self {
            config_path,
            connected: false,
            agent_id: Uuid::new_v4(),
        }
    }

    fn estimate_cost(model: &str, input_tokens: u64, output_tokens: u64) -> f64 {
        let (input_rate, output_rate) = match model {
            m if m.contains("opus-4-6") || m.contains("opus-4-5") => (5.0, 25.0),
            m if m.contains("opus-4-1") || m.contains("opus-4") => (15.0, 75.0),
            // Legacy catch-all: unversioned "opus" in a Claude model
            // id almost always means Claude 3 Opus at the older
            // $15/$75 rate, not the new Opus 4.5/4.6 pricing.
            m if m.contains("opus") => (15.0, 75.0),
            m if m.contains("sonnet") => (3.0, 15.0),
            m if m.contains("haiku-4-5") => (1.0, 5.0),
            m if m.contains("haiku-3") => (0.25, 1.25),
            m if m.contains("haiku") => (0.80, 4.0),
            _ => (3.0, 15.0),
        };
        let input = input_tokens as f64 / 1_000_000.0 * input_rate;
        let output = output_tokens as f64 / 1_000_000.0 * output_rate;
        input + output
    }
}

impl Default for RooAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AgentAdapter for RooAdapter {
    fn agent_type(&self) -> AgentType {
        AgentType::Roo
    }

    fn is_installed(&self) -> bool {
        self.config_path.exists()
    }

    fn detect_version(&self) -> Option<String> {
        None
    }

    async fn connect(&mut self) -> Result<()> {
        if !self.is_installed() {
            return Err(RimuruError::Adapter(format!(
                "Roo Code is not installed (expected VS Code extension at {})",
                self.config_path.display()
            )));
        }
        self.connected = true;
        debug!("Connected to Roo Code adapter");
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.connected = false;
        Ok(())
    }

    async fn get_status(&self) -> Result<Value> {
        let task_dirs = scan_task_dirs(&self.config_path).unwrap_or_default();
        Ok(serde_json::json!({
            "agent_type": "roo",
            "installed": self.is_installed(),
            "connected": self.connected,
            "config_path": self.config_path.to_string_lossy(),
            "task_count": task_dirs.len(),
        }))
    }

    async fn get_info(&self) -> Result<Agent> {
        let mut agent = Agent::new(AgentType::Roo, "Roo Code".into());
        agent.id = self.agent_id;
        agent.config_path = Some(self.config_path.to_string_lossy().to_string());
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
        agent.metadata = serde_json::json!({
            "extension_id": EXTENSION_ID,
            "fork_of": "cline",
        });
        Ok(agent)
    }

    async fn get_sessions(&self) -> Result<Vec<Session>> {
        let mut sessions = Vec::new();
        let task_dirs = scan_task_dirs(&self.config_path)?;
        for dir in &task_dirs {
            match parse_task_dir(dir, self.agent_id, AgentType::Roo) {
                Ok(mut s) => {
                    if let Some(ref model) = s.model {
                        s.total_cost = Self::estimate_cost(model, s.input_tokens, s.output_tokens);
                    }
                    sessions.push(s);
                }
                Err(e) => warn!("Failed to parse Roo Code task {}: {}", dir.display(), e),
            }
        }
        sessions.sort_by_key(|b| std::cmp::Reverse(b.started_at));
        Ok(sessions)
    }

    async fn health_check(&self) -> Result<bool> {
        Ok(self.is_installed() && self.connected)
    }
}

impl AdapterCore for RooAdapter {
    fn adapter_type_name(&self) -> &'static str {
        "roo"
    }

    fn supported_models(&self) -> Vec<String> {
        vec![
            "claude-opus-4-6".into(),
            "claude-sonnet-4-6".into(),
            "claude-haiku-4-5".into(),
        ]
    }

    fn estimate_cost_for_model(&self, model: &str, input_tokens: u64, output_tokens: u64) -> f64 {
        Self::estimate_cost(model, input_tokens, output_tokens)
    }
}
