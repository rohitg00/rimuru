use iii_sdk::{III, InitOptions, TriggerRequest, register_worker};
use serde_json::json;
use tracing::info;

use crate::error::RimuruError;
use crate::state::StateKV;
use crate::{functions, triggers};

pub struct RimuruWorker {
    iii: III,
}

impl RimuruWorker {
    pub fn new(engine_url: &str) -> Self {
        let iii = register_worker(engine_url, InitOptions::default());

        Self { iii }
    }

    pub async fn start(&self) -> Result<(), RimuruError> {
        info!("Initializing rimuru worker (connection started by register_worker)");

        let kv = StateKV::new(self.iii.clone());
        functions::register_all(&self.iii, &kv);
        info!("Registered all functions");

        triggers::register_all(&self.iii);
        info!("Registered all triggers");

        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        match self
            .iii
            .trigger(TriggerRequest {
                function_id: "rimuru.agents.detect".to_string(),
                payload: json!({"auto_register": true}),
                action: None,
                timeout_ms: None,
            })
            .await
        {
            Ok(result) => {
                let total = result.get("total").and_then(|v| v.as_u64()).unwrap_or(0);
                info!("Auto-detected and registered {} agents", total);
            }
            Err(e) => {
                tracing::warn!("Failed to auto-detect agents: {}", e);
            }
        }

        match self
            .iii
            .trigger(TriggerRequest {
                function_id: "rimuru.agents.sync".to_string(),
                payload: json!({}),
                action: None,
                timeout_ms: None,
            })
            .await
        {
            Ok(result) => {
                let sessions = result
                    .get("synced_sessions")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let costs = result
                    .get("synced_costs")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                info!(
                    "Synced {} sessions, {} cost records from disk",
                    sessions, costs
                );
            }
            Err(e) => {
                tracing::warn!("Failed to sync agent data: {}", e);
            }
        }

        match self
            .iii
            .trigger(TriggerRequest {
                function_id: "rimuru.metrics.collect".to_string(),
                payload: json!({}),
                action: None,
                timeout_ms: None,
            })
            .await
        {
            Ok(_) => info!("Initial metrics collected"),
            Err(e) => tracing::warn!("Failed to collect initial metrics: {}", e),
        }

        match self
            .iii
            .trigger(TriggerRequest {
                function_id: "rimuru.hardware.detect".to_string(),
                payload: json!({}),
                action: None,
                timeout_ms: None,
            })
            .await
        {
            Ok(result) => {
                let backend = result
                    .get("hardware")
                    .and_then(|h| h.get("backend"))
                    .and_then(|b| b.as_str())
                    .unwrap_or("unknown");
                info!("Hardware detected (backend: {})", backend);
            }
            Err(e) => tracing::warn!("Failed to detect hardware: {}", e),
        }

        info!("Rimuru worker is ready");
        Ok(())
    }

    pub fn iii(&self) -> &III {
        &self.iii
    }

    pub async fn shutdown(&self) {
        info!("Shutting down rimuru worker");
        self.iii.shutdown_async().await;
    }
}
