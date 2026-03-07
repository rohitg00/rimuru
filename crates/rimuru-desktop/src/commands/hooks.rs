use serde_json::{json, Value};
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub async fn list_hooks(state: State<'_, AppState>) -> Result<Value, String> {
    state
        .call_extract("rimuru.hooks.list", json!({}), "hooks")
        .await
}

#[tauri::command]
pub async fn register_hook(state: State<'_, AppState>, body: Value) -> Result<Value, String> {
    state.call("rimuru.hooks.register", body).await
}

#[tauri::command]
pub async fn dispatch_hook(state: State<'_, AppState>, body: Value) -> Result<Value, String> {
    state.call("rimuru.hooks.dispatch", body).await
}

#[tauri::command]
pub async fn delete_hook(state: State<'_, AppState>, hook_id: String) -> Result<Value, String> {
    state
        .call("rimuru.hooks.delete", json!({ "hook_id": hook_id }))
        .await
}
