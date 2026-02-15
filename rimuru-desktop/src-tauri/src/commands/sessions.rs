use crate::state::AppState;
use rimuru_core::{AgentRepository, Repository, Session, SessionRepository};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::State;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct SessionResponse {
    pub id: String,
    pub agent_id: String,
    pub agent_name: Option<String>,
    pub status: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub duration_secs: Option<i64>,
    pub total_tokens: i64,
    pub total_cost: f64,
}

#[derive(Debug, Deserialize)]
pub struct SessionFilters {
    pub agent_id: Option<String>,
    pub status: Option<String>,
    pub from_date: Option<String>,
    pub to_date: Option<String>,
    pub limit: Option<i64>,
    #[allow(dead_code)]
    pub offset: Option<i64>,
}

fn session_to_response(s: Session, agent_names: &HashMap<Uuid, String>) -> SessionResponse {
    let duration = s.duration_seconds();
    let agent_name = agent_names.get(&s.agent_id).cloned();

    let total_tokens = s
        .metadata
        .as_object()
        .map(|m| {
            let input = m.get("input_tokens").and_then(|v| v.as_i64()).unwrap_or(0);
            let output = m.get("output_tokens").and_then(|v| v.as_i64()).unwrap_or(0);
            input + output
        })
        .unwrap_or(0);

    let total_cost = s
        .metadata
        .as_object()
        .and_then(|m| m.get("cost_usd"))
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);

    SessionResponse {
        id: s.id.to_string(),
        agent_id: s.agent_id.to_string(),
        agent_name,
        status: s.status.to_string(),
        started_at: s.started_at.to_rfc3339(),
        ended_at: s.ended_at.map(|t| t.to_rfc3339()),
        duration_secs: duration,
        total_tokens,
        total_cost,
    }
}

async fn build_agent_name_map(state: &AppState) -> HashMap<Uuid, String> {
    let agent_repo = AgentRepository::new(state.db.pool().clone());
    match agent_repo.get_all().await {
        Ok(agents) => agents.into_iter().map(|a| (a.id, a.name)).collect(),
        Err(_) => HashMap::new(),
    }
}

#[tauri::command]
pub async fn get_sessions(
    state: State<'_, AppState>,
    filters: Option<SessionFilters>,
) -> Result<Vec<SessionResponse>, String> {
    let repo = SessionRepository::new(state.db.pool().clone());
    let sessions = repo.get_all().await.map_err(|e| e.to_string())?;
    let agent_names = build_agent_name_map(&state).await;

    let mut results: Vec<SessionResponse> = sessions
        .into_iter()
        .map(|s| session_to_response(s, &agent_names))
        .collect();

    if let Some(f) = filters {
        if let Some(agent_id) = f.agent_id {
            results.retain(|s| s.agent_id == agent_id);
        }
        if let Some(status) = f.status {
            results.retain(|s| s.status.to_lowercase() == status.to_lowercase());
        }
        if let Some(limit) = f.limit {
            results.truncate(limit as usize);
        }
    }

    Ok(results)
}

#[tauri::command]
pub async fn get_session_details(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<Option<SessionResponse>, String> {
    let repo = SessionRepository::new(state.db.pool().clone());
    let uuid = uuid::Uuid::parse_str(&session_id).map_err(|e| e.to_string())?;

    match repo.get_by_id(uuid).await.map_err(|e| e.to_string())? {
        Some(session) => {
            let agent_names = build_agent_name_map(&state).await;
            Ok(Some(session_to_response(session, &agent_names)))
        }
        None => Ok(None),
    }
}

#[tauri::command]
pub async fn get_active_sessions(
    state: State<'_, AppState>,
) -> Result<Vec<SessionResponse>, String> {
    let repo = SessionRepository::new(state.db.pool().clone());
    let sessions = repo.get_active().await.map_err(|e| e.to_string())?;
    let agent_names = build_agent_name_map(&state).await;

    Ok(sessions
        .into_iter()
        .map(|s| session_to_response(s, &agent_names))
        .collect())
}
