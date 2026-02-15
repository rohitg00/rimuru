use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tokio::time::{interval, Duration};
use tracing::{debug, error, info, warn};

use crate::error::{RimuruError, RimuruResult};
use crate::repo::ModelRepository;

use super::aggregator::ModelAggregator;
use super::traits::{ModelSyncProvider, SyncScheduler};
use super::types::{
    ProviderSyncStatus, SyncConfig, SyncHistory, SyncHistoryEntry, SyncResult, SyncStatus,
};

pub struct BackgroundSyncScheduler {
    config: SyncConfig,
    providers: Arc<RwLock<HashMap<String, Arc<dyn ModelSyncProvider>>>>,
    model_repo: Arc<ModelRepository>,
    aggregator: Arc<ModelAggregator>,
    status: Arc<RwLock<SyncStatus>>,
    history: Arc<Mutex<SyncHistory>>,
    running: Arc<RwLock<bool>>,
    shutdown_tx: Arc<Mutex<Option<tokio::sync::oneshot::Sender<()>>>>,
}

impl BackgroundSyncScheduler {
    pub fn new(config: SyncConfig, model_repo: ModelRepository) -> Self {
        Self {
            config,
            providers: Arc::new(RwLock::new(HashMap::new())),
            model_repo: Arc::new(model_repo),
            aggregator: Arc::new(ModelAggregator::new()),
            status: Arc::new(RwLock::new(SyncStatus::default())),
            history: Arc::new(Mutex::new(SyncHistory::default())),
            running: Arc::new(RwLock::new(false)),
            shutdown_tx: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn register_provider(&self, provider: Arc<dyn ModelSyncProvider>) {
        let name = provider.provider_name().to_string();
        let mut providers = self.providers.write().await;
        providers.insert(name.clone(), provider);

        let mut status = self.status.write().await;
        let enabled = self.is_provider_enabled(&name);
        status
            .provider_status
            .insert(name.clone(), ProviderSyncStatus::new(&name, enabled));

        info!("Registered sync provider: {}", name);
    }

    fn is_provider_enabled(&self, provider: &str) -> bool {
        match provider.to_lowercase().as_str() {
            "anthropic" => self.config.providers.anthropic,
            "openai" => self.config.providers.openai,
            "google" => self.config.providers.google,
            "openrouter" => self.config.providers.openrouter,
            "litellm" => self.config.providers.litellm,
            _ => true,
        }
    }

    pub async fn get_status(&self) -> SyncStatus {
        self.status.read().await.clone()
    }

    pub async fn get_history(&self, limit: usize) -> Vec<SyncHistoryEntry> {
        let history = self.history.lock().await;
        history.recent(limit).to_vec()
    }

    async fn sync_provider(&self, provider: &Arc<dyn ModelSyncProvider>) -> SyncResult {
        let provider_name = provider.provider_name();
        let start_time = std::time::Instant::now();
        let mut result = SyncResult::new(provider_name);

        info!("Starting sync for provider: {}", provider_name);

        match provider.fetch_models().await {
            Ok(models) => {
                debug!("Fetched {} models from {}", models.len(), provider_name);

                for model in &models {
                    match self.model_repo.upsert(model).await {
                        Ok(_) => {
                            result.models_added += 1;
                        }
                        Err(e) => {
                            warn!(
                                "Failed to upsert model {}/{}: {}",
                                model.provider, model.model_name, e
                            );
                        }
                    }
                }

                result.success = true;
            }
            Err(e) => {
                error!("Failed to fetch models from {}: {}", provider_name, e);
                result = result.with_error(super::types::SyncError::api_error(&e.to_string()));
            }
        }

        result.duration_ms = start_time.elapsed().as_millis() as u64;
        result.last_sync = Utc::now();

        let mut status = self.status.write().await;
        if let Some(provider_status) = status.provider_status.get_mut(provider_name) {
            provider_status.last_sync = Some(result.last_sync);
            provider_status.last_success = result.success;
            provider_status.models_count = result.total_models();
            if result.success {
                provider_status.consecutive_failures = 0;
                provider_status.last_error = None;
            } else {
                provider_status.consecutive_failures += 1;
                provider_status.last_error = result.errors.first().map(|e| e.message.clone());
            }
        }

        let entry = if result.success {
            SyncHistoryEntry::success(provider_name, result.total_models(), result.duration_ms)
        } else {
            SyncHistoryEntry::failure(
                provider_name,
                result
                    .errors
                    .first()
                    .map(|e| e.message.as_str())
                    .unwrap_or("Unknown error"),
                result.duration_ms,
            )
        };

        let mut history = self.history.lock().await;
        history.add_entry(entry);

        info!(
            "Sync completed for {}: success={}, models={}, duration={}ms",
            provider_name,
            result.success,
            result.total_models(),
            result.duration_ms
        );

        result
    }

    async fn run_full_sync(&self) -> RimuruResult<SyncResult> {
        let providers = self.providers.read().await;
        let mut combined_result = SyncResult::new("all");
        let start_time = std::time::Instant::now();

        for (name, provider) in providers.iter() {
            if !self.is_provider_enabled(name) {
                debug!("Skipping disabled provider: {}", name);
                continue;
            }

            let result = self.sync_provider(provider).await;
            combined_result.models_added += result.models_added;
            combined_result.models_updated += result.models_updated;
            combined_result.models_unchanged += result.models_unchanged;
            combined_result.errors.extend(result.errors);
        }

        combined_result.duration_ms = start_time.elapsed().as_millis() as u64;
        combined_result.success = combined_result.errors.is_empty();

        let mut status = self.status.write().await;
        status.last_full_sync = Some(Utc::now());
        status.next_scheduled_sync =
            Some(Utc::now() + chrono::Duration::seconds(self.config.interval_secs as i64));

        Ok(combined_result)
    }

    async fn background_loop(self: Arc<Self>, mut shutdown_rx: tokio::sync::oneshot::Receiver<()>) {
        let interval_duration = Duration::from_secs(self.config.interval_secs);
        let mut ticker = interval(interval_duration);

        ticker.tick().await;

        if let Err(e) = self.run_full_sync().await {
            error!("Initial sync failed: {}", e);
        }

        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    if !*self.running.read().await {
                        break;
                    }

                    info!("Running scheduled sync");
                    if let Err(e) = self.run_full_sync().await {
                        error!("Scheduled sync failed: {}", e);
                    }
                }
                _ = &mut shutdown_rx => {
                    info!("Sync scheduler shutting down");
                    break;
                }
            }
        }
    }
}

#[async_trait]
impl SyncScheduler for BackgroundSyncScheduler {
    async fn start(&self) -> RimuruResult<()> {
        let mut running = self.running.write().await;
        if *running {
            return Err(RimuruError::Internal(
                "Scheduler already running".to_string(),
            ));
        }

        *running = true;
        drop(running);

        let mut status = self.status.write().await;
        status.is_running = true;
        status.next_scheduled_sync =
            Some(Utc::now() + chrono::Duration::seconds(self.config.interval_secs as i64));
        drop(status);

        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
        *self.shutdown_tx.lock().await = Some(shutdown_tx);

        let self_arc = Arc::new(BackgroundSyncScheduler {
            config: self.config.clone(),
            providers: self.providers.clone(),
            model_repo: self.model_repo.clone(),
            aggregator: self.aggregator.clone(),
            status: self.status.clone(),
            history: self.history.clone(),
            running: self.running.clone(),
            shutdown_tx: self.shutdown_tx.clone(),
        });

        tokio::spawn(async move {
            self_arc.background_loop(shutdown_rx).await;
        });

        info!(
            "Sync scheduler started with interval: {} seconds",
            self.config.interval_secs
        );
        Ok(())
    }

    async fn stop(&self) -> RimuruResult<()> {
        let mut running = self.running.write().await;
        *running = false;
        drop(running);

        if let Some(tx) = self.shutdown_tx.lock().await.take() {
            let _ = tx.send(());
        }

        let mut status = self.status.write().await;
        status.is_running = false;
        status.next_scheduled_sync = None;

        info!("Sync scheduler stopped");
        Ok(())
    }

    async fn trigger_sync(&self) -> RimuruResult<SyncResult> {
        self.run_full_sync().await
    }

    async fn trigger_provider_sync(&self, provider_name: &str) -> RimuruResult<SyncResult> {
        let providers = self.providers.read().await;
        let provider = providers.get(provider_name).ok_or_else(|| {
            RimuruError::AgentNotFound(format!("Provider not found: {}", provider_name))
        })?;

        Ok(self.sync_provider(provider).await)
    }

    fn is_running(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ModelInfo;
    use crate::sync::traits::ModelSyncProvider;

    struct TestProvider {
        name: String,
        models: Vec<ModelInfo>,
    }

    #[async_trait]
    impl ModelSyncProvider for TestProvider {
        fn provider_name(&self) -> &str {
            &self.name
        }

        async fn fetch_models(&self) -> RimuruResult<Vec<ModelInfo>> {
            Ok(self.models.clone())
        }
    }

    #[test]
    fn test_sync_config_default() {
        let config = SyncConfig::default();
        assert!(config.enabled);
        assert_eq!(config.interval_secs, 6 * 60 * 60);
    }

    #[test]
    fn test_is_provider_enabled() {
        let mut config = SyncConfig::default();
        config.providers.anthropic = false;

        assert!(!config.providers.anthropic);
        assert!(config.providers.openai);
        assert!(config.providers.google);
    }
}
