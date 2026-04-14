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

/// Amp (Sourcegraph) adapter — installation + version detection only.
///
/// Session parsing is intentionally a stub: Amp's on-disk format is
/// not yet documented in a way we can rely on, and emitting empty
/// sessions with a tracing::warn is preferable to fabricating values
/// the dashboard would happily display. Once we have a real session
/// fixture we can lift the parser straight from the Gemini adapter.
pub struct AmpAdapter {
    config_path: PathBuf,
    connected: bool,
    agent_id: Uuid,
}

impl AmpAdapter {
    pub fn new() -> Self {
        // No home dir → empty base; joined candidates won't match
        // any real path, so is_installed() correctly reports false.
        let home = dirs::home_dir().unwrap_or_default();
        let candidates = [home.join(".amp"), home.join(".config/amp")];
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

    fn detect_cli_version(&self) -> Option<String> {
        let output = std::process::Command::new("amp")
            .arg("--version")
            .output()
            .ok()?;
        if output.status.success() {
            Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            None
        }
    }
}

impl Default for AmpAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AgentAdapter for AmpAdapter {
    fn agent_type(&self) -> AgentType {
        AgentType::Amp
    }

    fn is_installed(&self) -> bool {
        self.config_path.exists() || binary_on_path(&["amp"])
    }

    fn detect_version(&self) -> Option<String> {
        self.detect_cli_version()
    }

    async fn connect(&mut self) -> Result<()> {
        if !self.is_installed() {
            return Err(RimuruError::Adapter(format!(
                "Amp is not installed ({} not found and `amp` not on PATH)",
                self.config_path.display()
            )));
        }
        self.connected = true;
        debug!("Connected to Amp adapter (stub — no session parsing)");
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.connected = false;
        Ok(())
    }

    async fn get_status(&self) -> Result<Value> {
        Ok(serde_json::json!({
            "agent_type": "amp",
            "installed": self.is_installed(),
            "connected": self.connected,
            "config_path": self.config_path.to_string_lossy(),
            "version": self.detect_cli_version(),
            "session_parsing": "stub",
        }))
    }

    async fn get_info(&self) -> Result<Agent> {
        let mut agent = Agent::new(AgentType::Amp, "Amp".into());
        agent.id = self.agent_id;
        agent.config_path = Some(self.config_path.to_string_lossy().to_string());
        agent.version = self.detect_cli_version();
        agent.status = if self.connected {
            AgentStatus::Connected
        } else {
            AgentStatus::Disconnected
        };
        agent.last_seen = Some(Utc::now());
        agent.session_count = 0;
        agent.metadata = serde_json::json!({
            "session_parsing": "stub",
            "note": "Amp session format not yet supported — sessions list returns empty",
        });
        Ok(agent)
    }

    async fn get_sessions(&self) -> Result<Vec<Session>> {
        warn!(
            "Amp adapter is a stub — returning empty sessions for {}",
            self.config_path.display()
        );
        Ok(Vec::new())
    }

    async fn health_check(&self) -> Result<bool> {
        Ok(self.is_installed() && self.connected)
    }
}

impl AdapterCore for AmpAdapter {
    fn adapter_type_name(&self) -> &'static str {
        "amp"
    }

    fn supported_models(&self) -> Vec<String> {
        // Amp routes through whatever model the user has configured
        // upstream. We don't yet know the discoverable list, so we
        // surface nothing here. Cost estimation in the rare case we
        // do see a model name falls back to Sonnet 3 rates below.
        Vec::new()
    }

    fn estimate_cost_for_model(&self, _model: &str, input_tokens: u64, output_tokens: u64) -> f64 {
        // Conservative fallback: bill at Claude Sonnet rates ($3/$15
        // per 1M) so a stub session that sneaks through doesn't show
        // as free. Replace once Amp's pricing surface is documented.
        let input = input_tokens as f64 / 1_000_000.0 * 3.0;
        let output = output_tokens as f64 / 1_000_000.0 * 15.0;
        input + output
    }
}
