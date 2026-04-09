use serde_json::{Value, json};
use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub async fn get_cost_summary(
    state: State<'_, AppState>,
    since: Option<String>,
    until: Option<String>,
) -> Result<Value, String> {
    let mut input = json!({});
    if let Some(s) = since {
        input["since"] = json!(s);
    }
    if let Some(u) = until {
        input["until"] = json!(u);
    }
    state
        .call_extract("rimuru.costs.summary", input, "summary")
        .await
}

#[tauri::command]
pub async fn get_cost_breakdown(
    state: State<'_, AppState>,
    agent_id: String,
    days: Option<u64>,
) -> Result<Value, String> {
    let mut input = json!({ "agent_id": agent_id });
    if let Some(d) = days {
        input["days"] = json!(d);
    }
    state.call("rimuru.costs.by_agent", input).await
}

#[tauri::command]
pub async fn get_cost_history(
    state: State<'_, AppState>,
    days: Option<u64>,
) -> Result<Value, String> {
    let mut input = json!({});
    if let Some(d) = days {
        input["days"] = json!(d);
    }
    state
        .call_extract("rimuru.costs.daily", input, "daily")
        .await
}

#[tauri::command]
pub async fn record_cost(state: State<'_, AppState>, body: Value) -> Result<Value, String> {
    state.call("rimuru.costs.record", body).await
}

#[tauri::command]
pub async fn get_daily_rollup(
    state: State<'_, AppState>,
    date: Option<String>,
) -> Result<Value, String> {
    let mut input = json!({});
    if let Some(d) = date {
        input["date"] = json!(d);
    }
    state.call("rimuru.costs.daily_rollup", input).await
}
