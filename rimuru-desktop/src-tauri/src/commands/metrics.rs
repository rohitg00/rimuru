use crate::state::AppState;
use rimuru_core::{MetricsRepository, SessionRepository, SystemCollector};
use serde::Serialize;
use tauri::State;

#[derive(Debug, Serialize)]
pub struct SystemMetricsResponse {
    pub cpu_usage: f64,
    pub memory_used_mb: u64,
    pub memory_total_mb: u64,
    pub memory_usage_percent: f64,
    pub active_sessions: i32,
    pub timestamp: String,
}

#[derive(Debug, Serialize)]
pub struct MetricsHistoryPoint {
    pub timestamp: String,
    pub cpu_usage: f64,
    pub memory_usage_percent: f64,
    pub active_sessions: i32,
}

#[tauri::command]
pub async fn get_system_metrics(
    state: State<'_, AppState>,
) -> Result<SystemMetricsResponse, String> {
    let collector = SystemCollector::new();

    let session_repo = SessionRepository::new(state.db.pool().clone());
    let active_count = session_repo.get_active_count().await.unwrap_or(0) as i32;

    let metrics = collector.collect(active_count).await;

    let memory_usage_percent = if metrics.memory_total_mb > 0 {
        (metrics.memory_used_mb as f64 / metrics.memory_total_mb as f64) * 100.0
    } else {
        0.0
    };

    Ok(SystemMetricsResponse {
        cpu_usage: metrics.cpu_percent as f64,
        memory_used_mb: metrics.memory_used_mb as u64,
        memory_total_mb: metrics.memory_total_mb as u64,
        memory_usage_percent,
        active_sessions: metrics.active_sessions,
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

#[tauri::command]
pub async fn get_metrics_history(
    state: State<'_, AppState>,
    hours: Option<i64>,
) -> Result<Vec<MetricsHistoryPoint>, String> {
    let repo = MetricsRepository::new(state.db.pool().clone());
    let num_hours = hours.unwrap_or(24);

    let snapshots = repo
        .get_recent(num_hours * 12)
        .await
        .map_err(|e| e.to_string())?;

    let now = chrono::Utc::now();

    let filtered: Vec<MetricsHistoryPoint> = snapshots
        .into_iter()
        .filter(|s| {
            let age = now.signed_duration_since(s.timestamp);
            age.num_hours() < num_hours
        })
        .map(|s| {
            let memory_usage_percent = if s.memory_total_mb > 0 {
                (s.memory_used_mb as f64 / s.memory_total_mb as f64) * 100.0
            } else {
                0.0
            };
            MetricsHistoryPoint {
                timestamp: s.timestamp.to_rfc3339(),
                cpu_usage: s.cpu_percent as f64,
                memory_usage_percent,
                active_sessions: s.active_sessions,
            }
        })
        .collect();

    Ok(filtered)
}
