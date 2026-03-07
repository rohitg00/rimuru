use serde_json::{json, Value};
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub async fn export_costs(state: State<'_, AppState>, format: String) -> Result<Value, String> {
    let _ = format;
    state.call("rimuru.costs.summary", json!({})).await
}

#[tauri::command]
pub async fn export_sessions(state: State<'_, AppState>) -> Result<Value, String> {
    state
        .call_extract("rimuru.sessions.list", json!({}), "sessions")
        .await
}

#[tauri::command]
pub async fn export_agents(state: State<'_, AppState>) -> Result<Value, String> {
    state
        .call_extract("rimuru.agents.list", json!({}), "agents")
        .await
}

#[tauri::command]
pub fn open_external(url: String) -> Result<(), String> {
    open::that(&url).map_err(|e| e.to_string())
}
