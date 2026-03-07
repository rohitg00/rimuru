use std::sync::Arc;

use iii_sdk::III;
use rimuru_core::StateKV;
use serde_json::Value;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct AppState {
    pub iii: III,
    #[allow(dead_code)]
    pub kv: StateKV,
    pub api_port: u16,
    pub last_sync: Arc<RwLock<Option<chrono::DateTime<chrono::Utc>>>>,
}

impl AppState {
    pub fn new(iii: III, kv: StateKV, api_port: u16) -> Self {
        Self {
            iii,
            kv,
            api_port,
            last_sync: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn call(&self, function_id: &str, input: Value) -> Result<Value, String> {
        self.iii
            .trigger(function_id, input)
            .await
            .map_err(|e| format!("{}: {}", function_id, e))
    }

    pub async fn call_extract(&self, function_id: &str, input: Value, field: &str) -> Result<Value, String> {
        let result = self.call(function_id, input).await?;
        Ok(result.get(field).cloned().unwrap_or(result))
    }
}
