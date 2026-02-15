use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::AgentType;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum AdapterStatus {
    Connected,
    Disconnected,
    Error,
    #[default]
    Unknown,
}

impl std::fmt::Display for AdapterStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AdapterStatus::Connected => write!(f, "connected"),
            AdapterStatus::Disconnected => write!(f, "disconnected"),
            AdapterStatus::Error => write!(f, "error"),
            AdapterStatus::Unknown => write!(f, "unknown"),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UsageStats {
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub requests: i64,
    pub model_name: Option<String>,
    pub period_start: Option<DateTime<Utc>>,
    pub period_end: Option<DateTime<Utc>>,
}

impl UsageStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn total_tokens(&self) -> i64 {
        self.input_tokens + self.output_tokens
    }

    pub fn add(&mut self, other: &UsageStats) {
        self.input_tokens += other.input_tokens;
        self.output_tokens += other.output_tokens;
        self.requests += other.requests;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionEvent {
    Started {
        session_id: Uuid,
        timestamp: DateTime<Utc>,
        metadata: serde_json::Value,
    },
    MessageSent {
        session_id: Uuid,
        timestamp: DateTime<Utc>,
        tokens: i64,
    },
    MessageReceived {
        session_id: Uuid,
        timestamp: DateTime<Utc>,
        tokens: i64,
    },
    Completed {
        session_id: Uuid,
        timestamp: DateTime<Utc>,
        total_tokens: i64,
    },
    Error {
        session_id: Uuid,
        timestamp: DateTime<Utc>,
        error: String,
    },
}

impl SessionEvent {
    pub fn session_id(&self) -> Uuid {
        match self {
            SessionEvent::Started { session_id, .. } => *session_id,
            SessionEvent::MessageSent { session_id, .. } => *session_id,
            SessionEvent::MessageReceived { session_id, .. } => *session_id,
            SessionEvent::Completed { session_id, .. } => *session_id,
            SessionEvent::Error { session_id, .. } => *session_id,
        }
    }

    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            SessionEvent::Started { timestamp, .. } => *timestamp,
            SessionEvent::MessageSent { timestamp, .. } => *timestamp,
            SessionEvent::MessageReceived { timestamp, .. } => *timestamp,
            SessionEvent::Completed { timestamp, .. } => *timestamp,
            SessionEvent::Error { timestamp, .. } => *timestamp,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterInfo {
    pub name: String,
    pub agent_type: AgentType,
    pub version: Option<String>,
    pub status: AdapterStatus,
    pub config_path: Option<String>,
    pub last_connected: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
}

impl AdapterInfo {
    pub fn new(name: String, agent_type: AgentType) -> Self {
        Self {
            name,
            agent_type,
            version: None,
            status: AdapterStatus::Unknown,
            config_path: None,
            last_connected: None,
            error_message: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveSession {
    pub session_id: Uuid,
    pub agent_type: AgentType,
    pub started_at: DateTime<Utc>,
    pub current_tokens: i64,
    pub model_name: Option<String>,
    pub project_path: Option<String>,
}

impl ActiveSession {
    pub fn new(session_id: Uuid, agent_type: AgentType) -> Self {
        Self {
            session_id,
            agent_type,
            started_at: Utc::now(),
            current_tokens: 0,
            model_name: None,
            project_path: None,
        }
    }

    pub fn duration_seconds(&self) -> i64 {
        (Utc::now() - self.started_at).num_seconds()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionHistory {
    pub session_id: Uuid,
    pub agent_type: AgentType,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub model_name: Option<String>,
    pub cost_usd: Option<f64>,
    pub project_path: Option<String>,
}

impl SessionHistory {
    pub fn total_tokens(&self) -> i64 {
        self.total_input_tokens + self.total_output_tokens
    }

    pub fn duration_seconds(&self) -> Option<i64> {
        self.ended_at
            .map(|end| (end - self.started_at).num_seconds())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adapter_status_display() {
        assert_eq!(AdapterStatus::Connected.to_string(), "connected");
        assert_eq!(AdapterStatus::Disconnected.to_string(), "disconnected");
        assert_eq!(AdapterStatus::Error.to_string(), "error");
        assert_eq!(AdapterStatus::Unknown.to_string(), "unknown");
    }

    #[test]
    fn test_usage_stats_total_tokens() {
        let stats = UsageStats {
            input_tokens: 1000,
            output_tokens: 500,
            requests: 10,
            model_name: Some("claude-3-opus".to_string()),
            period_start: None,
            period_end: None,
        };

        assert_eq!(stats.total_tokens(), 1500);
    }

    #[test]
    fn test_usage_stats_add() {
        let mut stats1 = UsageStats {
            input_tokens: 1000,
            output_tokens: 500,
            requests: 5,
            ..Default::default()
        };

        let stats2 = UsageStats {
            input_tokens: 2000,
            output_tokens: 1000,
            requests: 10,
            ..Default::default()
        };

        stats1.add(&stats2);

        assert_eq!(stats1.input_tokens, 3000);
        assert_eq!(stats1.output_tokens, 1500);
        assert_eq!(stats1.requests, 15);
    }

    #[test]
    fn test_session_event_accessors() {
        let session_id = Uuid::new_v4();
        let timestamp = Utc::now();

        let event = SessionEvent::Started {
            session_id,
            timestamp,
            metadata: serde_json::json!({}),
        };

        assert_eq!(event.session_id(), session_id);
        assert_eq!(event.timestamp(), timestamp);
    }

    #[test]
    fn test_adapter_info_new() {
        let info = AdapterInfo::new("test-adapter".to_string(), AgentType::ClaudeCode);

        assert_eq!(info.name, "test-adapter");
        assert_eq!(info.agent_type, AgentType::ClaudeCode);
        assert_eq!(info.status, AdapterStatus::Unknown);
        assert!(info.version.is_none());
    }

    #[test]
    fn test_active_session_new() {
        let session_id = Uuid::new_v4();
        let session = ActiveSession::new(session_id, AgentType::OpenCode);

        assert_eq!(session.session_id, session_id);
        assert_eq!(session.agent_type, AgentType::OpenCode);
        assert_eq!(session.current_tokens, 0);
    }

    #[test]
    fn test_session_history_total_tokens() {
        let history = SessionHistory {
            session_id: Uuid::new_v4(),
            agent_type: AgentType::ClaudeCode,
            started_at: Utc::now(),
            ended_at: Some(Utc::now()),
            total_input_tokens: 5000,
            total_output_tokens: 2500,
            model_name: Some("claude-3-sonnet".to_string()),
            cost_usd: Some(0.15),
            project_path: None,
        };

        assert_eq!(history.total_tokens(), 7500);
    }
}
