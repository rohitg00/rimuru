pub mod claude_code;
pub mod codex;
pub mod copilot;
pub mod cursor;
pub mod detection;
pub mod goose;
pub mod opencode;

use async_trait::async_trait;
use serde_json::Value;

use crate::error::RimuruError;
use crate::models::{Agent, AgentType, Session};

type Result<T> = std::result::Result<T, RimuruError>;

#[async_trait]
pub trait AgentAdapter: Send + Sync {
    fn agent_type(&self) -> AgentType;
    fn is_installed(&self) -> bool;
    fn detect_version(&self) -> Option<String> {
        None
    }
    async fn connect(&mut self) -> Result<()>;
    async fn disconnect(&mut self) -> Result<()>;
    async fn get_status(&self) -> Result<Value>;
    async fn get_info(&self) -> Result<Agent>;
    async fn get_sessions(&self) -> Result<Vec<Session>>;
    async fn health_check(&self) -> Result<bool>;
}

#[async_trait]
pub trait CostTracker: Send + Sync {
    async fn get_usage(&self) -> Result<Value>;
    async fn calculate_cost(&self, model: &str, input_tokens: u64, output_tokens: u64) -> Result<f64>;
    fn get_supported_models(&self) -> Vec<String>;
    async fn get_total_cost(&self) -> Result<f64>;
}

#[async_trait]
pub trait SessionMonitor: Send + Sync {
    async fn get_session_history(&self) -> Result<Vec<Session>>;
    async fn get_session_details(&self, session_id: &str) -> Result<Option<Session>>;
    async fn get_active_sessions(&self) -> Result<Vec<Session>>;
}

pub trait AdapterCore: AgentAdapter {
    fn adapter_type_name(&self) -> &'static str;
    fn supported_models(&self) -> Vec<String>;
    fn estimate_cost_for_model(&self, model: &str, input_tokens: u64, output_tokens: u64) -> f64;
}

#[async_trait]
impl<T: AdapterCore> CostTracker for T {
    async fn get_usage(&self) -> Result<Value> {
        let sessions = self.get_sessions().await?;
        let total_input: u64 = sessions.iter().map(|s| s.input_tokens).sum();
        let total_output: u64 = sessions.iter().map(|s| s.output_tokens).sum();
        let total_cost: f64 = sessions.iter().map(|s| s.total_cost).sum();
        Ok(serde_json::json!({
            "agent_type": self.adapter_type_name(),
            "total_sessions": sessions.len(),
            "total_input_tokens": total_input,
            "total_output_tokens": total_output,
            "total_tokens": total_input + total_output,
            "estimated_total_cost": total_cost,
        }))
    }

    async fn calculate_cost(&self, model: &str, input_tokens: u64, output_tokens: u64) -> Result<f64> {
        Ok(self.estimate_cost_for_model(model, input_tokens, output_tokens))
    }

    fn get_supported_models(&self) -> Vec<String> {
        self.supported_models()
    }

    async fn get_total_cost(&self) -> Result<f64> {
        let sessions = self.get_sessions().await?;
        Ok(sessions.iter().map(|s| s.total_cost).sum())
    }
}

#[async_trait]
impl<T: AdapterCore> SessionMonitor for T {
    async fn get_session_history(&self) -> Result<Vec<Session>> {
        self.get_sessions().await
    }

    async fn get_session_details(&self, session_id: &str) -> Result<Option<Session>> {
        let sessions = self.get_sessions().await?;
        let target = uuid::Uuid::parse_str(session_id)
            .map_err(|e| RimuruError::Validation(format!("Invalid session ID: {}", e)))?;
        Ok(sessions.into_iter().find(|s| s.id == target))
    }

    async fn get_active_sessions(&self) -> Result<Vec<Session>> {
        let sessions = self.get_sessions().await?;
        Ok(sessions
            .into_iter()
            .filter(|s| matches!(s.status, crate::models::SessionStatus::Active))
            .collect())
    }
}

pub use claude_code::ClaudeCodeAdapter;
pub use codex::CodexAdapter;
pub use copilot::CopilotAdapter;
pub use cursor::CursorAdapter;
pub use detection::{detect_agent_config_path, detect_all_with_paths, detect_installed_agents};
pub use goose::GooseAdapter;
pub use opencode::OpenCodeAdapter;
