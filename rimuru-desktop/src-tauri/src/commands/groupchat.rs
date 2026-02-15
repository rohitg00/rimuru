use base64::Engine;
use serde::Deserialize;
use tauri::State;

use crate::groupchat::types::{ChatAgent, ChatMessage, ChatRoom};
use crate::pty::manager::PtySessionManager;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct ChatAgentConfig {
    pub agent_type: String,
    pub name: String,
    pub role: String,
}

#[tauri::command]
pub async fn create_chat_room(
    name: String,
    agents: Vec<ChatAgentConfig>,
    state: State<'_, AppState>,
) -> Result<ChatRoom, String> {
    let chat_agents: Vec<ChatAgent> = agents
        .into_iter()
        .map(|a| ChatAgent {
            agent_type: a.agent_type,
            name: a.name,
            role: a.role,
            session_id: None,
        })
        .collect();
    Ok(state.chat_manager.create_room(name, chat_agents))
}

#[tauri::command]
pub async fn send_chat_message(
    room_id: String,
    content: String,
    state: State<'_, AppState>,
    pty_state: State<'_, std::sync::Arc<PtySessionManager>>,
) -> Result<ChatMessage, String> {
    let message = state.chat_manager.add_message(
        &room_id,
        "user".to_string(),
        content.clone(),
        "user".to_string(),
    )?;

    let room = state
        .chat_manager
        .get_room(&room_id)
        .ok_or_else(|| "Room not found".to_string())?;

    for agent in &room.agents {
        if let Some(session_id) = &agent.session_id {
            let payload = format!("{}\n", content);
            let encoded = base64::engine::general_purpose::STANDARD.encode(payload.as_bytes());
            let data = base64::engine::general_purpose::STANDARD
                .decode(&encoded)
                .map_err(|e| format!("Encode error: {}", e))?;
            let _ = pty_state.write(session_id, &data);
        }
    }

    Ok(message)
}

#[tauri::command]
pub async fn get_chat_messages(
    room_id: String,
    state: State<'_, AppState>,
) -> Result<Vec<ChatMessage>, String> {
    Ok(state.chat_manager.get_messages(&room_id))
}

#[tauri::command]
pub async fn list_chat_rooms(state: State<'_, AppState>) -> Result<Vec<ChatRoom>, String> {
    Ok(state.chat_manager.list_rooms())
}

#[tauri::command]
pub async fn close_chat_room(room_id: String, state: State<'_, AppState>) -> Result<(), String> {
    state.chat_manager.close_room(&room_id)
}
