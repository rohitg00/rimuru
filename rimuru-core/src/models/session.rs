use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "session_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    Active,
    Completed,
    Failed,
    Terminated,
}

impl std::fmt::Display for SessionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionStatus::Active => write!(f, "active"),
            SessionStatus::Completed => write!(f, "completed"),
            SessionStatus::Failed => write!(f, "failed"),
            SessionStatus::Terminated => write!(f, "terminated"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Session {
    pub id: Uuid,
    pub agent_id: Uuid,
    pub status: SessionStatus,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub metadata: serde_json::Value,
}

impl Session {
    pub fn new(agent_id: Uuid, metadata: serde_json::Value) -> Self {
        Self {
            id: Uuid::new_v4(),
            agent_id,
            status: SessionStatus::Active,
            started_at: Utc::now(),
            ended_at: None,
            metadata,
        }
    }

    pub fn end(&mut self, status: SessionStatus) {
        self.status = status;
        self.ended_at = Some(Utc::now());
    }

    pub fn is_active(&self) -> bool {
        self.status == SessionStatus::Active
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
    fn test_session_status_display() {
        assert_eq!(SessionStatus::Active.to_string(), "active");
        assert_eq!(SessionStatus::Completed.to_string(), "completed");
        assert_eq!(SessionStatus::Failed.to_string(), "failed");
        assert_eq!(SessionStatus::Terminated.to_string(), "terminated");
    }

    #[test]
    fn test_session_new() {
        let agent_id = Uuid::new_v4();
        let metadata = serde_json::json!({"project": "test"});
        let session = Session::new(agent_id, metadata.clone());

        assert_eq!(session.agent_id, agent_id);
        assert_eq!(session.status, SessionStatus::Active);
        assert!(session.ended_at.is_none());
        assert_eq!(session.metadata, metadata);
    }

    #[test]
    fn test_session_end() {
        let agent_id = Uuid::new_v4();
        let mut session = Session::new(agent_id, serde_json::json!({}));

        assert!(session.is_active());

        session.end(SessionStatus::Completed);

        assert!(!session.is_active());
        assert_eq!(session.status, SessionStatus::Completed);
        assert!(session.ended_at.is_some());
    }
}
