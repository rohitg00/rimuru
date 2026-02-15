use crate::state::AppState;
use rimuru_core::{
    AnthropicSyncProvider, GoogleSyncProvider, ModelRepository, ModelSyncProvider,
    OpenAISyncProvider,
};
use serde::Serialize;
use tauri::State;
use tracing::{info, warn};

#[derive(Debug, Serialize)]
pub struct SyncStatusResponse {
    pub last_sync: Option<String>,
    pub is_syncing: bool,
    pub provider_statuses: Vec<ProviderStatus>,
}

#[derive(Debug, Serialize)]
pub struct ProviderStatus {
    pub name: String,
    pub enabled: bool,
    pub last_sync: Option<String>,
    pub model_count: i32,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SyncResultResponse {
    pub success: bool,
    pub models_synced: i32,
    pub providers_synced: Vec<String>,
    pub errors: Vec<String>,
}

#[tauri::command]
pub async fn trigger_sync(state: State<'_, AppState>) -> Result<SyncResultResponse, String> {
    info!("Triggering model pricing sync");

    let model_repo = ModelRepository::new(state.db.pool().clone());
    let mut total_synced: i32 = 0;
    let mut providers_synced = Vec::new();
    let mut errors = Vec::new();

    if state.config.sync.providers.anthropic {
        let provider = AnthropicSyncProvider::new();
        match provider.fetch_models().await {
            Ok(models) => {
                let count = models.len() as i32;
                for model in &models {
                    if let Err(e) = model_repo.upsert(model).await {
                        warn!("Failed to persist model {}: {}", model.model_name, e);
                    }
                }
                total_synced += count;
                providers_synced.push("anthropic".to_string());
                info!("Synced {} models from Anthropic", count);
            }
            Err(e) => {
                let msg = format!("Anthropic sync failed: {}", e);
                warn!("{}", msg);
                errors.push(msg);
            }
        }
    }

    if state.config.sync.providers.openai {
        let provider = OpenAISyncProvider::new();
        match provider.fetch_models().await {
            Ok(models) => {
                let count = models.len() as i32;
                for model in &models {
                    if let Err(e) = model_repo.upsert(model).await {
                        warn!("Failed to persist model {}: {}", model.model_name, e);
                    }
                }
                total_synced += count;
                providers_synced.push("openai".to_string());
                info!("Synced {} models from OpenAI", count);
            }
            Err(e) => {
                let msg = format!("OpenAI sync failed: {}", e);
                warn!("{}", msg);
                errors.push(msg);
            }
        }
    }

    if state.config.sync.providers.google {
        let provider = GoogleSyncProvider::new();
        match provider.fetch_models().await {
            Ok(models) => {
                let count = models.len() as i32;
                for model in &models {
                    if let Err(e) = model_repo.upsert(model).await {
                        warn!("Failed to persist model {}: {}", model.model_name, e);
                    }
                }
                total_synced += count;
                providers_synced.push("google".to_string());
                info!("Synced {} models from Google", count);
            }
            Err(e) => {
                let msg = format!("Google sync failed: {}", e);
                warn!("{}", msg);
                errors.push(msg);
            }
        }
    }

    let mut last_sync = state.last_sync.write().await;
    *last_sync = Some(chrono::Utc::now());

    Ok(SyncResultResponse {
        success: errors.is_empty(),
        models_synced: total_synced,
        providers_synced,
        errors,
    })
}

#[tauri::command]
pub async fn get_sync_status(state: State<'_, AppState>) -> Result<SyncStatusResponse, String> {
    let last_sync = state.last_sync.read().await;
    let model_repo = ModelRepository::new(state.db.pool().clone());

    let anthropic_count = model_repo
        .get_by_provider("anthropic")
        .await
        .map(|m| m.len())
        .unwrap_or(0) as i32;
    let openai_count = model_repo
        .get_by_provider("openai")
        .await
        .map(|m| m.len())
        .unwrap_or(0) as i32;
    let google_count = model_repo
        .get_by_provider("google")
        .await
        .map(|m| m.len())
        .unwrap_or(0) as i32;

    let last_sync_str = last_sync.map(|t| t.to_rfc3339());

    let providers = vec![
        ProviderStatus {
            name: "OpenAI".to_string(),
            enabled: state.config.sync.providers.openai,
            last_sync: last_sync_str.clone(),
            model_count: openai_count,
            error: None,
        },
        ProviderStatus {
            name: "Anthropic".to_string(),
            enabled: state.config.sync.providers.anthropic,
            last_sync: last_sync_str.clone(),
            model_count: anthropic_count,
            error: None,
        },
        ProviderStatus {
            name: "Google".to_string(),
            enabled: state.config.sync.providers.google,
            last_sync: last_sync_str,
            model_count: google_count,
            error: None,
        },
    ];

    Ok(SyncStatusResponse {
        last_sync: last_sync.map(|t| t.to_rfc3339()),
        is_syncing: false,
        provider_statuses: providers,
    })
}
