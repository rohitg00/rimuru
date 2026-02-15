mod aggregator;
mod collector;

pub use aggregator::SessionAggregator;
pub use collector::SystemCollector;

use crate::db::DatabaseError;
use crate::models::SystemMetrics;
use crate::repo::MetricsRepository;
use sqlx::PgPool;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tracing::{debug, error, info, warn};

const DEFAULT_COLLECTION_INTERVAL_SECS: u64 = 5;

#[derive(Debug, Clone)]
pub struct MetricsCollectorConfig {
    pub collection_interval_secs: u64,
    pub store_to_database: bool,
}

impl Default for MetricsCollectorConfig {
    fn default() -> Self {
        Self {
            collection_interval_secs: DEFAULT_COLLECTION_INTERVAL_SECS,
            store_to_database: true,
        }
    }
}

impl MetricsCollectorConfig {
    pub fn with_interval(mut self, secs: u64) -> Self {
        self.collection_interval_secs = secs;
        self
    }

    pub fn without_database_storage(mut self) -> Self {
        self.store_to_database = false;
        self
    }
}

pub struct MetricsCollector {
    system_collector: SystemCollector,
    session_aggregator: SessionAggregator,
    metrics_repo: MetricsRepository,
    config: MetricsCollectorConfig,
    is_running: Arc<AtomicBool>,
    latest_metrics: Arc<RwLock<Option<SystemMetrics>>>,
    task_handle: Arc<RwLock<Option<JoinHandle<()>>>>,
}

impl MetricsCollector {
    pub fn new(pool: PgPool) -> Self {
        Self::with_config(pool, MetricsCollectorConfig::default())
    }

    pub fn with_config(pool: PgPool, config: MetricsCollectorConfig) -> Self {
        Self {
            system_collector: SystemCollector::new(),
            session_aggregator: SessionAggregator::new(pool.clone()),
            metrics_repo: MetricsRepository::new(pool),
            config,
            is_running: Arc::new(AtomicBool::new(false)),
            latest_metrics: Arc::new(RwLock::new(None)),
            task_handle: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn collect_once(&self) -> Result<SystemMetrics, DatabaseError> {
        let active_sessions = self
            .session_aggregator
            .get_active_sessions_count_or_default()
            .await;
        let metrics = self.system_collector.collect(active_sessions).await;

        if self.config.store_to_database {
            self.metrics_repo.record_snapshot(&metrics).await?;
            debug!(
                cpu = metrics.cpu_percent,
                memory_used = metrics.memory_used_mb,
                memory_total = metrics.memory_total_mb,
                sessions = active_sessions,
                "Metrics recorded to database"
            );
        }

        *self.latest_metrics.write().await = Some(metrics.clone());

        Ok(metrics)
    }

    pub async fn start(&self) {
        if self.is_running.load(Ordering::SeqCst) {
            warn!("Metrics collector is already running");
            return;
        }

        self.is_running.store(true, Ordering::SeqCst);
        info!(
            interval_secs = self.config.collection_interval_secs,
            "Starting metrics collector background task"
        );

        let is_running = Arc::clone(&self.is_running);
        let latest_metrics = Arc::clone(&self.latest_metrics);
        let system_collector = self.system_collector.clone();
        let session_aggregator = self.session_aggregator.clone();
        let metrics_repo = MetricsRepository::new(self.session_aggregator.pool().clone());
        let interval = Duration::from_secs(self.config.collection_interval_secs);
        let store_to_db = self.config.store_to_database;

        let handle = tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);

            // Don't immediately tick - wait for first interval
            interval_timer.tick().await;

            while is_running.load(Ordering::SeqCst) {
                interval_timer.tick().await;

                if !is_running.load(Ordering::SeqCst) {
                    break;
                }

                let active_sessions = session_aggregator
                    .get_active_sessions_count_or_default()
                    .await;
                let metrics = system_collector.collect(active_sessions).await;

                if store_to_db {
                    match metrics_repo.record_snapshot(&metrics).await {
                        Ok(_) => {
                            debug!(
                                cpu = metrics.cpu_percent,
                                memory_used = metrics.memory_used_mb,
                                sessions = active_sessions,
                                "Background metrics recorded"
                            );
                        }
                        Err(e) => {
                            error!(error = %e, "Failed to record metrics to database");
                        }
                    }
                }

                *latest_metrics.write().await = Some(metrics);
            }

            info!("Metrics collector background task stopped");
        });

        *self.task_handle.write().await = Some(handle);
    }

    pub async fn stop(&self) {
        if !self.is_running.load(Ordering::SeqCst) {
            warn!("Metrics collector is not running");
            return;
        }

        info!("Stopping metrics collector...");
        self.is_running.store(false, Ordering::SeqCst);

        if let Some(handle) = self.task_handle.write().await.take() {
            if let Err(e) = handle.await {
                error!(error = %e, "Error waiting for metrics collector task to stop");
            }
        }

        info!("Metrics collector stopped");
    }

    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }

    pub async fn get_latest(&self) -> Option<SystemMetrics> {
        self.latest_metrics.read().await.clone()
    }

    pub async fn get_latest_from_db(&self) -> Result<Option<SystemMetrics>, DatabaseError> {
        self.metrics_repo.get_latest().await
    }

    pub fn system_collector(&self) -> &SystemCollector {
        &self.system_collector
    }

    pub fn session_aggregator(&self) -> &SessionAggregator {
        &self.session_aggregator
    }

    pub fn config(&self) -> &MetricsCollectorConfig {
        &self.config
    }
}

impl Drop for MetricsCollector {
    fn drop(&mut self) {
        if self.is_running.load(Ordering::SeqCst) {
            self.is_running.store(false, Ordering::SeqCst);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = MetricsCollectorConfig::default();
        assert_eq!(config.collection_interval_secs, 5);
        assert!(config.store_to_database);
    }

    #[test]
    fn test_config_builder() {
        let config = MetricsCollectorConfig::default()
            .with_interval(10)
            .without_database_storage();

        assert_eq!(config.collection_interval_secs, 10);
        assert!(!config.store_to_database);
    }

    #[tokio::test]
    async fn test_system_collector_standalone() {
        let collector = SystemCollector::new();
        let metrics = collector.collect(5).await;

        assert!(metrics.cpu_percent >= 0.0);
        assert!(metrics.memory_total_mb > 0);
        assert_eq!(metrics.active_sessions, 5);
    }
}
