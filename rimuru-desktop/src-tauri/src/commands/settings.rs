use crate::state::AppState;
use rimuru_core::{AgentRepository, SessionRepository};
use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub sync_interval: String,
    pub default_model: String,
    pub session_timeout: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            sync_interval: "60".to_string(),
            default_model: "gpt-4o".to_string(),
            session_timeout: "30".to_string(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct DbStats {
    pub db_size_bytes: u64,
    pub db_size_display: String,
    pub total_sessions: i64,
    pub total_agents: i64,
    pub db_path: String,
}

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, String> {
    let settings = state.settings.read().await;
    Ok(settings.clone())
}

#[tauri::command]
pub async fn save_settings(
    state: State<'_, AppState>,
    settings: AppSettings,
) -> Result<bool, String> {
    info!(
        "Saving settings: sync_interval={}, default_model={}, session_timeout={}",
        settings.sync_interval, settings.default_model, settings.session_timeout
    );

    let mut current = state.settings.write().await;
    *current = settings;
    Ok(true)
}

#[tauri::command]
pub async fn get_db_stats(state: State<'_, AppState>) -> Result<DbStats, String> {
    let session_repo = SessionRepository::new(state.db.pool().clone());
    let agent_repo = AgentRepository::new(state.db.pool().clone());

    let total_sessions = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM sessions")
        .fetch_one(state.db.pool())
        .await
        .unwrap_or(0);

    let total_agents = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM agents")
        .fetch_one(state.db.pool())
        .await
        .unwrap_or(0);

    let db_path = state.config.database.url.clone();
    let file_path = db_path
        .strip_prefix("sqlite://")
        .or_else(|| db_path.strip_prefix("sqlite:"))
        .unwrap_or(&db_path);

    let db_size_bytes = tokio::fs::metadata(file_path)
        .await
        .map(|m| m.len())
        .unwrap_or(0);

    let db_size_display = if db_size_bytes > 1_048_576 {
        format!("{:.1} MB", db_size_bytes as f64 / 1_048_576.0)
    } else if db_size_bytes > 1024 {
        format!("{:.1} KB", db_size_bytes as f64 / 1024.0)
    } else {
        format!("{} B", db_size_bytes)
    };

    Ok(DbStats {
        db_size_bytes,
        db_size_display,
        total_sessions,
        total_agents,
        db_path: file_path.to_string(),
    })
}
