use serde_json::{json, Value};
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<Value, String> {
    state
        .call_extract("rimuru.config.get", json!({}), "config")
        .await
}

#[tauri::command]
pub async fn update_setting(
    state: State<'_, AppState>,
    key: String,
    value: Value,
) -> Result<Value, String> {
    state
        .call("rimuru.config.set", json!({ "key": key, "value": value }))
        .await
}

#[tauri::command]
pub async fn get_health(state: State<'_, AppState>) -> Result<Value, String> {
    state.call("rimuru.health.check", json!({})).await
}

#[tauri::command]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[tauri::command]
pub fn get_port(state: State<'_, AppState>) -> u16 {
    state.api_port
}
