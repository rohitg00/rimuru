use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum HookEvent {
    AgentConnected { agent_id: Uuid, agent_type: String },
    AgentDisconnected { agent_id: Uuid, agent_type: String },
    SessionStarted { session_id: Uuid, agent_id: Uuid },
    SessionEnded { session_id: Uuid, agent_id: Uuid, duration_secs: u64 },
    CostRecorded { record_id: Uuid, agent_id: Uuid, amount: f64 },
    ModelSynced { provider: String, model_count: usize },
    MetricsCollected { timestamp: String },
    PluginInstalled { plugin_id: String, name: String },
    PluginUninstalled { plugin_id: String },
    ThresholdExceeded { metric: String, value: f64, threshold: f64 },
    HealthCheckFailed { component: String, error: String },
}

impl HookEvent {
    pub fn event_type(&self) -> &'static str {
        match self {
            Self::AgentConnected { .. } => "agent_connected",
            Self::AgentDisconnected { .. } => "agent_disconnected",
            Self::SessionStarted { .. } => "session_started",
            Self::SessionEnded { .. } => "session_ended",
            Self::CostRecorded { .. } => "cost_recorded",
            Self::ModelSynced { .. } => "model_synced",
            Self::MetricsCollected { .. } => "metrics_collected",
            Self::PluginInstalled { .. } => "plugin_installed",
            Self::PluginUninstalled { .. } => "plugin_uninstalled",
            Self::ThresholdExceeded { .. } => "threshold_exceeded",
            Self::HealthCheckFailed { .. } => "health_check_failed",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookConfig {
    pub id: String,
    pub event_type: String,
    pub function_id: String,
    pub priority: i32,
    pub enabled: bool,
    pub metadata: Value,
}
