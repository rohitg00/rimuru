use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::RwLock;
use tracing::info;

use iii_sdk::{III, IIIError, TriggerConfig, TriggerHandler};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRoute {
    pub method: String,
    pub path: String,
    pub function_id: String,
}

pub struct HttpTriggerHandler {
    routes: Arc<RwLock<HashMap<String, HttpRoute>>>,
}

impl HttpTriggerHandler {
    pub fn new() -> Self {
        Self {
            routes: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl TriggerHandler for HttpTriggerHandler {
    async fn register_trigger(&self, config: TriggerConfig) -> Result<(), IIIError> {
        let method = config
            .config
            .get("method")
            .and_then(|v| v.as_str())
            .unwrap_or("GET")
            .to_uppercase();
        let path = config
            .config
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap_or("/")
            .to_string();

        let route = HttpRoute {
            method: method.clone(),
            path: path.clone(),
            function_id: config.function_id.clone(),
        };

        self.routes.write().await.insert(config.id.clone(), route);
        info!(
            "Registered HTTP trigger: {} {} -> {}",
            method, path, config.function_id
        );
        Ok(())
    }

    async fn unregister_trigger(&self, config: TriggerConfig) -> Result<(), IIIError> {
        self.routes.write().await.remove(&config.id);
        info!("Unregistered HTTP trigger: {}", config.id);
        Ok(())
    }
}

struct Route {
    method: &'static str,
    path: &'static str,
    function_id: &'static str,
}

const ROUTES: &[Route] = &[
    Route { method: "GET", path: "/api/agents", function_id: "rimuru.agents.list" },
    Route { method: "GET", path: "/api/agents/:id", function_id: "rimuru.agents.get" },
    Route { method: "POST", path: "/api/agents", function_id: "rimuru.agents.register" },
    Route { method: "POST", path: "/api/agents/:id/connect", function_id: "rimuru.agents.connect" },
    Route { method: "POST", path: "/api/agents/:id/disconnect", function_id: "rimuru.agents.disconnect" },
    Route { method: "GET", path: "/api/agents/detect", function_id: "rimuru.agents.detect" },
    Route { method: "GET", path: "/api/sessions", function_id: "rimuru.sessions.list" },
    Route { method: "GET", path: "/api/sessions/:id", function_id: "rimuru.sessions.get" },
    Route { method: "GET", path: "/api/sessions/active", function_id: "rimuru.sessions.active" },
    Route { method: "GET", path: "/api/costs/summary", function_id: "rimuru.costs.summary" },
    Route { method: "GET", path: "/api/costs/daily", function_id: "rimuru.costs.daily" },
    Route { method: "GET", path: "/api/costs/agent/:id", function_id: "rimuru.costs.by_agent" },
    Route { method: "POST", path: "/api/costs", function_id: "rimuru.costs.record" },
    Route { method: "GET", path: "/api/models", function_id: "rimuru.models.list" },
    Route { method: "POST", path: "/api/models/sync", function_id: "rimuru.models.sync" },
    Route { method: "GET", path: "/api/metrics", function_id: "rimuru.metrics.current" },
    Route { method: "GET", path: "/api/metrics/history", function_id: "rimuru.metrics.history" },
    Route { method: "GET", path: "/api/health", function_id: "rimuru.health.check" },
    Route { method: "GET", path: "/api/hooks", function_id: "rimuru.hooks.list" },
    Route { method: "POST", path: "/api/hooks/dispatch", function_id: "rimuru.hooks.dispatch" },
    Route { method: "GET", path: "/api/plugins", function_id: "rimuru.plugins.list" },
    Route { method: "POST", path: "/api/plugins/install", function_id: "rimuru.plugins.install" },
];

pub fn register(iii: &III) {
    let handler = HttpTriggerHandler::new();

    iii.register_trigger_type("http", "HTTP request triggers for REST API endpoints", handler);
    info!("Registered HTTP trigger type");

    for route in ROUTES {
        let config = json!({
            "method": route.method,
            "path": route.path,
        });

        match iii.register_trigger("http", route.function_id, config) {
            Ok(_trigger) => {
                info!(
                    "Registered HTTP trigger: {} {} -> {}",
                    route.method, route.path, route.function_id
                );
            }
            Err(e) => {
                tracing::error!(
                    "Failed to register HTTP trigger {} {} -> {}: {}",
                    route.method,
                    route.path,
                    route.function_id,
                    e
                );
            }
        }
    }

    info!("Registered {} HTTP API triggers", ROUTES.len());
}
