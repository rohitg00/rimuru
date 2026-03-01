use iii_sdk::{III, WorkerMetadata};
use tracing::info;

use crate::error::RimuruError;
use crate::state::StateKV;
use crate::{functions, triggers};

pub struct RimuruWorker {
    iii: III,
    kv: StateKV,
    engine_url: String,
}

impl RimuruWorker {
    pub fn new(engine_url: &str) -> Self {
        let iii = III::with_metadata(
            engine_url,
            WorkerMetadata {
                runtime: "rust".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                name: "rimuru-core".to_string(),
                os: std::env::consts::OS.to_string(),
                telemetry: None,
            },
        );
        let kv = StateKV::new(iii.clone());

        Self {
            iii,
            kv,
            engine_url: engine_url.to_string(),
        }
    }

    pub async fn start(&self) -> Result<(), RimuruError> {
        info!("Connecting to iii engine at {}", self.engine_url);
        self.iii
            .connect()
            .await
            .map_err(|e: iii_sdk::IIIError| RimuruError::Bridge(e.to_string()))?;
        info!("Connected to iii engine");

        functions::register_all(&self.iii, &self.kv);
        info!("Registered all functions");

        triggers::register_all(&self.iii);
        info!("Registered all triggers");

        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        match self
            .iii
            .trigger(
                "rimuru.agents.detect",
                serde_json::json!({"auto_register": true}),
            )
            .await
        {
            Ok(result) => {
                let total = result
                    .get("total")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                info!("Auto-detected and registered {} agents", total);
            }
            Err(e) => {
                tracing::warn!("Failed to auto-detect agents: {}", e);
            }
        }

        match self
            .iii
            .trigger("rimuru.agents.sync", serde_json::json!({}))
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
                info!("Synced {} sessions, {} cost records from disk", sessions, costs);
            }
            Err(e) => {
                tracing::warn!("Failed to sync agent data: {}", e);
            }
        }

        match self
            .iii
            .trigger("rimuru.metrics.collect", serde_json::json!({}))
            .await
        {
            Ok(_) => info!("Initial metrics collected"),
            Err(e) => tracing::warn!("Failed to collect initial metrics: {}", e),
        }

        info!("Rimuru worker is ready");
        Ok(())
    }

    pub fn iii(&self) -> &III {
        &self.iii
    }

    pub fn kv(&self) -> &StateKV {
        &self.kv
    }

    pub async fn shutdown(&self) {
        info!("Shutting down rimuru worker");
        self.iii.shutdown_async().await;
    }
}
