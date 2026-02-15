use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rimuru_core::adapters::{
    ActiveSession, AdapterInfo, AdapterStatus, SessionEvent, SessionHistory, UsageStats,
};
use rimuru_core::error::RimuruResult;
use rimuru_core::models::{AgentType, ModelInfo, Session};
use rimuru_core::{AgentAdapter, CostTracker, FullAdapter, SessionEventCallback, SessionMonitor};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

struct TestAdapter {
    name: String,
    agent_type: AgentType,
    status: Arc<Mutex<AdapterStatus>>,
    connected: Arc<Mutex<bool>>,
    sessions: Arc<Mutex<Vec<Session>>>,
    usage: Arc<Mutex<UsageStats>>,
    event_subscriptions: Arc<Mutex<Vec<Uuid>>>,
}

impl TestAdapter {
    fn new(name: &str, agent_type: AgentType) -> Self {
        Self {
            name: name.to_string(),
            agent_type,
            status: Arc::new(Mutex::new(AdapterStatus::Unknown)),
            connected: Arc::new(Mutex::new(false)),
            sessions: Arc::new(Mutex::new(Vec::new())),
            usage: Arc::new(Mutex::new(UsageStats::new())),
            event_subscriptions: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn add_session(&self, session: Session) {
        self.sessions.lock().unwrap().push(session);
    }

    fn set_usage(&self, input_tokens: i64, output_tokens: i64, requests: i64) {
        let mut usage = self.usage.lock().unwrap();
        usage.input_tokens = input_tokens;
        usage.output_tokens = output_tokens;
        usage.requests = requests;
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
            config_path: Some("/home/user/.config/test".to_string()),
            last_connected: None,
            error_message: None,
        })
    }

    async fn get_sessions(&self) -> RimuruResult<Vec<Session>> {
        Ok(self.sessions.lock().unwrap().clone())
    }

    async fn get_active_session(&self) -> RimuruResult<Option<ActiveSession>> {
        let sessions = self.sessions.lock().unwrap();
        let active = sessions
            .iter()
            .find(|s| s.is_active())
            .map(|s| ActiveSession::new(s.id, self.agent_type));
        Ok(active)
    }

    async fn is_installed(&self) -> bool {
        true
    }

    async fn health_check(&self) -> RimuruResult<bool> {
        Ok(*self.connected.lock().unwrap())
    }
}

#[async_trait]
impl CostTracker for TestAdapter {
    async fn get_usage(&self, _since: Option<DateTime<Utc>>) -> RimuruResult<UsageStats> {
        Ok(self.usage.lock().unwrap().clone())
    }

    async fn calculate_cost(
        &self,
        input_tokens: i64,
        output_tokens: i64,
        model_name: &str,
    ) -> RimuruResult<f64> {
        let (input_rate, output_rate) = match model_name {
            "claude-3-opus" => (0.015, 0.075),
            "claude-3-sonnet" => (0.003, 0.015),
            "gpt-4" => (0.03, 0.06),
            _ => (0.001, 0.002),
        };
        let input_cost = (input_tokens as f64 / 1000.0) * input_rate;
        let output_cost = (output_tokens as f64 / 1000.0) * output_rate;
        Ok(input_cost + output_cost)
    }

    async fn get_model_info(&self, model_name: &str) -> RimuruResult<Option<ModelInfo>> {
        let model = match model_name {
            "claude-3-opus" => Some(ModelInfo::new(
                "anthropic".to_string(),
                "claude-3-opus".to_string(),
                0.015,
                0.075,
                200000,
            )),
            "gpt-4" => Some(ModelInfo::new(
                "openai".to_string(),
                "gpt-4".to_string(),
                0.03,
                0.06,
                128000,
            )),
            _ => None,
        };
        Ok(model)
    }

    async fn get_supported_models(&self) -> RimuruResult<Vec<String>> {
        Ok(vec![
            "claude-3-opus".to_string(),
            "claude-3-sonnet".to_string(),
            "claude-3-haiku".to_string(),
        ])
    }

    async fn get_total_cost(&self, _since: Option<DateTime<Utc>>) -> RimuruResult<f64> {
        let usage = self.usage.lock().unwrap();
        let cost = (usage.input_tokens as f64 / 1000.0) * 0.01
            + (usage.output_tokens as f64 / 1000.0) * 0.03;
        Ok(cost)
    }
}

#[async_trait]
impl SessionMonitor for TestAdapter {
    async fn subscribe_to_events(&self, _callback: SessionEventCallback) -> RimuruResult<Uuid> {
        let sub_id = Uuid::new_v4();
        self.event_subscriptions.lock().unwrap().push(sub_id);
        Ok(sub_id)
    }

    async fn unsubscribe(&self, subscription_id: Uuid) -> RimuruResult<()> {
        let mut subs = self.event_subscriptions.lock().unwrap();
        subs.retain(|id| *id != subscription_id);
        Ok(())
    }

    async fn get_session_history(
        &self,
        limit: Option<usize>,
        _since: Option<DateTime<Utc>>,
    ) -> RimuruResult<Vec<SessionHistory>> {
        let sessions = self.sessions.lock().unwrap();
        let history: Vec<SessionHistory> = sessions
            .iter()
            .take(limit.unwrap_or(usize::MAX))
            .map(|s| SessionHistory {
                session_id: s.id,
                agent_type: self.agent_type,
                started_at: s.started_at,
                ended_at: s.ended_at,
                total_input_tokens: 1000,
                total_output_tokens: 500,
                model_name: Some("claude-3-opus".to_string()),
                cost_usd: Some(0.05),
                project_path: None,
            })
            .collect();
        Ok(history)
    }

    async fn get_session_details(&self, session_id: Uuid) -> RimuruResult<Option<SessionHistory>> {
        let sessions = self.sessions.lock().unwrap();
        let session = sessions.iter().find(|s| s.id == session_id);
        Ok(session.map(|s| SessionHistory {
            session_id: s.id,
            agent_type: self.agent_type,
            started_at: s.started_at,
            ended_at: s.ended_at,
            total_input_tokens: 1000,
            total_output_tokens: 500,
            model_name: Some("claude-3-opus".to_string()),
            cost_usd: Some(0.05),
            project_path: None,
        }))
    }

    async fn get_active_sessions(&self) -> RimuruResult<Vec<ActiveSession>> {
        let sessions = self.sessions.lock().unwrap();
        let active: Vec<ActiveSession> = sessions
            .iter()
            .filter(|s| s.is_active())
            .map(|s| ActiveSession::new(s.id, self.agent_type))
            .collect();
        Ok(active)
    }
}

mod agent_adapter_tests {
    use super::*;

    #[tokio::test]
    async fn test_adapter_connect_disconnect() {
        let mut adapter = TestAdapter::new("test-claude", AgentType::ClaudeCode);

        assert_eq!(adapter.get_status().await, AdapterStatus::Unknown);
        assert!(!adapter.health_check().await.unwrap());

        adapter.connect().await.unwrap();
        assert_eq!(adapter.get_status().await, AdapterStatus::Connected);
        assert!(adapter.health_check().await.unwrap());

        adapter.disconnect().await.unwrap();
        assert_eq!(adapter.get_status().await, AdapterStatus::Disconnected);
        assert!(!adapter.health_check().await.unwrap());
    }

    #[tokio::test]
    async fn test_adapter_info() {
        let adapter = TestAdapter::new("my-adapter", AgentType::OpenCode);
        let info = adapter.get_info().await.unwrap();

        assert_eq!(info.name, "my-adapter");
        assert_eq!(info.agent_type, AgentType::OpenCode);
        assert_eq!(info.version, Some("1.0.0".to_string()));
        assert!(info.config_path.is_some());
    }

    #[tokio::test]
    async fn test_adapter_sessions() {
        let adapter = TestAdapter::new("test", AgentType::Codex);

        let session1 = Session::new(Uuid::new_v4(), serde_json::json!({}));
        let mut session2 = Session::new(Uuid::new_v4(), serde_json::json!({}));
        session2.end(rimuru_core::models::SessionStatus::Completed);

        adapter.add_session(session1.clone());
        adapter.add_session(session2);

        let sessions = adapter.get_sessions().await.unwrap();
        assert_eq!(sessions.len(), 2);

        let active = adapter.get_active_session().await.unwrap();
        assert!(active.is_some());
        assert_eq!(active.unwrap().session_id, session1.id);
    }

    #[tokio::test]
    async fn test_adapter_is_installed() {
        let adapter = TestAdapter::new("test", AgentType::Goose);
        assert!(adapter.is_installed().await);
    }

    #[test]
    fn test_adapter_type_and_name() {
        let adapter = TestAdapter::new("cursor-adapter", AgentType::Cursor);
        assert_eq!(adapter.agent_type(), AgentType::Cursor);
        assert_eq!(adapter.name(), "cursor-adapter");
    }
}

mod cost_tracker_tests {
    use super::*;

    #[tokio::test]
    async fn test_get_usage() {
        let adapter = TestAdapter::new("test", AgentType::ClaudeCode);
        adapter.set_usage(10000, 5000, 50);

        let usage = adapter.get_usage(None).await.unwrap();
        assert_eq!(usage.input_tokens, 10000);
        assert_eq!(usage.output_tokens, 5000);
        assert_eq!(usage.requests, 50);
        assert_eq!(usage.total_tokens(), 15000);
    }

    #[tokio::test]
    async fn test_calculate_cost_claude() {
        let adapter = TestAdapter::new("test", AgentType::ClaudeCode);

        let cost = adapter
            .calculate_cost(10000, 5000, "claude-3-opus")
            .await
            .unwrap();
        let expected = (10000.0 / 1000.0) * 0.015 + (5000.0 / 1000.0) * 0.075;
        assert!((cost - expected).abs() < 0.0001);
    }

    #[tokio::test]
    async fn test_calculate_cost_gpt4() {
        let adapter = TestAdapter::new("test", AgentType::ClaudeCode);

        let cost = adapter.calculate_cost(5000, 2500, "gpt-4").await.unwrap();
        let expected = (5000.0 / 1000.0) * 0.03 + (2500.0 / 1000.0) * 0.06;
        assert!((cost - expected).abs() < 0.0001);
    }

    #[tokio::test]
    async fn test_calculate_cost_unknown_model() {
        let adapter = TestAdapter::new("test", AgentType::ClaudeCode);

        let cost = adapter
            .calculate_cost(1000, 500, "unknown-model")
            .await
            .unwrap();
        let expected = (1000.0 / 1000.0) * 0.001 + (500.0 / 1000.0) * 0.002;
        assert!((cost - expected).abs() < 0.0001);
    }

    #[tokio::test]
    async fn test_get_model_info() {
        let adapter = TestAdapter::new("test", AgentType::ClaudeCode);

        let opus = adapter.get_model_info("claude-3-opus").await.unwrap();
        assert!(opus.is_some());
        let opus = opus.unwrap();
        assert_eq!(opus.provider, "anthropic");
        assert_eq!(opus.context_window, 200000);

        let gpt4 = adapter.get_model_info("gpt-4").await.unwrap();
        assert!(gpt4.is_some());
        let gpt4 = gpt4.unwrap();
        assert_eq!(gpt4.provider, "openai");
        assert_eq!(gpt4.context_window, 128000);

        let unknown = adapter.get_model_info("unknown").await.unwrap();
        assert!(unknown.is_none());
    }

    #[tokio::test]
    async fn test_get_supported_models() {
        let adapter = TestAdapter::new("test", AgentType::ClaudeCode);

        let models = adapter.get_supported_models().await.unwrap();
        assert_eq!(models.len(), 3);
        assert!(models.contains(&"claude-3-opus".to_string()));
        assert!(models.contains(&"claude-3-sonnet".to_string()));
        assert!(models.contains(&"claude-3-haiku".to_string()));
    }

    #[tokio::test]
    async fn test_get_total_cost() {
        let adapter = TestAdapter::new("test", AgentType::ClaudeCode);
        adapter.set_usage(10000, 5000, 50);

        let total = adapter.get_total_cost(None).await.unwrap();
        let expected = (10000.0 / 1000.0) * 0.01 + (5000.0 / 1000.0) * 0.03;
        assert!((total - expected).abs() < 0.0001);
    }
}

mod session_monitor_tests {
    use super::*;

    #[tokio::test]
    async fn test_subscribe_unsubscribe() {
        let adapter = TestAdapter::new("test", AgentType::ClaudeCode);

        let sub_id = adapter
            .subscribe_to_events(Box::new(|_event| {}))
            .await
            .unwrap();
        assert!(adapter
            .event_subscriptions
            .lock()
            .unwrap()
            .contains(&sub_id));

        adapter.unsubscribe(sub_id).await.unwrap();
        assert!(!adapter
            .event_subscriptions
            .lock()
            .unwrap()
            .contains(&sub_id));
    }

    #[tokio::test]
    async fn test_get_session_history() {
        let adapter = TestAdapter::new("test", AgentType::ClaudeCode);

        adapter.add_session(Session::new(Uuid::new_v4(), serde_json::json!({})));
        adapter.add_session(Session::new(Uuid::new_v4(), serde_json::json!({})));
        adapter.add_session(Session::new(Uuid::new_v4(), serde_json::json!({})));

        let history = adapter.get_session_history(None, None).await.unwrap();
        assert_eq!(history.len(), 3);

        let limited = adapter.get_session_history(Some(2), None).await.unwrap();
        assert_eq!(limited.len(), 2);
    }

    #[tokio::test]
    async fn test_get_session_details() {
        let adapter = TestAdapter::new("test", AgentType::ClaudeCode);
        let session = Session::new(Uuid::new_v4(), serde_json::json!({}));
        let session_id = session.id;
        adapter.add_session(session);

        let details = adapter.get_session_details(session_id).await.unwrap();
        assert!(details.is_some());
        let details = details.unwrap();
        assert_eq!(details.session_id, session_id);
        assert_eq!(details.total_input_tokens, 1000);
        assert_eq!(details.total_output_tokens, 500);

        let not_found = adapter.get_session_details(Uuid::new_v4()).await.unwrap();
        assert!(not_found.is_none());
    }

    #[tokio::test]
    async fn test_get_active_sessions() {
        let adapter = TestAdapter::new("test", AgentType::ClaudeCode);

        let session1 = Session::new(Uuid::new_v4(), serde_json::json!({}));
        let mut session2 = Session::new(Uuid::new_v4(), serde_json::json!({}));
        session2.end(rimuru_core::models::SessionStatus::Completed);
        let session3 = Session::new(Uuid::new_v4(), serde_json::json!({}));

        adapter.add_session(session1.clone());
        adapter.add_session(session2);
        adapter.add_session(session3.clone());

        let active = adapter.get_active_sessions().await.unwrap();
        assert_eq!(active.len(), 2);

        let active_ids: Vec<Uuid> = active.iter().map(|a| a.session_id).collect();
        assert!(active_ids.contains(&session1.id));
        assert!(active_ids.contains(&session3.id));
    }
}

mod full_adapter_tests {
    use super::*;

    #[tokio::test]
    async fn test_full_adapter_trait() {
        let adapter = TestAdapter::new("full-test", AgentType::ClaudeCode);

        fn assert_full_adapter<T: FullAdapter>(_: &T) {}
        assert_full_adapter(&adapter);
    }

    #[tokio::test]
    async fn test_full_adapter_all_capabilities() {
        let mut adapter = TestAdapter::new("complete", AgentType::ClaudeCode);

        adapter.connect().await.unwrap();
        assert_eq!(adapter.get_status().await, AdapterStatus::Connected);

        adapter.set_usage(5000, 2500, 25);
        let usage = adapter.get_usage(None).await.unwrap();
        assert_eq!(usage.total_tokens(), 7500);

        let session = Session::new(Uuid::new_v4(), serde_json::json!({}));
        adapter.add_session(session.clone());

        let active = adapter.get_active_sessions().await.unwrap();
        assert_eq!(active.len(), 1);

        let sub_id = adapter.subscribe_to_events(Box::new(|_| {})).await.unwrap();
        adapter.unsubscribe(sub_id).await.unwrap();

        adapter.disconnect().await.unwrap();
        assert_eq!(adapter.get_status().await, AdapterStatus::Disconnected);
    }
}

mod adapter_types_tests {
    use super::*;

    #[test]
    fn test_adapter_status_display() {
        assert_eq!(AdapterStatus::Connected.to_string(), "connected");
        assert_eq!(AdapterStatus::Disconnected.to_string(), "disconnected");
        assert_eq!(AdapterStatus::Error.to_string(), "error");
        assert_eq!(AdapterStatus::Unknown.to_string(), "unknown");
    }

    #[test]
    fn test_adapter_status_default() {
        assert_eq!(AdapterStatus::default(), AdapterStatus::Unknown);
    }

    #[test]
    fn test_usage_stats_add() {
        let mut stats1 = UsageStats {
            input_tokens: 1000,
            output_tokens: 500,
            requests: 10,
            ..Default::default()
        };

        let stats2 = UsageStats {
            input_tokens: 2000,
            output_tokens: 1000,
            requests: 20,
            ..Default::default()
        };

        stats1.add(&stats2);

        assert_eq!(stats1.input_tokens, 3000);
        assert_eq!(stats1.output_tokens, 1500);
        assert_eq!(stats1.requests, 30);
    }

    #[test]
    fn test_session_event_accessors() {
        let session_id = Uuid::new_v4();
        let timestamp = Utc::now();

        let started = SessionEvent::Started {
            session_id,
            timestamp,
            metadata: serde_json::json!({}),
        };
        assert_eq!(started.session_id(), session_id);
        assert_eq!(started.timestamp(), timestamp);

        let msg_sent = SessionEvent::MessageSent {
            session_id,
            timestamp,
            tokens: 100,
        };
        assert_eq!(msg_sent.session_id(), session_id);

        let msg_recv = SessionEvent::MessageReceived {
            session_id,
            timestamp,
            tokens: 200,
        };
        assert_eq!(msg_recv.session_id(), session_id);

        let completed = SessionEvent::Completed {
            session_id,
            timestamp,
            total_tokens: 300,
        };
        assert_eq!(completed.session_id(), session_id);

        let error = SessionEvent::Error {
            session_id,
            timestamp,
            error: "test error".to_string(),
        };
        assert_eq!(error.session_id(), session_id);
    }

    #[test]
    fn test_adapter_info_new() {
        let info = AdapterInfo::new("test-adapter".to_string(), AgentType::Copilot);

        assert_eq!(info.name, "test-adapter");
        assert_eq!(info.agent_type, AgentType::Copilot);
        assert_eq!(info.status, AdapterStatus::Unknown);
        assert!(info.version.is_none());
        assert!(info.config_path.is_none());
    }

    #[test]
    fn test_active_session_duration() {
        let session = ActiveSession::new(Uuid::new_v4(), AgentType::ClaudeCode);
        assert!(session.duration_seconds() >= 0);
    }

    #[test]
    fn test_session_history_total_tokens() {
        let history = SessionHistory {
            session_id: Uuid::new_v4(),
            agent_type: AgentType::ClaudeCode,
            started_at: Utc::now(),
            ended_at: Some(Utc::now()),
            total_input_tokens: 5000,
            total_output_tokens: 2500,
            model_name: Some("claude-3-opus".to_string()),
            cost_usd: Some(0.50),
            project_path: None,
        };

        assert_eq!(history.total_tokens(), 7500);
        assert!(history.duration_seconds().is_some());
    }
}
