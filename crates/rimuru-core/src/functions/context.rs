use iii_sdk::{III, RegisterFunctionMessage};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::sysutil::{kv_err, require_str};
use crate::adapters::ClaudeCodeAdapter;
use crate::models::{AgentType, ContextBreakdown, ContextUtilization, Session};
use crate::state::StateKV;

pub fn register(iii: &III, kv: &StateKV) {
    register_breakdown(iii, kv);
    register_breakdown_by_session(iii, kv);
    register_utilization(iii, kv);
    register_waste(iii, kv);
}

fn register_breakdown(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function_with(
        RegisterFunctionMessage::with_id("rimuru.context.breakdown".to_string()),
        move |input: Value| {
            let kv = kv.clone();
            async move {
                let session_id = require_str(&input, "session_id")?;

                let cached: Option<ContextBreakdown> = kv
                    .get("context_breakdowns", &session_id)
                    .await
                    .map_err(kv_err)?;

                if let Some(breakdown) = cached {
                    return Ok(serde_json::to_value(breakdown).unwrap_or_default());
                }

                let session: Session = kv
                    .get("sessions", &session_id)
                    .await
                    .map_err(kv_err)?
                    .ok_or_else(|| {
                        iii_sdk::IIIError::Handler(format!("Session not found: {}", session_id))
                    })?;

                if session.agent_type == AgentType::ClaudeCode {
                    let adapter = ClaudeCodeAdapter::new();
                    let session_files = adapter.find_session_file(&session_id);

                    if let Some(path) = session_files {
                        let (parsed_session, breakdown) =
                            adapter.parse_session_jsonl_full(&path).map_err(|e| {
                                iii_sdk::IIIError::Handler(format!("Parse error: {}", e))
                            })?;

                        if let Err(e) = kv.set("sessions", &session_id, &parsed_session).await {
                            tracing::warn!("Failed to persist parsed session: {}", e);
                        }

                        if let Err(e) = kv.set("context_breakdowns", &session_id, &breakdown).await
                        {
                            tracing::warn!("Failed to cache breakdown: {}", e);
                        }

                        return Ok(serde_json::to_value(breakdown).unwrap_or_default());
                    }
                }

                Ok(json!({"error": "No breakdown available for this session type"}))
            }
        },
    );
}

fn register_breakdown_by_session(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function_with(
        RegisterFunctionMessage::with_id("rimuru.context.breakdown_by_session".to_string()),
        move |_input: Value| {
            let kv = kv.clone();
            async move {
                let breakdowns: Vec<ContextBreakdown> =
                    kv.list("context_breakdowns").await.map_err(kv_err)?;

                Ok(json!({
                    "breakdowns": breakdowns,
                    "total": breakdowns.len()
                }))
            }
        },
    );
}

fn register_utilization(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function_with(
        RegisterFunctionMessage::with_id("rimuru.context.utilization".to_string()),
        move |_input: Value| {
            let kv = kv.clone();
            async move {
                let sessions: Vec<Session> = kv.list("sessions").await.map_err(kv_err)?;
                let active: Vec<&Session> = sessions
                    .iter()
                    .filter(|s| matches!(s.status, crate::models::SessionStatus::Active))
                    .collect();

                let threshold: f64 = 80.0;

                let utilizations: Vec<ContextUtilization> = active
                    .iter()
                    .map(|s| {
                        let model = s.model.clone().unwrap_or_else(|| "unknown".to_string());
                        let ctx_size = model_context_window(&model);

                        let used = last_turn_input_tokens(&s.metadata).unwrap_or(s.input_tokens);

                        let pct = if ctx_size > 0 {
                            (used as f64 / ctx_size as f64) * 100.0
                        } else {
                            0.0
                        };

                        ContextUtilization {
                            session_id: s.id,
                            model: model.clone(),
                            context_window_size: ctx_size,
                            tokens_used: used,
                            utilization_percent: pct,
                            is_near_limit: pct >= threshold,
                        }
                    })
                    .collect();

                Ok(json!({
                    "utilizations": utilizations,
                    "total_active": active.len(),
                    "threshold_percent": threshold
                }))
            }
        },
    );
}

fn register_waste(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function_with(
        RegisterFunctionMessage::with_id("rimuru.context.waste".to_string()),
        move |_input: Value| {
            let kv = kv.clone();
            async move {
                let breakdowns: Vec<ContextBreakdown> =
                    kv.list("context_breakdowns").await.map_err(kv_err)?;

                let mut waste_reports: Vec<WasteReport> = breakdowns
                    .iter()
                    .filter(|b| b.total_tokens > 0)
                    .map(|b| {
                        let potential_savings = b.tool_schema_tokens + b.bash_output_tokens;
                        WasteReport {
                            session_id: b.session_id,
                            total_tokens: b.total_tokens,
                            tool_schema_tokens: b.tool_schema_tokens,
                            bash_output_tokens: b.bash_output_tokens,
                            mcp_tokens: b.mcp_tokens,
                            waste_percent: b.waste_percent(),
                            potential_savings,
                        }
                    })
                    .collect();

                waste_reports.sort_by(|a, b| {
                    b.waste_percent
                        .partial_cmp(&a.waste_percent)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });

                let total_waste: u64 = waste_reports.iter().map(|r| r.potential_savings).sum();

                Ok(json!({
                    "sessions": waste_reports,
                    "total_waste_tokens": total_waste,
                    "total_sessions_analyzed": breakdowns.len()
                }))
            }
        },
    );
}

fn last_turn_input_tokens(metadata: &Value) -> Option<u64> {
    metadata
        .get("turns")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.last())
        .and_then(|turn| turn.get("input_tokens"))
        .and_then(|v| v.as_u64())
}

fn model_context_window(model: &str) -> u64 {
    let models = super::models::hardcoded_models();
    models
        .iter()
        .find(|m| model.contains(&m.id))
        .map(|m| m.context_window)
        .unwrap_or(200_000)
}

#[derive(Debug, Serialize, Deserialize)]
struct WasteReport {
    session_id: uuid::Uuid,
    total_tokens: u64,
    tool_schema_tokens: u64,
    bash_output_tokens: u64,
    mcp_tokens: u64,
    waste_percent: f64,
    potential_savings: u64,
}
