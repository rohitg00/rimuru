use serde_json::{json, Value};
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub async fn list_agents(state: State<'_, AppState>) -> Result<Value, String> {
    state
        .call_extract("rimuru.agents.list", json!({}), "agents")
        .await
}

#[tauri::command]
pub async fn get_agent(state: State<'_, AppState>, agent_id: String) -> Result<Value, String> {
    state
        .call("rimuru.agents.get", json!({ "agent_id": agent_id }))
        .await
}

#[tauri::command]
pub async fn register_agent(state: State<'_, AppState>, body: Value) -> Result<Value, String> {
    state.call("rimuru.agents.create", body).await
}

#[tauri::command]
pub async fn unregister_agent(
    state: State<'_, AppState>,
    agent_id: String,
) -> Result<Value, String> {
    state
        .call("rimuru.agents.delete", json!({ "agent_id": agent_id }))
        .await
}

#[tauri::command]
pub async fn detect_agents(state: State<'_, AppState>) -> Result<Value, String> {
    state
        .call("rimuru.agents.detect", json!({ "auto_register": true }))
        .await
}

#[tauri::command]
pub async fn connect_agent(
    state: State<'_, AppState>,
    agent_type: String,
) -> Result<Value, String> {
    state
        .call("rimuru.agents.connect", json!({ "agent_type": agent_type }))
        .await
}

#[tauri::command]
pub async fn disconnect_agent(
    state: State<'_, AppState>,
    agent_id: String,
) -> Result<Value, String> {
    state
        .call("rimuru.agents.disconnect", json!({ "agent_id": agent_id }))
        .await
}

#[tauri::command]
pub async fn sync_agents(state: State<'_, AppState>) -> Result<Value, String> {
    state.call("rimuru.agents.sync", json!({})).await
}
