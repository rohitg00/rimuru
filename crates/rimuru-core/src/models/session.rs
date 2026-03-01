use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::AgentType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    Active,
    Completed,
    Abandoned,
    Error,
}

impl std::fmt::Display for SessionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "Active"),
            Self::Completed => write!(f, "Completed"),
            Self::Abandoned => write!(f, "Abandoned"),
            Self::Error => write!(f, "Error"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub agent_id: Uuid,
    pub agent_type: AgentType,
    pub status: SessionStatus,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub project_path: Option<String>,
    pub total_tokens: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub total_cost: f64,
    pub model: Option<String>,
    pub messages: u64,
    pub metadata: serde_json::Value,
}

impl Session {
    pub fn new(agent_id: Uuid, agent_type: AgentType) -> Self {
        Self {
            id: Uuid::new_v4(),
            agent_id,
            agent_type,
            status: SessionStatus::Active,
            started_at: Utc::now(),
            ended_at: None,
            project_path: None,
            total_tokens: 0,
            input_tokens: 0,
            output_tokens: 0,
            total_cost: 0.0,
            model: None,
            messages: 0,
            metadata: serde_json::json!({}),
        }
    }

    pub fn duration_secs(&self) -> Option<i64> {
        let end = self.ended_at.unwrap_or_else(Utc::now);
        Some((end - self.started_at).num_seconds())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionFilter {
    pub agent_id: Option<Uuid>,
    pub agent_type: Option<AgentType>,
    pub status: Option<SessionStatus>,
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
    pub limit: Option<usize>,
}

impl Default for SessionFilter {
    fn default() -> Self {
        Self {
            agent_id: None,
            agent_type: None,
            status: None,
            since: None,
            until: None,
            limit: Some(100),
        }
    }
}
