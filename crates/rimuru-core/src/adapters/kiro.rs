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

/// Kiro (AWS) adapter — installation + version detection only.
///
/// Same shape as the Amp stub: detect the install, surface the
/// version, return empty sessions with a tracing::warn until we have
/// a real on-disk format to parse. Kiro ships as a JetBrains/VS Code
/// extension, so the config tree lives under the IDE's plugin storage
/// rather than a top-level dotfile — paths below cover both common
/// install layouts.
pub struct KiroAdapter {
    config_path: PathBuf,
    connected: bool,
    agent_id: Uuid,
}

impl KiroAdapter {
    pub fn new() -> Self {
        // No home dir → empty base; joined candidates won't match
        // any real path, so is_installed() correctly reports false.
        let home = dirs::home_dir().unwrap_or_default();
        let candidates = [
            home.join(".kiro"),
            home.join(".config/kiro"),
            home.join(".aws/kiro"),
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

    fn detect_cli_version(&self) -> Option<String> {
        let output = std::process::Command::new("kiro")
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

impl Default for KiroAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AgentAdapter for KiroAdapter {
    fn agent_type(&self) -> AgentType {
        AgentType::Kiro
    }

    fn is_installed(&self) -> bool {
        self.config_path.exists() || binary_on_path(&["kiro"])
    }

    fn detect_version(&self) -> Option<String> {
        self.detect_cli_version()
    }

    async fn connect(&mut self) -> Result<()> {
        if !self.is_installed() {
            return Err(RimuruError::Adapter(format!(
                "Kiro is not installed ({} not found and `kiro` not on PATH)",
                self.config_path.display()
            )));
        }
        self.connected = true;
        debug!("Connected to Kiro adapter (stub — no session parsing)");
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.connected = false;
        Ok(())
    }

    async fn get_status(&self) -> Result<Value> {
        Ok(serde_json::json!({
            "agent_type": "kiro",
            "installed": self.is_installed(),
            "connected": self.connected,
            "config_path": self.config_path.to_string_lossy(),
            "version": self.detect_cli_version(),
            "session_parsing": "stub",
        }))
    }

    async fn get_info(&self) -> Result<Agent> {
        let mut agent = Agent::new(AgentType::Kiro, "Kiro".into());
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
            "note": "Kiro session format not yet supported — sessions list returns empty",
        });
        Ok(agent)
    }

    async fn get_sessions(&self) -> Result<Vec<Session>> {
        warn!(
            "Kiro adapter is a stub — returning empty sessions for {}",
            self.config_path.display()
        );
        Ok(Vec::new())
    }

    async fn health_check(&self) -> Result<bool> {
        Ok(self.is_installed() && self.connected)
    }
}

impl AdapterCore for KiroAdapter {
    fn adapter_type_name(&self) -> &'static str {
        "kiro"
    }

    fn supported_models(&self) -> Vec<String> {
        // Kiro proxies to Bedrock; the actual model list is dynamic
        // and customer-specific. Until we can pull it from the
        // user's account we surface nothing here.
        Vec::new()
    }

    fn estimate_cost_for_model(&self, model: &str, input_tokens: u64, output_tokens: u64) -> f64 {
        // Best-effort fallback: if the model hint matches a known
        // Bedrock-hosted Claude tier we use those rates; otherwise
        // fall through to mid-tier Sonnet pricing so a stray session
        // doesn't appear free.
        let (input_rate, output_rate) = match model {
            m if m.contains("opus") => (15.0, 75.0),
            m if m.contains("haiku") => (0.80, 4.0),
            _ => (3.0, 15.0),
        };
        let input = input_tokens as f64 / 1_000_000.0 * input_rate;
        let output = output_tokens as f64 / 1_000_000.0 * output_rate;
        input + output
    }
}
