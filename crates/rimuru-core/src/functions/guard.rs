use iii_sdk::{III, RegisterFunctionMessage};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::sysutil::{api_response, extract_input, kv_err, require_str};
use crate::state::StateKV;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GuardRecord {
    id: String,
    command: String,
    limit: f64,
    action: String,
    started_at: String,
    current_cost: f64,
    #[serde(default)]
    pid: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GuardHistoryRecord {
    id: String,
    command: String,
    limit: f64,
    final_cost: f64,
    action_taken: String,
    started_at: String,
    ended_at: String,
}

pub fn register(iii: &III, kv: &StateKV) {
    register_register(iii, kv);
    register_complete(iii, kv);
    register_list(iii, kv);
    register_history(iii, kv);
}

fn register_register(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function_with(
        RegisterFunctionMessage::with_id("rimuru.guard.register".to_string()),
        move |input: Value| {
            let kv = kv.clone();
            async move {
                let input = extract_input(input);
                let id = require_str(&input, "id")?;
                let command = require_str(&input, "command")?;
                let limit = input.get("limit").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let action = input
                    .get("action")
                    .and_then(|v| v.as_str())
                    .unwrap_or("warn")
                    .to_string();
                let started_at = require_str(&input, "started_at")?;
                let pid = input.get("pid").and_then(|v| v.as_i64()).unwrap_or(0);

                let record = GuardRecord {
                    id: id.clone(),
                    command,
                    limit,
                    action,
                    started_at,
                    current_cost: 0.0,
                    pid,
                };

                kv.set("guards", &id, &record).await.map_err(kv_err)?;

                Ok(api_response(json!({
                    "guard": record,
                    "registered": true
                })))
            }
        },
    );
}

fn register_complete(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function_with(
        RegisterFunctionMessage::with_id("rimuru.guard.complete".to_string()),
        move |input: Value| {
            let kv = kv.clone();
            async move {
                let input = extract_input(input);
                let id = require_str(&input, "id")?;
                let final_cost = input
                    .get("final_cost")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                let action_taken = input
                    .get("action_taken")
                    .and_then(|v| v.as_str())
                    .unwrap_or("none")
                    .to_string();
                let ended_at = require_str(&input, "ended_at")?;

                let guard: Option<GuardRecord> = kv.get("guards", &id).await.map_err(kv_err)?;

                let guard = guard.ok_or_else(|| {
                    iii_sdk::IIIError::Handler(format!("guard not found: {}", id))
                })?;

                let history = GuardHistoryRecord {
                    id: id.clone(),
                    command: guard.command,
                    limit: guard.limit,
                    final_cost,
                    action_taken,
                    started_at: guard.started_at,
                    ended_at,
                };

                kv.set("guard_history", &id, &history)
                    .await
                    .map_err(kv_err)?;
                kv.delete("guards", &id).await.map_err(kv_err)?;

                Ok(api_response(json!({
                    "history": history,
                    "completed": true
                })))
            }
        },
    );
}

fn register_list(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function_with(
        RegisterFunctionMessage::with_id("rimuru.guard.list".to_string()),
        move |_input: Value| {
            let kv = kv.clone();
            async move {
                let guards: Vec<GuardRecord> = kv.list("guards").await.map_err(kv_err)?;

                Ok(api_response(json!({
                    "guards": guards,
                    "total": guards.len()
                })))
            }
        },
    );
}

fn register_history(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function_with(
        RegisterFunctionMessage::with_id("rimuru.guard.history".to_string()),
        move |_input: Value| {
            let kv = kv.clone();
            async move {
                let history: Vec<GuardHistoryRecord> =
                    kv.list("guard_history").await.map_err(kv_err)?;

                Ok(api_response(json!({
                    "history": history,
                    "total": history.len()
                })))
            }
        },
    );
}
