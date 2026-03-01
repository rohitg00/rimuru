use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentType {
    ClaudeCode,
    Cursor,
    Copilot,
    Codex,
    Goose,
    OpenCode,
}

impl AgentType {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::ClaudeCode => "Claude Code",
            Self::Cursor => "Cursor",
            Self::Copilot => "GitHub Copilot",
            Self::Codex => "Codex",
            Self::Goose => "Goose",
            Self::OpenCode => "OpenCode",
        }
    }

    pub fn all() -> &'static [AgentType] {
        &[
            Self::ClaudeCode,
            Self::Cursor,
            Self::Copilot,
            Self::Codex,
            Self::Goose,
            Self::OpenCode,
        ]
    }
}

impl std::fmt::Display for AgentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentStatus {
    Connected,
    Disconnected,
    Active,
    Idle,
    Error,
}

impl std::fmt::Display for AgentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Connected => write!(f, "Connected"),
            Self::Disconnected => write!(f, "Disconnected"),
            Self::Active => write!(f, "Active"),
            Self::Idle => write!(f, "Idle"),
            Self::Error => write!(f, "Error"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: Uuid,
    pub agent_type: AgentType,
    pub name: String,
    pub status: AgentStatus,
    pub version: Option<String>,
    pub config_path: Option<String>,
    pub connected_at: Option<DateTime<Utc>>,
    pub last_seen: Option<DateTime<Utc>>,
    pub session_count: u64,
    pub total_cost: f64,
    pub metadata: serde_json::Value,
}

impl Agent {
    pub fn new(agent_type: AgentType, name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            agent_type,
            name,
            status: AgentStatus::Disconnected,
            version: None,
            config_path: None,
            connected_at: None,
            last_seen: None,
            session_count: 0,
            total_cost: 0.0,
            metadata: serde_json::json!({}),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub agent_id: Uuid,
    pub poll_interval_secs: u64,
    pub cost_tracking_enabled: bool,
    pub session_monitoring_enabled: bool,
    pub custom_config: serde_json::Value,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            agent_id: Uuid::nil(),
            poll_interval_secs: 30,
            cost_tracking_enabled: true,
            session_monitoring_enabled: true,
            custom_config: serde_json::json!({}),
        }
    }
}
