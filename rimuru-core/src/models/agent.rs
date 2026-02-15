use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "agent_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum AgentType {
    ClaudeCode,
    Codex,
    Copilot,
    Goose,
    OpenCode,
    Cursor,
}

impl std::fmt::Display for AgentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentType::ClaudeCode => write!(f, "claude_code"),
            AgentType::Codex => write!(f, "codex"),
            AgentType::Copilot => write!(f, "copilot"),
            AgentType::Goose => write!(f, "goose"),
            AgentType::OpenCode => write!(f, "open_code"),
            AgentType::Cursor => write!(f, "cursor"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Agent {
    pub id: Uuid,
    pub name: String,
    pub agent_type: AgentType,
    pub config: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Agent {
    pub fn new(name: String, agent_type: AgentType, config: serde_json::Value) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            agent_type,
            config,
            created_at: now,
            updated_at: now,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_type_display() {
        assert_eq!(AgentType::ClaudeCode.to_string(), "claude_code");
        assert_eq!(AgentType::Codex.to_string(), "codex");
        assert_eq!(AgentType::Copilot.to_string(), "copilot");
        assert_eq!(AgentType::Goose.to_string(), "goose");
        assert_eq!(AgentType::OpenCode.to_string(), "open_code");
        assert_eq!(AgentType::Cursor.to_string(), "cursor");
    }

    #[test]
    fn test_agent_new() {
        let config = serde_json::json!({"key": "value"});
        let agent = Agent::new(
            "test-agent".to_string(),
            AgentType::ClaudeCode,
            config.clone(),
        );

        assert_eq!(agent.name, "test-agent");
        assert_eq!(agent.agent_type, AgentType::ClaudeCode);
        assert_eq!(agent.config, config);
    }
}
