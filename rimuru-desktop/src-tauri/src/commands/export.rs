use crate::state::AppState;
use chrono::{Duration, Utc};
use rimuru_core::{
    create_builtin_exporter, CostRepository, ExportOptions, Repository, SessionRepository,
};
use serde::Deserialize;
use tauri::State;
use tracing::info;

#[derive(Debug, Clone, Deserialize)]
pub struct SessionFilters {
    pub agent_id: Option<String>,
    pub status: Option<String>,
    pub from_date: Option<String>,
    pub to_date: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TimeRangeRequest {
    pub range: Option<String>,
    pub from_date: Option<String>,
    pub to_date: Option<String>,
}

fn parse_datetime(s: &str) -> Option<chrono::DateTime<Utc>> {
    chrono::DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

#[tauri::command]
pub async fn export_sessions(
    state: State<'_, AppState>,
    format: String,
    filters: Option<SessionFilters>,
) -> Result<String, String> {
    info!("Exporting sessions as {}", format);

    let session_repo = SessionRepository::new(state.db.pool().clone());

    let (start, end) = if let Some(ref f) = filters {
        let start = f
            .from_date
            .as_deref()
            .and_then(parse_datetime)
            .unwrap_or_else(|| Utc::now() - Duration::days(365));
        let end = f
            .to_date
            .as_deref()
            .and_then(parse_datetime)
            .unwrap_or_else(|| Utc::now());
        (start, end)
    } else {
        (Utc::now() - Duration::days(365), Utc::now())
    };

    let mut sessions = session_repo
        .get_by_date_range(start, end)
        .await
        .map_err(|e| format!("Failed to fetch sessions: {}", e))?;

    if let Some(ref f) = filters {
        if let Some(ref status) = f.status {
            sessions.retain(|s| format!("{:?}", s.status).to_lowercase() == status.to_lowercase());
        }
        if let Some(limit) = f.limit {
            sessions.truncate(limit);
        }
    }

    let exporter_name = match format.as_str() {
        "csv" => "csv-exporter",
        "json" => "json-exporter",
        _ => return Err(format!("Unsupported export format: {}", format)),
    };

    let exporter = create_builtin_exporter(exporter_name)
        .ok_or_else(|| format!("Exporter '{}' not available", exporter_name))?;

    let options = ExportOptions {
        include_headers: true,
        pretty: format == "json",
        ..Default::default()
    };

    let bytes = exporter
        .export_sessions(&sessions, options)
        .await
        .map_err(|e| format!("Export failed: {}", e))?;

    String::from_utf8(bytes).map_err(|e| format!("Invalid UTF-8 in export: {}", e))
}

#[tauri::command]
pub async fn export_costs(
    state: State<'_, AppState>,
    format: String,
    time_range: Option<TimeRangeRequest>,
) -> Result<String, String> {
    info!("Exporting costs as {}", format);

    let cost_repo = CostRepository::new(state.db.pool().clone());

    let (start, end) = if let Some(ref tr) = time_range {
        let end = tr
            .to_date
            .as_deref()
            .and_then(parse_datetime)
            .unwrap_or_else(|| Utc::now());
        let start = tr
            .from_date
            .as_deref()
            .and_then(parse_datetime)
            .unwrap_or_else(|| match tr.range.as_deref() {
                Some("7d") => end - Duration::days(7),
                Some("30d") => end - Duration::days(30),
                Some("90d") => end - Duration::days(90),
                _ => end - Duration::days(30),
            });
        (start, end)
    } else {
        (Utc::now() - Duration::days(30), Utc::now())
    };

    let all_costs = cost_repo
        .get_all()
        .await
        .map_err(|e| format!("Failed to fetch costs: {}", e))?;

    let costs: Vec<_> = all_costs
        .into_iter()
        .filter(|c| c.recorded_at >= start && c.recorded_at <= end)
        .collect();

    let exporter_name = match format.as_str() {
        "csv" => "csv-exporter",
        "json" => "json-exporter",
        _ => return Err(format!("Unsupported export format: {}", format)),
    };

    let exporter = create_builtin_exporter(exporter_name)
        .ok_or_else(|| format!("Exporter '{}' not available", exporter_name))?;

    let options = ExportOptions {
        include_headers: true,
        pretty: format == "json",
        ..Default::default()
    };

    let bytes = exporter
        .export_costs(&costs, options)
        .await
        .map_err(|e| format!("Export failed: {}", e))?;

    String::from_utf8(bytes).map_err(|e| format!("Invalid UTF-8 in export: {}", e))
}
