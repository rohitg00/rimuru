use serde_json::{Value, json};
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub async fn list_sessions(
    state: State<'_, AppState>,
    agent_id: Option<String>,
    status: Option<String>,
) -> Result<Value, String> {
    let mut input = json!({});
    if let Some(a) = agent_id {
        input["agent_id"] = json!(a);
    }
    if let Some(s) = status {
        input["status"] = json!(s);
    }
    state
        .call_extract("rimuru.sessions.list", input, "sessions")
        .await
}

#[tauri::command]
pub async fn get_session(state: State<'_, AppState>, session_id: String) -> Result<Value, String> {
    state
        .call("rimuru.sessions.get", json!({ "session_id": session_id }))
        .await
}

#[tauri::command]
pub async fn get_active_sessions(state: State<'_, AppState>) -> Result<Value, String> {
    state.call("rimuru.sessions.active", json!({})).await
}

#[tauri::command]
pub async fn get_session_history(state: State<'_, AppState>) -> Result<Value, String> {
    state.call("rimuru.sessions.history", json!({})).await
}
