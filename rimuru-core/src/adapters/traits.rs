use async_trait::async_trait;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::error::RimuruResult;
use crate::models::{AgentType, ModelInfo, Session};

use super::types::{
    ActiveSession, AdapterInfo, AdapterStatus, SessionEvent, SessionHistory, UsageStats,
};

#[async_trait]
pub trait AgentAdapter: Send + Sync {
    fn agent_type(&self) -> AgentType;

    fn name(&self) -> &str;

    async fn connect(&mut self) -> RimuruResult<()>;

    async fn disconnect(&mut self) -> RimuruResult<()>;

    async fn get_status(&self) -> AdapterStatus;

    async fn get_info(&self) -> RimuruResult<AdapterInfo>;

    async fn get_sessions(&self) -> RimuruResult<Vec<Session>>;

    async fn get_active_session(&self) -> RimuruResult<Option<ActiveSession>>;

    async fn is_installed(&self) -> bool;

    async fn health_check(&self) -> RimuruResult<bool>;
}

#[async_trait]
pub trait CostTracker: Send + Sync {
    async fn get_usage(&self, since: Option<DateTime<Utc>>) -> RimuruResult<UsageStats>;

    async fn calculate_cost(
        &self,
        input_tokens: i64,
        output_tokens: i64,
        model_name: &str,
    ) -> RimuruResult<f64>;

    async fn get_model_info(&self, model_name: &str) -> RimuruResult<Option<ModelInfo>>;

    async fn get_supported_models(&self) -> RimuruResult<Vec<String>>;

    async fn get_total_cost(&self, since: Option<DateTime<Utc>>) -> RimuruResult<f64>;
}

pub type SessionEventCallback = Box<dyn Fn(SessionEvent) + Send + Sync>;

#[async_trait]
pub trait SessionMonitor: Send + Sync {
    async fn subscribe_to_events(&self, callback: SessionEventCallback) -> RimuruResult<Uuid>;

    async fn unsubscribe(&self, subscription_id: Uuid) -> RimuruResult<()>;

    async fn get_session_history(
        &self,
        limit: Option<usize>,
        since: Option<DateTime<Utc>>,
    ) -> RimuruResult<Vec<SessionHistory>>;

    async fn get_session_details(&self, session_id: Uuid) -> RimuruResult<Option<SessionHistory>>;

    async fn get_active_sessions(&self) -> RimuruResult<Vec<ActiveSession>>;
}

pub trait FullAdapter: AgentAdapter + CostTracker + SessionMonitor {}

impl<T> FullAdapter for T where T: AgentAdapter + CostTracker + SessionMonitor {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    struct MockAdapter {
        name: String,
        agent_type: AgentType,
        status: Arc<Mutex<AdapterStatus>>,
        connected: Arc<Mutex<bool>>,
    }

    impl MockAdapter {
        fn new(name: &str, agent_type: AgentType) -> Self {
            Self {
                name: name.to_string(),
                agent_type,
                status: Arc::new(Mutex::new(AdapterStatus::Unknown)),
                connected: Arc::new(Mutex::new(false)),
            }
        }
    }

    #[async_trait]
    impl AgentAdapter for MockAdapter {
        fn agent_type(&self) -> AgentType {
            self.agent_type
        }

        fn name(&self) -> &str {
            &self.name
        }

        async fn connect(&mut self) -> RimuruResult<()> {
            *self.connected.lock().unwrap() = true;
            *self.status.lock().unwrap() = AdapterStatus::Connected;
            Ok(())
        }

        async fn disconnect(&mut self) -> RimuruResult<()> {
            *self.connected.lock().unwrap() = false;
            *self.status.lock().unwrap() = AdapterStatus::Disconnected;
            Ok(())
        }

        async fn get_status(&self) -> AdapterStatus {
            *self.status.lock().unwrap()
        }

        async fn get_info(&self) -> RimuruResult<AdapterInfo> {
            Ok(AdapterInfo {
                name: self.name.clone(),
                agent_type: self.agent_type,
                version: Some("1.0.0".to_string()),
                status: *self.status.lock().unwrap(),
                config_path: None,
                last_connected: None,
                error_message: None,
            })
        }

        async fn get_sessions(&self) -> RimuruResult<Vec<Session>> {
            Ok(vec![])
        }

        async fn get_active_session(&self) -> RimuruResult<Option<ActiveSession>> {
            Ok(None)
        }

        async fn is_installed(&self) -> bool {
            true
        }

        async fn health_check(&self) -> RimuruResult<bool> {
            Ok(*self.connected.lock().unwrap())
        }
    }

    #[async_trait]
    impl CostTracker for MockAdapter {
        async fn get_usage(&self, _since: Option<DateTime<Utc>>) -> RimuruResult<UsageStats> {
            Ok(UsageStats {
                input_tokens: 1000,
                output_tokens: 500,
                requests: 10,
                model_name: Some("test-model".to_string()),
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
            let input_cost = input_tokens as f64 * 0.00001;
            let output_cost = output_tokens as f64 * 0.00003;
            Ok(input_cost + output_cost)
        }

        async fn get_model_info(&self, model_name: &str) -> RimuruResult<Option<ModelInfo>> {
            Ok(Some(ModelInfo::new(
                "test-provider".to_string(),
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
            Ok(0.05)
        }
    }

    #[async_trait]
    impl SessionMonitor for MockAdapter {
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
    async fn test_mock_adapter_connect_disconnect() {
        let mut adapter = MockAdapter::new("test", AgentType::ClaudeCode);

        assert_eq!(adapter.get_status().await, AdapterStatus::Unknown);

        adapter.connect().await.unwrap();
        assert_eq!(adapter.get_status().await, AdapterStatus::Connected);

        adapter.disconnect().await.unwrap();
        assert_eq!(adapter.get_status().await, AdapterStatus::Disconnected);
    }

    #[tokio::test]
    async fn test_mock_adapter_info() {
        let adapter = MockAdapter::new("test-claude", AgentType::ClaudeCode);
        let info = adapter.get_info().await.unwrap();

        assert_eq!(info.name, "test-claude");
        assert_eq!(info.agent_type, AgentType::ClaudeCode);
        assert_eq!(info.version, Some("1.0.0".to_string()));
    }

    #[tokio::test]
    async fn test_mock_adapter_cost_tracking() {
        let adapter = MockAdapter::new("test", AgentType::ClaudeCode);

        let usage = adapter.get_usage(None).await.unwrap();
        assert_eq!(usage.input_tokens, 1000);
        assert_eq!(usage.output_tokens, 500);

        let cost = adapter
            .calculate_cost(1000, 500, "test-model")
            .await
            .unwrap();
        assert!(cost > 0.0);

        let total_cost = adapter.get_total_cost(None).await.unwrap();
        assert_eq!(total_cost, 0.05);
    }

    #[tokio::test]
    async fn test_full_adapter_trait() {
        let adapter = MockAdapter::new("full-test", AgentType::OpenCode);

        fn assert_full_adapter<T: FullAdapter>(_: &T) {}
        assert_full_adapter(&adapter);

        assert_eq!(adapter.agent_type(), AgentType::OpenCode);
        let _ = adapter.get_usage(None).await;
        let _ = adapter.get_session_history(None, None).await;
    }
}
