use serde_json::{Value, json};
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub async fn list_plugins(state: State<'_, AppState>) -> Result<Value, String> {
    state
        .call_extract("rimuru.plugins.list", json!({}), "plugins")
        .await
}

#[tauri::command]
pub async fn install_plugin(state: State<'_, AppState>, body: Value) -> Result<Value, String> {
    state.call("rimuru.plugins.install", body).await
}

#[tauri::command]
pub async fn uninstall_plugin(state: State<'_, AppState>, id: String) -> Result<Value, String> {
    state
        .call("rimuru.plugins.uninstall", json!({ "id": id }))
        .await
}

#[tauri::command]
pub async fn start_plugin(state: State<'_, AppState>, id: String) -> Result<Value, String> {
    state
        .call("rimuru.plugins.start", json!({ "id": id }))
        .await
}

#[tauri::command]
pub async fn stop_plugin(state: State<'_, AppState>, id: String) -> Result<Value, String> {
    state.call("rimuru.plugins.stop", json!({ "id": id })).await
}
