use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Utc};
use tokio::sync::RwLock;
use tokio::time::sleep;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::error::{RimuruError, RimuruResult};
use crate::models::AgentType;

use super::factory::{create_adapter, create_adapter_with_config, AdapterConfig};
use super::traits::FullAdapter;
use super::types::{ActiveSession, AdapterInfo, AdapterStatus, SessionHistory, UsageStats};

type BoxedAdapter = Box<dyn FullAdapter>;

pub struct AdapterRegistry {
    adapters: RwLock<HashMap<String, Arc<RwLock<BoxedAdapter>>>>,
    type_index: RwLock<HashMap<AgentType, Vec<String>>>,
    max_retry_attempts: u32,
    retry_delay_secs: u64,
}

impl AdapterRegistry {
    pub fn new() -> Self {
        Self {
            adapters: RwLock::new(HashMap::new()),
            type_index: RwLock::new(HashMap::new()),
            max_retry_attempts: 3,
            retry_delay_secs: 5,
        }
    }

    pub fn with_retry_config(mut self, max_attempts: u32, delay_secs: u64) -> Self {
        self.max_retry_attempts = max_attempts;
        self.retry_delay_secs = delay_secs;
        self
    }

    pub async fn register<A>(&self, name: String, adapter: A) -> RimuruResult<()>
    where
        A: FullAdapter + 'static,
    {
        let agent_type = adapter.agent_type();

        let mut adapters = self.adapters.write().await;
        if adapters.contains_key(&name) {
            return Err(RimuruError::AgentAlreadyExists(name));
        }

        adapters.insert(name.clone(), Arc::new(RwLock::new(Box::new(adapter))));

        let mut type_index = self.type_index.write().await;
        type_index.entry(agent_type).or_default().push(name);

        Ok(())
    }

    pub async fn register_from_factory(
        &self,
        name: String,
        agent_type: AgentType,
    ) -> RimuruResult<()> {
        let adapter = create_adapter(&name, agent_type);

        let mut adapters = self.adapters.write().await;
        if adapters.contains_key(&name) {
            return Err(RimuruError::AgentAlreadyExists(name));
        }

        adapters.insert(name.clone(), Arc::new(RwLock::new(adapter)));

        let mut type_index = self.type_index.write().await;
        type_index.entry(agent_type).or_default().push(name.clone());

        info!(
            "Registered adapter '{}' of type {:?} via factory",
            name, agent_type
        );
        Ok(())
    }

    pub async fn register_with_config(
        &self,
        name: String,
        config: AdapterConfig,
    ) -> RimuruResult<()> {
        let agent_type = config.agent_type();
        let adapter = create_adapter_with_config(&name, config)?;

        let mut adapters = self.adapters.write().await;
        if adapters.contains_key(&name) {
            return Err(RimuruError::AgentAlreadyExists(name));
        }

        adapters.insert(name.clone(), Arc::new(RwLock::new(adapter)));

        let mut type_index = self.type_index.write().await;
        type_index.entry(agent_type).or_default().push(name.clone());

        info!(
            "Registered adapter '{}' of type {:?} with custom config",
            name, agent_type
        );
        Ok(())
    }

    pub async fn unregister(&self, name: &str) -> RimuruResult<()> {
        let mut adapters = self.adapters.write().await;
        let adapter = adapters
            .remove(name)
            .ok_or_else(|| RimuruError::AgentNotFound(name.to_string()))?;

        let adapter_guard = adapter.read().await;
        let agent_type = adapter_guard.agent_type();
        drop(adapter_guard);

        let mut type_index = self.type_index.write().await;
        if let Some(names) = type_index.get_mut(&agent_type) {
            names.retain(|n| n != name);
            if names.is_empty() {
                type_index.remove(&agent_type);
            }
        }

        Ok(())
    }

    pub async fn get(&self, name: &str) -> Option<Arc<RwLock<BoxedAdapter>>> {
        let adapters = self.adapters.read().await;
        adapters.get(name).cloned()
    }

    pub async fn get_by_type(&self, agent_type: AgentType) -> Vec<Arc<RwLock<BoxedAdapter>>> {
        let type_index = self.type_index.read().await;
        let adapters = self.adapters.read().await;

        type_index
            .get(&agent_type)
            .map(|names| {
                names
                    .iter()
                    .filter_map(|name| adapters.get(name).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    pub async fn list_names(&self) -> Vec<String> {
        let adapters = self.adapters.read().await;
        adapters.keys().cloned().collect()
    }

    pub async fn list_by_type(&self) -> HashMap<AgentType, Vec<String>> {
        let type_index = self.type_index.read().await;
        type_index.clone()
    }

    pub async fn count(&self) -> usize {
        let adapters = self.adapters.read().await;
        adapters.len()
    }

    pub async fn connect_all(&self) -> Vec<(String, RimuruResult<()>)> {
        let adapters = self.adapters.read().await;
        let mut results = Vec::new();

        for (name, adapter) in adapters.iter() {
            let mut adapter_guard = adapter.write().await;
            let result = adapter_guard.connect().await;
            results.push((name.clone(), result));
        }

        results
    }

    pub async fn disconnect_all(&self) -> Vec<(String, RimuruResult<()>)> {
        let adapters = self.adapters.read().await;
        let mut results = Vec::new();

        for (name, adapter) in adapters.iter() {
            let mut adapter_guard = adapter.write().await;
            let result = adapter_guard.disconnect().await;
            results.push((name.clone(), result));
        }

        results
    }

    pub async fn get_all_statuses(&self) -> HashMap<String, AdapterStatus> {
        let adapters = self.adapters.read().await;
        let mut statuses = HashMap::new();

        for (name, adapter) in adapters.iter() {
            let adapter_guard = adapter.read().await;
            let status = adapter_guard.get_status().await;
            statuses.insert(name.clone(), status);
        }

        statuses
    }

    pub async fn get_all_info(&self) -> Vec<RimuruResult<AdapterInfo>> {
        let adapters = self.adapters.read().await;
        let mut infos = Vec::new();

        for adapter in adapters.values() {
            let adapter_guard = adapter.read().await;
            infos.push(adapter_guard.get_info().await);
        }

        infos
    }

    pub async fn get_aggregated_usage(
        &self,
        since: Option<DateTime<Utc>>,
    ) -> RimuruResult<UsageStats> {
        let adapters = self.adapters.read().await;
        let mut total = UsageStats::new();

        for adapter in adapters.values() {
            let adapter_guard = adapter.read().await;
            if let Ok(usage) = adapter_guard.get_usage(since).await {
                total.add(&usage);
            }
        }

        Ok(total)
    }

    pub async fn get_aggregated_cost(&self, since: Option<DateTime<Utc>>) -> RimuruResult<f64> {
        let adapters = self.adapters.read().await;
        let mut total_cost = 0.0;

        for adapter in adapters.values() {
            let adapter_guard = adapter.read().await;
            if let Ok(cost) = adapter_guard.get_total_cost(since).await {
                total_cost += cost;
            }
        }

        Ok(total_cost)
    }

    pub async fn get_all_active_sessions(&self) -> Vec<ActiveSession> {
        let adapters = self.adapters.read().await;
        let mut all_sessions = Vec::new();

        for adapter in adapters.values() {
            let adapter_guard = adapter.read().await;
            if let Ok(sessions) = adapter_guard.get_active_sessions().await {
                all_sessions.extend(sessions);
            }
        }

        all_sessions
    }

    pub async fn get_all_session_history(
        &self,
        limit: Option<usize>,
        since: Option<DateTime<Utc>>,
    ) -> Vec<SessionHistory> {
        let adapters = self.adapters.read().await;
        let mut all_history = Vec::new();

        for adapter in adapters.values() {
            let adapter_guard = adapter.read().await;
            if let Ok(history) = adapter_guard.get_session_history(limit, since).await {
                all_history.extend(history);
            }
        }

        all_history.sort_by(|a, b| b.started_at.cmp(&a.started_at));

        if let Some(limit) = limit {
            all_history.truncate(limit);
        }

        all_history
    }

    pub async fn health_check_all(&self) -> HashMap<String, bool> {
        let adapters = self.adapters.read().await;
        let mut results = HashMap::new();

        for (name, adapter) in adapters.iter() {
            let adapter_guard = adapter.read().await;
            let healthy = adapter_guard.health_check().await.unwrap_or(false);
            results.insert(name.clone(), healthy);
        }

        results
    }

    pub async fn connect_with_retry(&self, name: &str) -> RimuruResult<()> {
        let adapter = self
            .get(name)
            .await
            .ok_or_else(|| RimuruError::AgentNotFound(name.to_string()))?;

        for attempt in 1..=self.max_retry_attempts {
            let mut adapter_guard = adapter.write().await;
            match adapter_guard.connect().await {
                Ok(()) => {
                    info!(
                        "Adapter '{}' connected successfully on attempt {}",
                        name, attempt
                    );
                    return Ok(());
                }
                Err(e) => {
                    warn!(
                        "Adapter '{}' connection attempt {} failed: {}",
                        name, attempt, e
                    );

                    if attempt < self.max_retry_attempts {
                        drop(adapter_guard);
                        sleep(Duration::from_secs(self.retry_delay_secs)).await;
                    }
                }
            }
        }

        Err(RimuruError::AgentConnectionFailed {
            agent: name.to_string(),
            message: format!(
                "Failed to connect after {} attempts",
                self.max_retry_attempts
            ),
        })
    }

    pub async fn health_check_with_reconnect(&self, name: &str) -> RimuruResult<bool> {
        let adapter = self
            .get(name)
            .await
            .ok_or_else(|| RimuruError::AgentNotFound(name.to_string()))?;

        {
            let adapter_guard = adapter.read().await;
            match adapter_guard.health_check().await {
                Ok(true) => {
                    debug!("Adapter '{}' health check passed", name);
                    return Ok(true);
                }
                Ok(false) => {
                    warn!("Adapter '{}' health check returned unhealthy", name);
                }
                Err(e) => {
                    warn!("Adapter '{}' health check failed: {}", name, e);
                }
            }

            let status = adapter_guard.get_status().await;
            if status != AdapterStatus::Error && status != AdapterStatus::Disconnected {
                return Ok(false);
            }
        }

        info!("Attempting to reconnect adapter '{}'", name);

        for attempt in 1..=self.max_retry_attempts {
            let mut adapter_guard = adapter.write().await;
            match adapter_guard.connect().await {
                Ok(()) => {
                    info!(
                        "Adapter '{}' reconnected successfully on attempt {}",
                        name, attempt
                    );

                    if let Ok(true) = adapter_guard.health_check().await {
                        return Ok(true);
                    }
                    return Ok(false);
                }
                Err(e) => {
                    warn!(
                        "Adapter '{}' reconnect attempt {} failed: {}",
                        name, attempt, e
                    );

                    if attempt < self.max_retry_attempts {
                        drop(adapter_guard);
                        sleep(Duration::from_secs(self.retry_delay_secs)).await;
                    }
                }
            }
        }

        Ok(false)
    }

    pub async fn connect_all_with_retry(&self) -> Vec<(String, RimuruResult<()>)> {
        let names: Vec<String> = self.list_names().await;
        let mut results = Vec::new();

        for name in names {
            let result = self.connect_with_retry(&name).await;
            results.push((name, result));
        }

        results
    }

    pub async fn health_check_all_with_reconnect(&self) -> HashMap<String, bool> {
        let names: Vec<String> = self.list_names().await;
        let mut results = HashMap::new();

        for name in names {
            let healthy = self
                .health_check_with_reconnect(&name)
                .await
                .unwrap_or(false);
            results.insert(name, healthy);
        }

        results
    }

    pub async fn find_session(&self, session_id: Uuid) -> Option<SessionHistory> {
        let adapters = self.adapters.read().await;

        for adapter in adapters.values() {
            let adapter_guard = adapter.read().await;
            if let Ok(Some(session)) = adapter_guard.get_session_details(session_id).await {
                return Some(session);
            }
        }

        None
    }
}

impl Default for AdapterRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for AdapterRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AdapterRegistry")
            .field("max_retry_attempts", &self.max_retry_attempts)
            .field("retry_delay_secs", &self.retry_delay_secs)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::super::traits::{AgentAdapter, CostTracker, SessionEventCallback, SessionMonitor};
    use super::*;
    use crate::models::ModelInfo;
    use async_trait::async_trait;
    use std::sync::Mutex;

    struct TestAdapter {
        name: String,
        agent_type: AgentType,
        status: Mutex<AdapterStatus>,
    }

    impl TestAdapter {
        fn new(name: &str, agent_type: AgentType) -> Self {
            Self {
                name: name.to_string(),
                agent_type,
                status: Mutex::new(AdapterStatus::Disconnected),
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

        async fn get_info(&self) -> RimuruResult<AdapterInfo> {
            Ok(AdapterInfo::new(self.name.clone(), self.agent_type))
        }

        async fn get_sessions(&self) -> RimuruResult<Vec<crate::models::Session>> {
            Ok(vec![])
        }

        async fn get_active_session(&self) -> RimuruResult<Option<ActiveSession>> {
            Ok(None)
        }

        async fn is_installed(&self) -> bool {
            true
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
    async fn test_register_adapter() {
        let registry = AdapterRegistry::new();
        let adapter = TestAdapter::new("claude-1", AgentType::ClaudeCode);

        registry
            .register("claude-1".to_string(), adapter)
            .await
            .unwrap();

        assert_eq!(registry.count().await, 1);
        assert!(registry.get("claude-1").await.is_some());
    }

    #[tokio::test]
    async fn test_register_duplicate_fails() {
        let registry = AdapterRegistry::new();

        registry
            .register(
                "claude-1".to_string(),
                TestAdapter::new("claude-1", AgentType::ClaudeCode),
            )
            .await
            .unwrap();

        let result = registry
            .register(
                "claude-1".to_string(),
                TestAdapter::new("claude-1", AgentType::ClaudeCode),
            )
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_unregister_adapter() {
        let registry = AdapterRegistry::new();

        registry
            .register(
                "claude-1".to_string(),
                TestAdapter::new("claude-1", AgentType::ClaudeCode),
            )
            .await
            .unwrap();

        registry.unregister("claude-1").await.unwrap();

        assert_eq!(registry.count().await, 0);
        assert!(registry.get("claude-1").await.is_none());
    }

    #[tokio::test]
    async fn test_get_by_type() {
        let registry = AdapterRegistry::new();

        registry
            .register(
                "claude-1".to_string(),
                TestAdapter::new("claude-1", AgentType::ClaudeCode),
            )
            .await
            .unwrap();
        registry
            .register(
                "claude-2".to_string(),
                TestAdapter::new("claude-2", AgentType::ClaudeCode),
            )
            .await
            .unwrap();
        registry
            .register(
                "opencode-1".to_string(),
                TestAdapter::new("opencode-1", AgentType::OpenCode),
            )
            .await
            .unwrap();

        let claude_adapters = registry.get_by_type(AgentType::ClaudeCode).await;
        assert_eq!(claude_adapters.len(), 2);

        let opencode_adapters = registry.get_by_type(AgentType::OpenCode).await;
        assert_eq!(opencode_adapters.len(), 1);
    }

    #[tokio::test]
    async fn test_connect_disconnect_all() {
        let registry = AdapterRegistry::new();

        registry
            .register(
                "adapter-1".to_string(),
                TestAdapter::new("adapter-1", AgentType::ClaudeCode),
            )
            .await
            .unwrap();
        registry
            .register(
                "adapter-2".to_string(),
                TestAdapter::new("adapter-2", AgentType::OpenCode),
            )
            .await
            .unwrap();

        let connect_results = registry.connect_all().await;
        assert_eq!(connect_results.len(), 2);
        assert!(connect_results.iter().all(|(_, r)| r.is_ok()));

        let statuses = registry.get_all_statuses().await;
        assert!(statuses.values().all(|s| *s == AdapterStatus::Connected));

        let disconnect_results = registry.disconnect_all().await;
        assert_eq!(disconnect_results.len(), 2);
        assert!(disconnect_results.iter().all(|(_, r)| r.is_ok()));
    }

    #[tokio::test]
    async fn test_aggregated_usage() {
        let registry = AdapterRegistry::new();

        registry
            .register(
                "adapter-1".to_string(),
                TestAdapter::new("adapter-1", AgentType::ClaudeCode),
            )
            .await
            .unwrap();
        registry
            .register(
                "adapter-2".to_string(),
                TestAdapter::new("adapter-2", AgentType::OpenCode),
            )
            .await
            .unwrap();

        let usage = registry.get_aggregated_usage(None).await.unwrap();
        assert_eq!(usage.input_tokens, 200);
        assert_eq!(usage.output_tokens, 100);
        assert_eq!(usage.requests, 10);
    }

    #[tokio::test]
    async fn test_aggregated_cost() {
        let registry = AdapterRegistry::new();

        registry
            .register(
                "adapter-1".to_string(),
                TestAdapter::new("adapter-1", AgentType::ClaudeCode),
            )
            .await
            .unwrap();
        registry
            .register(
                "adapter-2".to_string(),
                TestAdapter::new("adapter-2", AgentType::OpenCode),
            )
            .await
            .unwrap();

        let cost = registry.get_aggregated_cost(None).await.unwrap();
        assert!((cost - 0.02).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_health_check_all() {
        let registry = AdapterRegistry::new();

        registry
            .register(
                "adapter-1".to_string(),
                TestAdapter::new("adapter-1", AgentType::ClaudeCode),
            )
            .await
            .unwrap();

        let health_before = registry.health_check_all().await;
        assert!(!health_before.get("adapter-1").unwrap());

        registry.connect_all().await;

        let health_after = registry.health_check_all().await;
        assert!(health_after.get("adapter-1").unwrap());
    }
}
