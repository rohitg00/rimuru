use serde_json::{Value, json};
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub async fn get_system_metrics(state: State<'_, AppState>) -> Result<Value, String> {
    state
        .call_extract("rimuru.metrics.current", json!({}), "metrics")
        .await
}

#[tauri::command]
pub async fn get_metrics_history(state: State<'_, AppState>) -> Result<Value, String> {
    state.call("rimuru.metrics.history", json!({})).await
}

#[tauri::command]
pub async fn get_hardware_info(state: State<'_, AppState>) -> Result<Value, String> {
    state
        .call_extract("rimuru.hardware.get", json!({}), "hardware")
        .await
}

#[tauri::command]
pub async fn detect_hardware(state: State<'_, AppState>) -> Result<Value, String> {
    state
        .call_extract("rimuru.hardware.detect", json!({}), "hardware")
        .await
}

#[tauri::command]
pub async fn get_model_advisor(state: State<'_, AppState>) -> Result<Value, String> {
    state
        .call_extract("rimuru.advisor.assess", json!({}), "advisories")
        .await
}
