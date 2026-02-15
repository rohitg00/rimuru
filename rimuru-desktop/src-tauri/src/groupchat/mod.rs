pub mod types;

use std::collections::HashMap;

use parking_lot::RwLock;

use types::{ChatAgent, ChatMessage, ChatRoom};

pub struct ChatRoomManager {
    rooms: RwLock<HashMap<String, ChatRoom>>,
    messages: RwLock<HashMap<String, Vec<ChatMessage>>>,
}

impl ChatRoomManager {
    pub fn new() -> Self {
        Self {
            rooms: RwLock::new(HashMap::new()),
            messages: RwLock::new(HashMap::new()),
        }
    }

    pub fn create_room(&self, name: String, agents: Vec<ChatAgent>) -> ChatRoom {
        let room = ChatRoom {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            agents,
            created_at: chrono::Utc::now().to_rfc3339(),
        };
        self.rooms.write().insert(room.id.clone(), room.clone());
        self.messages.write().insert(room.id.clone(), Vec::new());
        room
    }

    pub fn add_message(
        &self,
        room_id: &str,
        sender: String,
        content: String,
        msg_type: String,
    ) -> Result<ChatMessage, String> {
        let rooms = self.rooms.read();
        if !rooms.contains_key(room_id) {
            return Err(format!("Room not found: {}", room_id));
        }
        drop(rooms);

        let message = ChatMessage {
            id: uuid::Uuid::new_v4().to_string(),
            room_id: room_id.to_string(),
            sender,
            content,
            timestamp: chrono::Utc::now().to_rfc3339(),
            message_type: msg_type,
        };

        self.messages
            .write()
            .entry(room_id.to_string())
            .or_default()
            .push(message.clone());

        Ok(message)
    }

    pub fn get_messages(&self, room_id: &str) -> Vec<ChatMessage> {
        self.messages
            .read()
            .get(room_id)
            .cloned()
            .unwrap_or_default()
    }

    pub fn get_room(&self, room_id: &str) -> Option<ChatRoom> {
        self.rooms.read().get(room_id).cloned()
    }

    pub fn list_rooms(&self) -> Vec<ChatRoom> {
        self.rooms.read().values().cloned().collect()
    }

    pub fn close_room(&self, room_id: &str) -> Result<(), String> {
        if self.rooms.write().remove(room_id).is_none() {
            return Err(format!("Room not found: {}", room_id));
        }
        self.messages.write().remove(room_id);
        Ok(())
    }

    pub fn update_agent_session(
        &self,
        room_id: &str,
        agent_name: &str,
        session_id: String,
    ) -> Result<(), String> {
        let mut rooms = self.rooms.write();
        let room = rooms
            .get_mut(room_id)
            .ok_or_else(|| format!("Room not found: {}", room_id))?;
        for agent in &mut room.agents {
            if agent.name == agent_name {
                agent.session_id = Some(session_id);
                return Ok(());
            }
        }
        Err(format!("Agent not found: {}", agent_name))
    }
}
