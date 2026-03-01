pub mod types;

use std::sync::Arc;

use dashmap::DashMap;
use serde_json::Value;
use tracing::{info, warn};

use crate::error::RimuruError;
use types::HookEvent;

type Result<T> = std::result::Result<T, RimuruError>;

#[derive(Clone)]
pub struct HookRegistry {
    handlers: Arc<DashMap<String, Vec<HookHandler>>>,
}

#[derive(Clone)]
struct HookHandler {
    id: String,
    function_id: String,
    priority: i32,
}

impl HookRegistry {
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(DashMap::new()),
        }
    }

    pub fn register(&self, event_type: &str, handler_id: &str, function_id: &str, priority: i32) {
        let handler = HookHandler {
            id: handler_id.to_string(),
            function_id: function_id.to_string(),
            priority,
        };

        self.handlers
            .entry(event_type.to_string())
            .or_default()
            .push(handler);

        if let Some(mut handlers) = self.handlers.get_mut(event_type) {
            handlers.sort_by(|a, b| b.priority.cmp(&a.priority));
        }

        info!("Registered hook handler '{}' for event '{}'", handler_id, event_type);
    }

    pub fn unregister(&self, event_type: &str, handler_id: &str) {
        if let Some(mut handlers) = self.handlers.get_mut(event_type) {
            handlers.retain(|h| h.id != handler_id);
        }
    }

    pub fn get_handlers(&self, event_type: &str) -> Vec<String> {
        self.handlers
            .get(event_type)
            .map(|h| h.iter().map(|handler| handler.function_id.clone()).collect())
            .unwrap_or_default()
    }

    pub fn list_all(&self) -> Vec<(String, Vec<String>)> {
        self.handlers
            .iter()
            .map(|entry| {
                let event = entry.key().clone();
                let fns = entry.value().iter().map(|h| h.function_id.clone()).collect();
                (event, fns)
            })
            .collect()
    }

    pub async fn dispatch(
        &self,
        iii: &iii_sdk::III,
        event: HookEvent,
    ) -> Result<Vec<Value>> {
        let event_type = event.event_type();
        let payload = serde_json::to_value(&event)?;

        let handler_fns = self.get_handlers(event_type);
        if handler_fns.is_empty() {
            return Ok(vec![]);
        }

        let mut results = Vec::new();
        for function_id in handler_fns {
            match iii.trigger(&function_id, payload.clone()).await {
                Ok(result) => results.push(result),
                Err(e) => {
                    warn!("Hook handler '{}' failed for '{}': {}", function_id, event_type, e);
                }
            }
        }

        Ok(results)
    }
}
