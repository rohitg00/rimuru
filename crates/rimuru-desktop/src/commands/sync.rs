use chrono::Utc;
use serde_json::{json, Value};
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub async fn trigger_sync(state: State<'_, AppState>) -> Result<Value, String> {
    let agents_result = state.call("rimuru.agents.sync", json!({})).await?;
    let models_result = state.call("rimuru.models.sync", json!({})).await?;

    let now = Utc::now();
    *state.last_sync.write().await = Some(now);

    Ok(json!({
        "agents": agents_result,
        "models": models_result,
        "synced_at": now.to_rfc3339()
    }))
}

#[tauri::command]
pub async fn get_sync_status(state: State<'_, AppState>) -> Result<Value, String> {
    let last = state.last_sync.read().await;
    Ok(json!({
        "last_sync": last.map(|t| t.to_rfc3339())
    }))
}
