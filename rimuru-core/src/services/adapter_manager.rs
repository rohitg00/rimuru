use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use tokio::sync::RwLock;
use tokio::time::interval;
use tracing::{debug, error, info, warn};

use crate::adapters::{
    AdapterRegistry, AdapterStatus, AgentAdapter, ClaudeCodeAdapter, ClaudeCodeConfig,
    CodexAdapter, CodexConfig, CopilotAdapter, CopilotConfig, CursorAdapter, CursorConfig,
    FullAdapter, GooseAdapter, GooseConfig, OpenCodeAdapter, OpenCodeConfig,
};
use crate::error::{RimuruError, RimuruResult};
use crate::models::AgentType;

use super::cost_aggregator::CostAggregator;
use super::session_aggregator::SessionAggregator;

#[derive(Debug, Clone)]
pub struct AdapterManagerConfig {
    pub auto_discover: bool,
    pub health_check_interval_secs: u64,
    pub reconnect_on_failure: bool,
    pub max_reconnect_attempts: u32,
}

impl Default for AdapterManagerConfig {
    fn default() -> Self {
        Self {
            auto_discover: true,
            health_check_interval_secs: 60,
            reconnect_on_failure: true,
            max_reconnect_attempts: 3,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AdapterHealth {
    pub name: String,
    pub agent_type: AgentType,
    pub status: AdapterStatus,
    pub healthy: bool,
    pub last_check: DateTime<Utc>,
    pub consecutive_failures: u32,
    pub error_message: Option<String>,
}

pub struct AdapterManager {
    registry: Arc<AdapterRegistry>,
    config: AdapterManagerConfig,
    cost_aggregator: Arc<CostAggregator>,
    session_aggregator: Arc<SessionAggregator>,
    health_status: Arc<RwLock<HashMap<String, AdapterHealth>>>,
    running: Arc<RwLock<bool>>,
}

impl AdapterManager {
    pub fn new(config: AdapterManagerConfig) -> Self {
        let registry = Arc::new(AdapterRegistry::new());
        Self {
            registry: Arc::clone(&registry),
            config,
            cost_aggregator: Arc::new(CostAggregator::new(Arc::clone(&registry))),
            session_aggregator: Arc::new(SessionAggregator::new(Arc::clone(&registry))),
            health_status: Arc::new(RwLock::new(HashMap::new())),
            running: Arc::new(RwLock::new(false)),
        }
    }

    pub fn with_default_config() -> Self {
        Self::new(AdapterManagerConfig::default())
    }

    pub async fn initialize(&self) -> RimuruResult<()> {
        info!("Initializing adapter manager");

        if self.config.auto_discover {
            self.discover_and_register_adapters().await?;
        }

        self.connect_all_adapters().await?;

        Ok(())
    }

    pub async fn start_health_monitoring(&self) {
        let registry = Arc::clone(&self.registry);
        let health_status = Arc::clone(&self.health_status);
        let running = Arc::clone(&self.running);
        let interval_secs = self.config.health_check_interval_secs;
        let reconnect_on_failure = self.config.reconnect_on_failure;
        let max_attempts = self.config.max_reconnect_attempts;

        *running.write().await = true;

        tokio::spawn(async move {
            let mut check_interval = interval(Duration::from_secs(interval_secs));

            loop {
                check_interval.tick().await;

                if !*running.read().await {
                    info!("Health monitoring stopped");
                    break;
                }

                let adapter_names = registry.list_names().await;

                for name in adapter_names {
                    if let Some(adapter) = registry.get(&name).await {
                        let adapter_guard = adapter.read().await;
                        let agent_type = adapter_guard.agent_type();
                        let status = adapter_guard.get_status().await;
                        let health_result = adapter_guard.health_check().await;
                        drop(adapter_guard);

                        let (healthy, error_msg) = match health_result {
                            Ok(h) => (h, None),
                            Err(e) => (false, Some(e.to_string())),
                        };

                        let mut health_map = health_status.write().await;
                        let previous_failures = health_map
                            .get(&name)
                            .map(|h| h.consecutive_failures)
                            .unwrap_or(0);

                        let consecutive_failures = if healthy { 0 } else { previous_failures + 1 };

                        health_map.insert(
                            name.clone(),
                            AdapterHealth {
                                name: name.clone(),
                                agent_type,
                                status,
                                healthy,
                                last_check: Utc::now(),
                                consecutive_failures,
                                error_message: error_msg.clone(),
                            },
                        );
                        drop(health_map);

                        if !healthy {
                            warn!(
                                "Adapter '{}' health check failed (attempt {}): {:?}",
                                name, consecutive_failures, error_msg
                            );

                            if reconnect_on_failure && consecutive_failures <= max_attempts {
                                debug!("Attempting to reconnect adapter '{}'", name);
                                let adapter = registry.get(&name).await;
                                if let Some(adapter) = adapter {
                                    let mut adapter_guard = adapter.write().await;
                                    if let Err(e) = adapter_guard.connect().await {
                                        error!("Failed to reconnect adapter '{}': {}", name, e);
                                    } else {
                                        info!("Successfully reconnected adapter '{}'", name);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });
    }

    pub async fn stop_health_monitoring(&self) {
        *self.running.write().await = false;
    }

    pub async fn discover_and_register_adapters(&self) -> RimuruResult<Vec<String>> {
        let mut registered = Vec::new();

        let name = "claude-code";
        let claude_adapter = ClaudeCodeAdapter::new(name, ClaudeCodeConfig::default());
        if claude_adapter.is_installed().await && self.registry.get(name).await.is_none() {
            self.registry
                .register(name.to_string(), claude_adapter)
                .await?;
            registered.push(name.to_string());
            info!("Discovered and registered Claude Code adapter");
        }

        let name = "codex";
        let codex_adapter = CodexAdapter::new(name, CodexConfig::default());
        if codex_adapter.is_installed().await && self.registry.get(name).await.is_none() {
            self.registry
                .register(name.to_string(), codex_adapter)
                .await?;
            registered.push(name.to_string());
            info!("Discovered and registered Codex adapter");
        }

        let name = "copilot";
        let copilot_adapter = CopilotAdapter::new(name, CopilotConfig::default());
        if copilot_adapter.is_installed().await && self.registry.get(name).await.is_none() {
            self.registry
                .register(name.to_string(), copilot_adapter)
                .await?;
            registered.push(name.to_string());
            info!("Discovered and registered GitHub Copilot adapter");
        }

        let name = "cursor";
        let cursor_adapter = CursorAdapter::new(name, CursorConfig::default());
        if cursor_adapter.is_installed().await && self.registry.get(name).await.is_none() {
            self.registry
                .register(name.to_string(), cursor_adapter)
                .await?;
            registered.push(name.to_string());
            info!("Discovered and registered Cursor adapter");
        }

        let name = "goose";
        let goose_adapter = GooseAdapter::new(name, GooseConfig::default());
        if goose_adapter.is_installed().await && self.registry.get(name).await.is_none() {
            self.registry
                .register(name.to_string(), goose_adapter)
                .await?;
            registered.push(name.to_string());
            info!("Discovered and registered Goose adapter");
        }

        let name = "opencode";
        let opencode_adapter = OpenCodeAdapter::new(name, OpenCodeConfig::default());
        if opencode_adapter.is_installed().await && self.registry.get(name).await.is_none() {
            self.registry
                .register(name.to_string(), opencode_adapter)
                .await?;
            registered.push(name.to_string());
            info!("Discovered and registered OpenCode adapter");
        }

        Ok(registered)
    }

    pub async fn register_adapter<A>(&self, name: String, adapter: A) -> RimuruResult<()>
    where
        A: FullAdapter + 'static,
    {
        self.registry.register(name.clone(), adapter).await?;
        info!("Registered adapter '{}'", name);
        Ok(())
    }

    pub async fn unregister_adapter(&self, name: &str) -> RimuruResult<()> {
        if let Some(adapter) = self.registry.get(name).await {
            let mut adapter_guard = adapter.write().await;
            let _ = adapter_guard.disconnect().await;
        }

        self.registry.unregister(name).await?;
        self.health_status.write().await.remove(name);

        info!("Unregistered adapter '{}'", name);
        Ok(())
    }

    pub async fn connect_all_adapters(&self) -> RimuruResult<Vec<(String, RimuruResult<()>)>> {
        let results = self.registry.connect_all().await;
        let mut converted_results = Vec::new();

        for (name, result) in results {
            if result.is_ok() {
                info!("Connected adapter '{}'", name);
            } else {
                warn!("Failed to connect adapter '{}': {:?}", name, result);
            }
            converted_results.push((name, result));
        }

        Ok(converted_results)
    }

    pub async fn disconnect_all_adapters(&self) -> RimuruResult<Vec<(String, RimuruResult<()>)>> {
        let results = self.registry.disconnect_all().await;
        let mut converted_results = Vec::new();

        for (name, result) in results {
            if result.is_ok() {
                info!("Disconnected adapter '{}'", name);
            }
            converted_results.push((name, result));
        }

        Ok(converted_results)
    }

    pub async fn get_adapter_status(&self, name: &str) -> RimuruResult<AdapterStatus> {
        let adapter = self
            .registry
            .get(name)
            .await
            .ok_or_else(|| RimuruError::AgentNotFound(name.to_string()))?;

        let adapter_guard = adapter.read().await;
        Ok(adapter_guard.get_status().await)
    }

    pub async fn get_all_statuses(&self) -> HashMap<String, AdapterStatus> {
        self.registry.get_all_statuses().await
    }

    pub async fn get_health_status(&self) -> HashMap<String, AdapterHealth> {
        self.health_status.read().await.clone()
    }

    pub async fn get_adapter_health(&self, name: &str) -> Option<AdapterHealth> {
        self.health_status.read().await.get(name).cloned()
    }

    pub async fn list_adapters(&self) -> Vec<String> {
        self.registry.list_names().await
    }

    pub async fn list_adapters_by_type(&self) -> HashMap<AgentType, Vec<String>> {
        self.registry.list_by_type().await
    }

    pub async fn get_adapters_by_type(&self, agent_type: AgentType) -> Vec<String> {
        self.registry
            .list_by_type()
            .await
            .get(&agent_type)
            .cloned()
            .unwrap_or_default()
    }

    pub async fn adapter_count(&self) -> usize {
        self.registry.count().await
    }

    pub fn cost_aggregator(&self) -> &CostAggregator {
        &self.cost_aggregator
    }

    pub fn session_aggregator(&self) -> &SessionAggregator {
        &self.session_aggregator
    }

    pub fn registry(&self) -> &AdapterRegistry {
        &self.registry
    }

    pub async fn run_health_check(&self, name: &str) -> RimuruResult<bool> {
        let adapter = self
            .registry
            .get(name)
            .await
            .ok_or_else(|| RimuruError::AgentNotFound(name.to_string()))?;

        let adapter_guard = adapter.read().await;
        let result = adapter_guard.health_check().await?;

        let mut health_map = self.health_status.write().await;
        if let Some(health) = health_map.get_mut(name) {
            health.healthy = result;
            health.last_check = Utc::now();
            health.consecutive_failures = if result {
                0
            } else {
                health.consecutive_failures + 1
            };
        }

        Ok(result)
    }

    pub async fn run_all_health_checks(&self) -> HashMap<String, bool> {
        self.registry.health_check_all().await
    }

    pub async fn reconnect_adapter(&self, name: &str) -> RimuruResult<()> {
        let adapter = self
            .registry
            .get(name)
            .await
            .ok_or_else(|| RimuruError::AgentNotFound(name.to_string()))?;

        let mut adapter_guard = adapter.write().await;
        adapter_guard.disconnect().await?;
        adapter_guard.connect().await?;

        info!("Reconnected adapter '{}'", name);
        Ok(())
    }

    pub async fn shutdown(&self) -> RimuruResult<()> {
        info!("Shutting down adapter manager");

        self.stop_health_monitoring().await;

        self.disconnect_all_adapters().await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::types::{ActiveSession, SessionHistory, UsageStats};
    use crate::adapters::{CostTracker, SessionEventCallback, SessionMonitor};
    use crate::models::{ModelInfo, Session};
    use async_trait::async_trait;
    use std::sync::Mutex;
    use uuid::Uuid;

    struct TestAdapter {
        name: String,
        agent_type: AgentType,
        status: Mutex<AdapterStatus>,
        installed: bool,
    }

    impl TestAdapter {
        fn new(name: &str, agent_type: AgentType) -> Self {
            Self {
                name: name.to_string(),
                agent_type,
                status: Mutex::new(AdapterStatus::Disconnected),
                installed: true,
            }
        }
    }

    #[async_trait]
    impl AgentAdapter for TestAdapter {
        fn agent_type(&self) -> AgentType {
            self.agent_type
        }

        fn name(&self) -> &str {
            &self.name
        }

        async fn connect(&mut self) -> RimuruResult<()> {
            *self.status.lock().unwrap() = AdapterStatus::Connected;
            Ok(())
        }

        async fn disconnect(&mut self) -> RimuruResult<()> {
            *self.status.lock().unwrap() = AdapterStatus::Disconnected;
            Ok(())
        }

        async fn get_status(&self) -> AdapterStatus {
            *self.status.lock().unwrap()
        }

        async fn get_info(&self) -> RimuruResult<crate::adapters::AdapterInfo> {
            Ok(crate::adapters::AdapterInfo::new(
                self.name.clone(),
                self.agent_type,
            ))
        }

        async fn get_sessions(&self) -> RimuruResult<Vec<Session>> {
            Ok(vec![])
        }

        async fn get_active_session(&self) -> RimuruResult<Option<ActiveSession>> {
            Ok(None)
        }

        async fn is_installed(&self) -> bool {
            self.installed
        }

        async fn health_check(&self) -> RimuruResult<bool> {
            Ok(*self.status.lock().unwrap() == AdapterStatus::Connected)
        }
    }

    #[async_trait]
    impl CostTracker for TestAdapter {
        async fn get_usage(&self, _since: Option<DateTime<Utc>>) -> RimuruResult<UsageStats> {
            Ok(UsageStats {
                input_tokens: 100,
                output_tokens: 50,
                requests: 5,
                model_name: None,
                period_start: None,
                period_end: None,
            })
        }

        async fn calculate_cost(
            &self,
            input_tokens: i64,
            output_tokens: i64,
            _model_name: &str,
        ) -> RimuruResult<f64> {
            Ok((input_tokens + output_tokens) as f64 * 0.00001)
        }

        async fn get_model_info(&self, model_name: &str) -> RimuruResult<Option<ModelInfo>> {
            Ok(Some(ModelInfo::new(
                "test".to_string(),
                model_name.to_string(),
                0.01,
                0.03,
                128000,
            )))
        }

        async fn get_supported_models(&self) -> RimuruResult<Vec<String>> {
            Ok(vec!["test-model".to_string()])
        }

        async fn get_total_cost(&self, _since: Option<DateTime<Utc>>) -> RimuruResult<f64> {
            Ok(0.01)
        }
    }

    #[async_trait]
    impl SessionMonitor for TestAdapter {
        async fn subscribe_to_events(&self, _callback: SessionEventCallback) -> RimuruResult<Uuid> {
            Ok(Uuid::new_v4())
        }

        async fn unsubscribe(&self, _subscription_id: Uuid) -> RimuruResult<()> {
            Ok(())
        }

        async fn get_session_history(
            &self,
            _limit: Option<usize>,
            _since: Option<DateTime<Utc>>,
        ) -> RimuruResult<Vec<SessionHistory>> {
            Ok(vec![])
        }

        async fn get_session_details(
            &self,
            _session_id: Uuid,
        ) -> RimuruResult<Option<SessionHistory>> {
            Ok(None)
        }

        async fn get_active_sessions(&self) -> RimuruResult<Vec<ActiveSession>> {
            Ok(vec![])
        }
    }

    #[tokio::test]
    async fn test_adapter_manager_creation() {
        let manager = AdapterManager::with_default_config();
        assert_eq!(manager.adapter_count().await, 0);
    }

    #[tokio::test]
    async fn test_register_and_list_adapters() {
        let manager = AdapterManager::with_default_config();

        manager
            .register_adapter(
                "test-claude".to_string(),
                TestAdapter::new("test-claude", AgentType::ClaudeCode),
            )
            .await
            .unwrap();

        manager
            .register_adapter(
                "test-opencode".to_string(),
                TestAdapter::new("test-opencode", AgentType::OpenCode),
            )
            .await
            .unwrap();

        let adapters = manager.list_adapters().await;
        assert_eq!(adapters.len(), 2);
        assert!(adapters.contains(&"test-claude".to_string()));
        assert!(adapters.contains(&"test-opencode".to_string()));
    }

    #[tokio::test]
    async fn test_connect_and_disconnect_adapters() {
        let manager = AdapterManager::with_default_config();

        manager
            .register_adapter(
                "test-adapter".to_string(),
                TestAdapter::new("test-adapter", AgentType::ClaudeCode),
            )
            .await
            .unwrap();

        let status = manager.get_adapter_status("test-adapter").await.unwrap();
        assert_eq!(status, AdapterStatus::Disconnected);

        manager.connect_all_adapters().await.unwrap();

        let status = manager.get_adapter_status("test-adapter").await.unwrap();
        assert_eq!(status, AdapterStatus::Connected);

        manager.disconnect_all_adapters().await.unwrap();

        let status = manager.get_adapter_status("test-adapter").await.unwrap();
        assert_eq!(status, AdapterStatus::Disconnected);
    }

    #[tokio::test]
    async fn test_get_all_statuses() {
        let manager = AdapterManager::with_default_config();

        manager
            .register_adapter(
                "adapter-1".to_string(),
                TestAdapter::new("adapter-1", AgentType::ClaudeCode),
            )
            .await
            .unwrap();

        manager
            .register_adapter(
                "adapter-2".to_string(),
                TestAdapter::new("adapter-2", AgentType::OpenCode),
            )
            .await
            .unwrap();

        let statuses = manager.get_all_statuses().await;
        assert_eq!(statuses.len(), 2);
        assert_eq!(
            *statuses.get("adapter-1").unwrap(),
            AdapterStatus::Disconnected
        );
        assert_eq!(
            *statuses.get("adapter-2").unwrap(),
            AdapterStatus::Disconnected
        );
    }

    #[tokio::test]
    async fn test_list_by_type() {
        let manager = AdapterManager::with_default_config();

        manager
            .register_adapter(
                "claude-1".to_string(),
                TestAdapter::new("claude-1", AgentType::ClaudeCode),
            )
            .await
            .unwrap();

        manager
            .register_adapter(
                "claude-2".to_string(),
                TestAdapter::new("claude-2", AgentType::ClaudeCode),
            )
            .await
            .unwrap();

        manager
            .register_adapter(
                "opencode-1".to_string(),
                TestAdapter::new("opencode-1", AgentType::OpenCode),
            )
            .await
            .unwrap();

        let by_type = manager.list_adapters_by_type().await;
        assert_eq!(by_type.get(&AgentType::ClaudeCode).unwrap().len(), 2);
        assert_eq!(by_type.get(&AgentType::OpenCode).unwrap().len(), 1);

        let claude_adapters = manager.get_adapters_by_type(AgentType::ClaudeCode).await;
        assert_eq!(claude_adapters.len(), 2);
    }

    #[tokio::test]
    async fn test_unregister_adapter() {
        let manager = AdapterManager::with_default_config();

        manager
            .register_adapter(
                "test-adapter".to_string(),
                TestAdapter::new("test-adapter", AgentType::ClaudeCode),
            )
            .await
            .unwrap();

        assert_eq!(manager.adapter_count().await, 1);

        manager.unregister_adapter("test-adapter").await.unwrap();

        assert_eq!(manager.adapter_count().await, 0);
    }

    #[tokio::test]
    async fn test_run_health_check() {
        let manager = AdapterManager::with_default_config();

        manager
            .register_adapter(
                "test-adapter".to_string(),
                TestAdapter::new("test-adapter", AgentType::ClaudeCode),
            )
            .await
            .unwrap();

        let healthy = manager.run_health_check("test-adapter").await.unwrap();
        assert!(!healthy);

        manager.connect_all_adapters().await.unwrap();

        let healthy = manager.run_health_check("test-adapter").await.unwrap();
        assert!(healthy);
    }

    #[tokio::test]
    async fn test_reconnect_adapter() {
        let manager = AdapterManager::with_default_config();

        manager
            .register_adapter(
                "test-adapter".to_string(),
                TestAdapter::new("test-adapter", AgentType::ClaudeCode),
            )
            .await
            .unwrap();

        manager.connect_all_adapters().await.unwrap();

        manager.reconnect_adapter("test-adapter").await.unwrap();

        let status = manager.get_adapter_status("test-adapter").await.unwrap();
        assert_eq!(status, AdapterStatus::Connected);
    }

    #[tokio::test]
    async fn test_run_all_health_checks() {
        let manager = AdapterManager::with_default_config();

        manager
            .register_adapter(
                "adapter-1".to_string(),
                TestAdapter::new("adapter-1", AgentType::ClaudeCode),
            )
            .await
            .unwrap();

        manager
            .register_adapter(
                "adapter-2".to_string(),
                TestAdapter::new("adapter-2", AgentType::OpenCode),
            )
            .await
            .unwrap();

        let health = manager.run_all_health_checks().await;
        assert_eq!(health.len(), 2);
        assert!(!*health.get("adapter-1").unwrap());
        assert!(!*health.get("adapter-2").unwrap());
    }

    #[tokio::test]
    async fn test_cost_and_session_aggregators() {
        let manager = AdapterManager::with_default_config();

        manager
            .register_adapter(
                "test-adapter".to_string(),
                TestAdapter::new("test-adapter", AgentType::ClaudeCode),
            )
            .await
            .unwrap();

        let _ = manager.cost_aggregator();
        let _ = manager.session_aggregator();
        let _ = manager.registry();
    }
}
