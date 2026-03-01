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

pub use claude_code::ClaudeCodeAdapter;
pub use codex::CodexAdapter;
pub use copilot::CopilotAdapter;
pub use cursor::CursorAdapter;
pub use detection::{detect_agent_config_path, detect_all_with_paths, detect_installed_agents};
pub use goose::GooseAdapter;
pub use opencode::OpenCodeAdapter;
