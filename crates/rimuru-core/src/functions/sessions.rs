use chrono::Utc;
use iii_sdk::{III, RegisterFunctionMessage};
use serde_json::{Value, json};
use uuid::Uuid;

use super::sysutil::{api_response, extract_input, kv_err, require_str};
use crate::models::{Agent, Session, SessionFilter, SessionStatus};
use crate::state::StateKV;

pub fn register(iii: &III, kv: &StateKV) {
    register_list(iii, kv);
    register_get(iii, kv);
    register_active(iii, kv);
    register_history(iii, kv);
    register_cleanup(iii, kv);
}

fn register_list(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function_with(
        RegisterFunctionMessage::with_id("rimuru.sessions.list".to_string()),
        move |input: Value| {
            let kv = kv.clone();
            async move {
                let input = extract_input(input);
                let sessions: Vec<Session> = kv.list("sessions").await.map_err(kv_err)?;

                let filter: SessionFilter = serde_json::from_value(input).unwrap_or_default();

                let filtered: Vec<&Session> = sessions
                    .iter()
                    .filter(|s| filter.agent_id.is_none_or(|id| s.agent_id == id))
                    .filter(|s| filter.agent_type.is_none_or(|t| s.agent_type == t))
                    .filter(|s| filter.status.is_none_or(|st| s.status == st))
                    .filter(|s| filter.since.is_none_or(|since| s.started_at >= since))
                    .filter(|s| filter.until.is_none_or(|until| s.started_at <= until))
                    .collect();

                let limit = filter.limit.unwrap_or(100);
                let result: Vec<&&Session> = filtered.iter().take(limit).collect();

                Ok(api_response(json!({
                    "sessions": result,
                    "total": filtered.len(),
                    "limit": limit
                })))
            }
        },
    );
}

fn register_get(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function_with(
        RegisterFunctionMessage::with_id("rimuru.sessions.get".to_string()),
        move |input: Value| {
            let kv = kv.clone();
            async move {
                let input = extract_input(input);
                let session_id = require_str(&input, "session_id")?;

                let session: Option<Session> =
                    kv.get("sessions", &session_id).await.map_err(kv_err)?;

                match session {
                    Some(s) => {
                        let duration = s.duration_secs();
                        Ok(api_response(json!({
                            "session": s,
                            "duration_secs": duration
                        })))
                    }
                    None => Err(iii_sdk::IIIError::Handler(format!(
                        "session not found: {}",
                        session_id
                    ))),
                }
            }
        },
    );
}

fn register_active(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function_with(
        RegisterFunctionMessage::with_id("rimuru.sessions.active".to_string()),
        move |_input: Value| {
            let kv = kv.clone();
            async move {
                let sessions: Vec<Session> = kv.list("sessions").await.map_err(kv_err)?;

                let active: Vec<&Session> = sessions
                    .iter()
                    .filter(|s| s.status == SessionStatus::Active)
                    .collect();

                let total_cost: f64 = active.iter().map(|s| s.total_cost).sum();
                let total_tokens: u64 = active.iter().map(|s| s.total_tokens).sum();

                let mut by_agent: Vec<Value> = Vec::new();
                let agents: Vec<Agent> = kv.list("agents").await.map_err(kv_err)?;

                for agent in &agents {
                    let agent_sessions: Vec<&&Session> =
                        active.iter().filter(|s| s.agent_id == agent.id).collect();

                    if !agent_sessions.is_empty() {
                        by_agent.push(json!({
                            "agent_id": agent.id,
                            "agent_name": agent.name,
                            "agent_type": agent.agent_type,
                            "active_sessions": agent_sessions.len(),
                            "total_cost": agent_sessions.iter().map(|s| s.total_cost).sum::<f64>()
                        }));
                    }
                }

                Ok(api_response(json!({
                    "active_sessions": active,
                    "total": active.len(),
                    "total_cost": total_cost,
                    "total_tokens": total_tokens,
                    "by_agent": by_agent
                })))
            }
        },
    );
}

fn register_history(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function_with(
        RegisterFunctionMessage::with_id("rimuru.sessions.history".to_string()),
        move |input: Value| {
            let kv = kv.clone();
            async move {
                let input = extract_input(input);
                let sessions: Vec<Session> = kv.list("sessions").await.map_err(kv_err)?;

                let agent_id = input
                    .get("agent_id")
                    .and_then(|v| v.as_str())
                    .and_then(|s| Uuid::parse_str(s).ok());

                let days = input.get("days").and_then(|v| v.as_u64()).unwrap_or(30);

                let cutoff = Utc::now() - chrono::Duration::days(days as i64);

                let mut history: Vec<&Session> = sessions
                    .iter()
                    .filter(|s| s.started_at >= cutoff)
                    .filter(|s| agent_id.is_none_or(|id| s.agent_id == id))
                    .collect();

                history.sort_by(|a, b| b.started_at.cmp(&a.started_at));

                let limit = input.get("limit").and_then(|v| v.as_u64()).unwrap_or(100) as usize;

                let result: Vec<&&Session> = history.iter().take(limit).collect();

                let total_cost: f64 = history.iter().map(|s| s.total_cost).sum();
                let total_tokens: u64 = history.iter().map(|s| s.total_tokens).sum();
                let total_messages: u64 = history.iter().map(|s| s.messages).sum();

                let completed = history
                    .iter()
                    .filter(|s| s.status == SessionStatus::Completed)
                    .count();
                let errored = history
                    .iter()
                    .filter(|s| s.status == SessionStatus::Error)
                    .count();

                Ok(api_response(json!({
                    "sessions": result,
                    "total": history.len(),
                    "total_cost": total_cost,
                    "total_tokens": total_tokens,
                    "total_messages": total_messages,
                    "completed": completed,
                    "errored": errored,
                    "days": days
                })))
            }
        },
    );
}

fn register_cleanup(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function_with(
        RegisterFunctionMessage::with_id("rimuru.sessions.cleanup".to_string()),
        move |input: Value| {
            let kv = kv.clone();
            async move {
                let input = extract_input(input);
                let max_age_days = input
                    .get("max_age_days")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(90);

                let cutoff = Utc::now() - chrono::Duration::days(max_age_days as i64);

                let sessions: Vec<Session> = kv.list("sessions").await.map_err(kv_err)?;

                let stale: Vec<&Session> = sessions
                    .iter()
                    .filter(|s| s.status != SessionStatus::Active)
                    .filter(|s| {
                        s.ended_at
                            .map_or(s.started_at < cutoff, |ended| ended < cutoff)
                    })
                    .collect();

                let mut cleaned = 0u64;
                let mut freed_cost = 0.0f64;

                for session in &stale {
                    let session_id = session.id.to_string();
                    freed_cost += session.total_cost;
                    kv.delete("sessions", &session_id).await.map_err(kv_err)?;
                    cleaned += 1;
                }

                Ok(api_response(json!({
                    "cleaned": cleaned,
                    "freed_cost": freed_cost,
                    "max_age_days": max_age_days,
                    "cutoff": cutoff.to_rfc3339()
                })))
            }
        },
    );
}
