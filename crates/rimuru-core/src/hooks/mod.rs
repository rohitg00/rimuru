pub mod types;

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use iii_sdk::TriggerRequest;
use serde_json::Value;
use tracing::{info, warn};

use crate::error::RimuruError;
use types::HookEvent;

type Result<T> = std::result::Result<T, RimuruError>;

const HOOK_TIMEOUT_MS: u64 = 15_000;

#[derive(Clone)]
pub struct HookRegistry {
    handlers: Arc<RwLock<HashMap<String, Vec<HookHandler>>>>,
}

#[derive(Clone)]
struct HookHandler {
    id: String,
    function_id: String,
    priority: i32,
}

impl Default for HookRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl HookRegistry {
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn register(&self, event_type: &str, handler_id: &str, function_id: &str, priority: i32) {
        let handler = HookHandler {
            id: handler_id.to_string(),
            function_id: function_id.to_string(),
            priority,
        };

        let mut map = self.handlers.write().unwrap_or_else(|e| e.into_inner());
        let entry = map.entry(event_type.to_string()).or_default();
        entry.push(handler);
        entry.sort_by(|a, b| b.priority.cmp(&a.priority));

        info!(
            "Registered hook handler '{}' for event '{}'",
            handler_id, event_type
        );
    }

    pub fn unregister(&self, event_type: &str, handler_id: &str) {
        let mut map = self.handlers.write().unwrap_or_else(|e| e.into_inner());
        if let Some(handlers) = map.get_mut(event_type) {
            handlers.retain(|h| h.id != handler_id);
        }
    }

    pub fn get_handlers(&self, event_type: &str) -> Vec<String> {
        let map = self.handlers.read().unwrap_or_else(|e| e.into_inner());
        map.get(event_type)
            .map(|h| {
                h.iter()
                    .map(|handler| handler.function_id.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn list_all(&self) -> Vec<(String, Vec<String>)> {
        let map = self.handlers.read().unwrap_or_else(|e| e.into_inner());
        map.iter()
            .map(|(event, handlers)| {
                let fns = handlers.iter().map(|h| h.function_id.clone()).collect();
                (event.clone(), fns)
            })
            .collect()
    }

    pub async fn dispatch(&self, iii: &iii_sdk::III, event: HookEvent) -> Result<Vec<Value>> {
        let event_type = event.event_type();
        let payload = serde_json::to_value(&event)?;

        let handler_fns = self.get_handlers(event_type);
        if handler_fns.is_empty() {
            return Ok(vec![]);
        }

        let mut results = Vec::new();
        for function_id in handler_fns {
            match iii
                .trigger(TriggerRequest {
                    function_id: function_id.clone(),
                    payload: payload.clone(),
                    action: None,
                    timeout_ms: Some(HOOK_TIMEOUT_MS),
                })
                .await
            {
                Ok(result) => results.push(result),
                Err(e) => {
                    warn!(
                        "Hook handler '{}' failed for '{}': {}",
                        function_id, event_type, e
                    );
                }
            }
        }

        Ok(results)
    }
}
