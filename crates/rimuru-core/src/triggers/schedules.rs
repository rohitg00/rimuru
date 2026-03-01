use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::RwLock;
use tracing::{info, warn};

use iii_sdk::{III, IIIError, TriggerConfig, TriggerHandler};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleEntry {
    pub function_id: String,
    pub interval_secs: u64,
    pub description: String,
}

pub struct ScheduleTriggerHandler {
    iii: III,
    entries: Arc<RwLock<HashMap<String, ScheduleEntry>>>,
    abort_handles: Arc<RwLock<HashMap<String, tokio::task::JoinHandle<()>>>>,
}

impl ScheduleTriggerHandler {
    pub fn new(iii: III) -> Self {
        Self {
            iii,
            entries: Arc::new(RwLock::new(HashMap::new())),
            abort_handles: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl TriggerHandler for ScheduleTriggerHandler {
    async fn register_trigger(&self, config: TriggerConfig) -> Result<(), IIIError> {
        let interval_secs = config
            .config
            .get("interval_secs")
            .and_then(|v| v.as_u64())
            .unwrap_or(60);
        let description = config
            .config
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let entry = ScheduleEntry {
            function_id: config.function_id.clone(),
            interval_secs,
            description: description.clone(),
        };

        self.entries
            .write()
            .await
            .insert(config.id.clone(), entry);

        let iii = self.iii.clone();
        let function_id = config.function_id.clone();
        let trigger_id = config.id.clone();
        let interval = Duration::from_secs(interval_secs);

        let handle = tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            ticker.tick().await;

            loop {
                ticker.tick().await;

                let payload = json!({
                    "trigger_id": trigger_id,
                    "trigger_type": "schedule",
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                });

                if let Err(e) = iii.trigger_void(&function_id, payload) {
                    warn!(
                        "Schedule trigger failed for {}: {}",
                        function_id, e
                    );
                }
            }
        });

        self.abort_handles
            .write()
            .await
            .insert(config.id.clone(), handle);

        info!(
            "Registered schedule trigger: {} every {}s ({})",
            config.function_id, interval_secs, description
        );
        Ok(())
    }

    async fn unregister_trigger(&self, config: TriggerConfig) -> Result<(), IIIError> {
        self.entries.write().await.remove(&config.id);

        if let Some(handle) = self.abort_handles.write().await.remove(&config.id) {
            handle.abort();
        }

        info!("Unregistered schedule trigger: {}", config.id);
        Ok(())
    }
}

struct Schedule {
    interval_secs: u64,
    function_id: &'static str,
    description: &'static str,
}

const SCHEDULES: &[Schedule] = &[
    Schedule {
        interval_secs: 300,
        function_id: "rimuru.metrics.collect",
        description: "Collect system metrics",
    },
    Schedule {
        interval_secs: 21600,
        function_id: "rimuru.models.sync",
        description: "Sync model pricing",
    },
    Schedule {
        interval_secs: 86400,
        function_id: "rimuru.costs.daily_rollup",
        description: "Aggregate daily costs",
    },
    Schedule {
        interval_secs: 86400,
        function_id: "rimuru.sessions.cleanup",
        description: "Clean stale sessions",
    },
];

pub fn register(iii: &III) {
    let handler = ScheduleTriggerHandler::new(iii.clone());

    iii.register_trigger_type(
        "schedule",
        "Time-based schedule triggers for periodic tasks",
        handler,
    );
    info!("Registered schedule trigger type");

    for schedule in SCHEDULES {
        let config = json!({
            "interval_secs": schedule.interval_secs,
            "description": schedule.description,
        });

        match iii.register_trigger("schedule", schedule.function_id, config) {
            Ok(_trigger) => {
                info!(
                    "Registered schedule trigger: {} every {}s ({})",
                    schedule.function_id, schedule.interval_secs, schedule.description
                );
            }
            Err(e) => {
                tracing::error!(
                    "Failed to register schedule trigger {} ({}): {}",
                    schedule.function_id,
                    schedule.description,
                    e
                );
            }
        }
    }

    info!("Registered {} schedule triggers", SCHEDULES.len());
}
