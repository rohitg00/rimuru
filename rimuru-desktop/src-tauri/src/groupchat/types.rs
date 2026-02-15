use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatRoom {
    pub id: String,
    pub name: String,
    pub agents: Vec<ChatAgent>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatAgent {
    pub agent_type: String,
    pub name: String,
    pub role: String,
    pub session_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub id: String,
    pub room_id: String,
    pub sender: String,
    pub content: String,
    pub timestamp: String,
    pub message_type: String,
}
