use crate::state::AppState;
use rimuru_core::{SessionRepository, SystemCollector};
use serde::Serialize;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, Serialize)]
pub struct MetricsUpdatePayload {
    pub cpu_usage: f64,
    pub memory_usage_percent: f64,
    pub memory_used_mb: u64,
    pub active_sessions: i32,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionEventPayload {
    pub session_id: String,
    pub agent_id: String,
    pub event_type: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CostRecordedPayload {
    pub session_id: String,
    pub model: String,
    pub cost: f64,
    pub tokens: i64,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentStatusPayload {
    pub agent_id: String,
    pub name: String,
    pub status: String,
    pub timestamp: String,
}

pub async fn start_event_emitter(app: AppHandle, state: AppState) -> Result<(), anyhow::Error> {
    let app_metrics = app.clone();
    let state_metrics = state.clone();

    tokio::spawn(async move {
        let collector = SystemCollector::new();
        let mut interval = tokio::time::interval(Duration::from_secs(5));

        loop {
            interval.tick().await;

            let session_repo = SessionRepository::new(state_metrics.db.pool().clone());

            let active_count = session_repo.get_active_count().await.unwrap_or(0) as i32;

            let metrics = collector.collect(active_count).await;

            let memory_usage_percent = if metrics.memory_total_mb > 0 {
                (metrics.memory_used_mb as f64 / metrics.memory_total_mb as f64) * 100.0
            } else {
                0.0
            };

            let payload = MetricsUpdatePayload {
                cpu_usage: metrics.cpu_percent as f64,
                memory_usage_percent,
                memory_used_mb: metrics.memory_used_mb as u64,
                active_sessions: metrics.active_sessions,
                timestamp: chrono::Utc::now().to_rfc3339(),
            };

            if let Err(e) = app_metrics.emit("metrics-update", payload) {
                tracing::warn!("Failed to emit metrics update: {}", e);
            }
        }
    });

    Ok(())
}

pub fn emit_session_started(app: &AppHandle, session_id: &str, agent_id: &str) {
    let payload = SessionEventPayload {
        session_id: session_id.to_string(),
        agent_id: agent_id.to_string(),
        event_type: "started".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    if let Err(e) = app.emit("session-started", payload) {
        tracing::warn!("Failed to emit session started event: {}", e);
    }
}

pub fn emit_session_ended(app: &AppHandle, session_id: &str, agent_id: &str) {
    let payload = SessionEventPayload {
        session_id: session_id.to_string(),
        agent_id: agent_id.to_string(),
        event_type: "ended".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    if let Err(e) = app.emit("session-ended", payload) {
        tracing::warn!("Failed to emit session ended event: {}", e);
    }
}

pub fn emit_cost_recorded(app: &AppHandle, session_id: &str, model: &str, cost: f64, tokens: i64) {
    let payload = CostRecordedPayload {
        session_id: session_id.to_string(),
        model: model.to_string(),
        cost,
        tokens,
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    if let Err(e) = app.emit("cost-recorded", payload) {
        tracing::warn!("Failed to emit cost recorded event: {}", e);
    }
}

pub fn emit_agent_status_changed(app: &AppHandle, agent_id: &str, name: &str, status: &str) {
    let payload = AgentStatusPayload {
        agent_id: agent_id.to_string(),
        name: name.to_string(),
        status: status.to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    if let Err(e) = app.emit("agent-status-changed", payload) {
        tracing::warn!("Failed to emit agent status changed event: {}", e);
    }
}
