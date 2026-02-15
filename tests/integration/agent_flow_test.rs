use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rimuru_core::{
    adapters::{ActiveSession, AdapterInfo, AdapterStatus, SessionHistory, UsageStats},
    db::DatabaseError,
    models::{Agent, AgentType, CostRecord, ModelInfo, Session, SessionStatus},
    repo::Repository,
    AgentAdapter, CostTracker, RimuruResult, SessionEventCallback, SessionMonitor,
};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use uuid::Uuid;

struct MockInMemoryDatabase {
    agents: Arc<RwLock<HashMap<Uuid, Agent>>>,
    sessions: Arc<RwLock<HashMap<Uuid, Session>>>,
    costs: Arc<RwLock<HashMap<Uuid, CostRecord>>>,
}

impl MockInMemoryDatabase {
    fn new() -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            costs: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

struct MockAgentRepo {
    db: Arc<MockInMemoryDatabase>,
}

impl MockAgentRepo {
    fn new(db: Arc<MockInMemoryDatabase>) -> Self {
        Self { db }
    }

    async fn create(&self, entity: &Agent) -> Result<Agent, DatabaseError> {
        let mut agents = self.db.agents.write().unwrap();
        agents.insert(entity.id, entity.clone());
        Ok(entity.clone())
    }

    async fn list(
        &self,
        _limit: Option<i64>,
        _offset: Option<i64>,
    ) -> Result<Vec<Agent>, DatabaseError> {
        let agents = self.db.agents.read().unwrap();
        Ok(agents.values().cloned().collect())
    }

    async fn count(&self) -> Result<i64, DatabaseError> {
        let agents = self.db.agents.read().unwrap();
        Ok(agents.len() as i64)
    }
}

#[async_trait]
impl Repository for MockAgentRepo {
    type Entity = Agent;
    type Id = Uuid;

    async fn get_by_id(&self, id: Self::Id) -> Result<Option<Self::Entity>, DatabaseError> {
        let agents = self.db.agents.read().unwrap();
        Ok(agents.get(&id).cloned())
    }

    async fn get_all(&self) -> Result<Vec<Self::Entity>, DatabaseError> {
        let agents = self.db.agents.read().unwrap();
        Ok(agents.values().cloned().collect())
    }

    async fn delete(&self, id: Self::Id) -> Result<bool, DatabaseError> {
        let mut agents = self.db.agents.write().unwrap();
        Ok(agents.remove(&id).is_some())
    }
}

struct MockSessionRepo {
    db: Arc<MockInMemoryDatabase>,
}

impl MockSessionRepo {
    fn new(db: Arc<MockInMemoryDatabase>) -> Self {
        Self { db }
    }

    async fn create(&self, entity: &Session) -> Result<Session, DatabaseError> {
        let mut sessions = self.db.sessions.write().unwrap();
        sessions.insert(entity.id, entity.clone());
        Ok(entity.clone())
    }

    async fn update(&self, entity: &Session) -> Result<Session, DatabaseError> {
        let mut sessions = self.db.sessions.write().unwrap();
        sessions.insert(entity.id, entity.clone());
        Ok(entity.clone())
    }

    async fn list(
        &self,
        limit: Option<i64>,
        _offset: Option<i64>,
    ) -> Result<Vec<Session>, DatabaseError> {
        let sessions = self.db.sessions.read().unwrap();
        let mut result: Vec<Session> = sessions.values().cloned().collect();
        if let Some(limit) = limit {
            result.truncate(limit as usize);
        }
        Ok(result)
    }

    async fn count(&self) -> Result<i64, DatabaseError> {
        let sessions = self.db.sessions.read().unwrap();
        Ok(sessions.len() as i64)
    }
}

#[async_trait]
impl Repository for MockSessionRepo {
    type Entity = Session;
    type Id = Uuid;

    async fn get_by_id(&self, id: Self::Id) -> Result<Option<Self::Entity>, DatabaseError> {
        let sessions = self.db.sessions.read().unwrap();
        Ok(sessions.get(&id).cloned())
    }

    async fn get_all(&self) -> Result<Vec<Self::Entity>, DatabaseError> {
        let sessions = self.db.sessions.read().unwrap();
        Ok(sessions.values().cloned().collect())
    }

    async fn delete(&self, id: Self::Id) -> Result<bool, DatabaseError> {
        let mut sessions = self.db.sessions.write().unwrap();
        Ok(sessions.remove(&id).is_some())
    }
}

struct MockCostRepo {
    db: Arc<MockInMemoryDatabase>,
}

impl MockCostRepo {
    fn new(db: Arc<MockInMemoryDatabase>) -> Self {
        Self { db }
    }

    async fn create(&self, entity: &CostRecord) -> Result<CostRecord, DatabaseError> {
        let mut costs = self.db.costs.write().unwrap();
        costs.insert(entity.id, entity.clone());
        Ok(entity.clone())
    }

    async fn list(
        &self,
        limit: Option<i64>,
        _offset: Option<i64>,
    ) -> Result<Vec<CostRecord>, DatabaseError> {
        let costs = self.db.costs.read().unwrap();
        let mut result: Vec<CostRecord> = costs.values().cloned().collect();
        if let Some(limit) = limit {
            result.truncate(limit as usize);
        }
        Ok(result)
    }
}

#[async_trait]
impl Repository for MockCostRepo {
    type Entity = CostRecord;
    type Id = Uuid;

    async fn get_by_id(&self, id: Self::Id) -> Result<Option<Self::Entity>, DatabaseError> {
        let costs = self.db.costs.read().unwrap();
        Ok(costs.get(&id).cloned())
    }

    async fn get_all(&self) -> Result<Vec<Self::Entity>, DatabaseError> {
        let costs = self.db.costs.read().unwrap();
        Ok(costs.values().cloned().collect())
    }

    async fn delete(&self, id: Self::Id) -> Result<bool, DatabaseError> {
        let mut costs = self.db.costs.write().unwrap();
        Ok(costs.remove(&id).is_some())
    }
}

struct IntegrationTestAdapter {
    name: String,
    agent_type: AgentType,
    status: Arc<RwLock<AdapterStatus>>,
    sessions: Arc<RwLock<Vec<Session>>>,
    usage: Arc<RwLock<UsageStats>>,
    subscriptions: Arc<RwLock<Vec<Uuid>>>,
}

impl IntegrationTestAdapter {
    fn new(name: &str, agent_type: AgentType) -> Self {
        Self {
            name: name.to_string(),
            agent_type,
            status: Arc::new(RwLock::new(AdapterStatus::Unknown)),
            sessions: Arc::new(RwLock::new(Vec::new())),
            usage: Arc::new(RwLock::new(UsageStats::new())),
            subscriptions: Arc::new(RwLock::new(Vec::new())),
        }
    }

    fn record_session(&self, session: Session) {
        self.sessions.write().unwrap().push(session);
    }

    fn record_usage(&self, input_tokens: i64, output_tokens: i64) {
        let mut usage = self.usage.write().unwrap();
        usage.input_tokens += input_tokens;
        usage.output_tokens += output_tokens;
        usage.requests += 1;
    }
}

#[async_trait]
impl AgentAdapter for IntegrationTestAdapter {
    fn agent_type(&self) -> AgentType {
        self.agent_type
    }

    fn name(&self) -> &str {
        &self.name
    }

    async fn connect(&mut self) -> RimuruResult<()> {
        *self.status.write().unwrap() = AdapterStatus::Connected;
        Ok(())
    }

    async fn disconnect(&mut self) -> RimuruResult<()> {
        *self.status.write().unwrap() = AdapterStatus::Disconnected;
        Ok(())
    }

    async fn get_status(&self) -> AdapterStatus {
        *self.status.read().unwrap()
    }

    async fn get_info(&self) -> RimuruResult<AdapterInfo> {
        Ok(AdapterInfo {
            name: self.name.clone(),
            agent_type: self.agent_type,
            version: Some("1.0.0".to_string()),
            status: *self.status.read().unwrap(),
            config_path: None,
            last_connected: None,
            error_message: None,
        })
    }

    async fn get_sessions(&self) -> RimuruResult<Vec<Session>> {
        Ok(self.sessions.read().unwrap().clone())
    }

    async fn get_active_session(&self) -> RimuruResult<Option<ActiveSession>> {
        let sessions = self.sessions.read().unwrap();
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
        Ok(*self.status.read().unwrap() == AdapterStatus::Connected)
    }
}

#[async_trait]
impl CostTracker for IntegrationTestAdapter {
    async fn get_usage(&self, _since: Option<DateTime<Utc>>) -> RimuruResult<UsageStats> {
        Ok(self.usage.read().unwrap().clone())
    }

    async fn calculate_cost(
        &self,
        input_tokens: i64,
        output_tokens: i64,
        model_name: &str,
    ) -> RimuruResult<f64> {
        let (input_rate, output_rate) = match model_name {
            "claude-3-opus" | "claude-opus-4" => (0.015, 0.075),
            "claude-3-sonnet" | "claude-sonnet-4" => (0.003, 0.015),
            "gpt-4" | "gpt-4-turbo" => (0.01, 0.03),
            _ => (0.001, 0.003),
        };
        let input_cost = (input_tokens as f64 / 1000.0) * input_rate;
        let output_cost = (output_tokens as f64 / 1000.0) * output_rate;
        Ok(input_cost + output_cost)
    }

    async fn get_model_info(&self, model_name: &str) -> RimuruResult<Option<ModelInfo>> {
        let model = match model_name {
            "claude-3-opus" | "claude-opus-4" => Some(ModelInfo::new(
                "anthropic".to_string(),
                model_name.to_string(),
                0.015,
                0.075,
                200000,
            )),
            "gpt-4" => Some(ModelInfo::new(
                "openai".to_string(),
                "gpt-4".to_string(),
                0.01,
                0.03,
                128000,
            )),
            _ => None,
        };
        Ok(model)
    }

    async fn get_supported_models(&self) -> RimuruResult<Vec<String>> {
        Ok(vec![
            "claude-opus-4".to_string(),
            "claude-sonnet-4".to_string(),
            "gpt-4-turbo".to_string(),
        ])
    }

    async fn get_total_cost(&self, _since: Option<DateTime<Utc>>) -> RimuruResult<f64> {
        let usage = self.usage.read().unwrap();
        Ok((usage.input_tokens as f64 / 1000.0) * 0.01
            + (usage.output_tokens as f64 / 1000.0) * 0.03)
    }
}

#[async_trait]
impl SessionMonitor for IntegrationTestAdapter {
    async fn subscribe_to_events(&self, _callback: SessionEventCallback) -> RimuruResult<Uuid> {
        let sub_id = Uuid::new_v4();
        self.subscriptions.write().unwrap().push(sub_id);
        Ok(sub_id)
    }

    async fn unsubscribe(&self, subscription_id: Uuid) -> RimuruResult<()> {
        self.subscriptions
            .write()
            .unwrap()
            .retain(|id| *id != subscription_id);
        Ok(())
    }

    async fn get_session_history(
        &self,
        limit: Option<usize>,
        _since: Option<DateTime<Utc>>,
    ) -> RimuruResult<Vec<SessionHistory>> {
        let sessions = self.sessions.read().unwrap();
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
                model_name: Some("claude-opus-4".to_string()),
                cost_usd: Some(0.05),
                project_path: None,
            })
            .collect();
        Ok(history)
    }

    async fn get_session_details(&self, session_id: Uuid) -> RimuruResult<Option<SessionHistory>> {
        let sessions = self.sessions.read().unwrap();
        Ok(sessions
            .iter()
            .find(|s| s.id == session_id)
            .map(|s| SessionHistory {
                session_id: s.id,
                agent_type: self.agent_type,
                started_at: s.started_at,
                ended_at: s.ended_at,
                total_input_tokens: 1000,
                total_output_tokens: 500,
                model_name: Some("claude-opus-4".to_string()),
                cost_usd: Some(0.05),
                project_path: None,
            }))
    }

    async fn get_active_sessions(&self) -> RimuruResult<Vec<ActiveSession>> {
        let sessions = self.sessions.read().unwrap();
        Ok(sessions
            .iter()
            .filter(|s| s.is_active())
            .map(|s| ActiveSession::new(s.id, self.agent_type))
            .collect())
    }
}

mod agent_discovery_flow {
    use super::*;

    #[tokio::test]
    async fn test_discover_and_register_agent() {
        let db = Arc::new(MockInMemoryDatabase::new());
        let agent_repo = MockAgentRepo::new(db.clone());

        let agent = Agent::new(
            "claude-code".to_string(),
            AgentType::ClaudeCode,
            serde_json::json!({}),
        );

        let created = agent_repo.create(&agent).await.unwrap();
        assert_eq!(created.id, agent.id);
        assert_eq!(created.agent_type, AgentType::ClaudeCode);

        let retrieved = agent_repo.get_by_id(agent.id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().agent_type, AgentType::ClaudeCode);
    }

    #[tokio::test]
    async fn test_discover_multiple_agents() {
        let db = Arc::new(MockInMemoryDatabase::new());
        let agent_repo = MockAgentRepo::new(db.clone());

        let agent_types = vec![
            AgentType::ClaudeCode,
            AgentType::Codex,
            AgentType::Copilot,
            AgentType::Cursor,
            AgentType::Goose,
            AgentType::OpenCode,
        ];

        for agent_type in &agent_types {
            let agent = Agent::new(
                format!("{:?}", agent_type),
                *agent_type,
                serde_json::json!({}),
            );
            agent_repo.create(&agent).await.unwrap();
        }

        let all_agents = agent_repo.list(None, None).await.unwrap();
        assert_eq!(all_agents.len(), 6);

        let count = agent_repo.count().await.unwrap();
        assert_eq!(count, 6);
    }

    #[tokio::test]
    async fn test_agent_adapter_connect_flow() {
        let mut adapter = IntegrationTestAdapter::new("test-claude", AgentType::ClaudeCode);

        assert_eq!(adapter.get_status().await, AdapterStatus::Unknown);
        assert!(!adapter.health_check().await.unwrap());

        adapter.connect().await.unwrap();
        assert_eq!(adapter.get_status().await, AdapterStatus::Connected);
        assert!(adapter.health_check().await.unwrap());

        let info = adapter.get_info().await.unwrap();
        assert_eq!(info.agent_type, AgentType::ClaudeCode);
        assert_eq!(info.status, AdapterStatus::Connected);
    }
}

mod session_tracking_flow {
    use super::*;

    #[tokio::test]
    async fn test_create_and_track_session() {
        let db = Arc::new(MockInMemoryDatabase::new());
        let session_repo = MockSessionRepo::new(db.clone());

        let agent_id = Uuid::new_v4();
        let session = Session::new(agent_id, serde_json::json!({"project": "/test/project"}));

        let created = session_repo.create(&session).await.unwrap();
        assert!(created.is_active());
        assert_eq!(created.agent_id, agent_id);

        let retrieved = session_repo.get_by_id(session.id).await.unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert!(retrieved.is_active());
    }

    #[tokio::test]
    async fn test_session_lifecycle() {
        let db = Arc::new(MockInMemoryDatabase::new());
        let session_repo = MockSessionRepo::new(db.clone());

        let agent_id = Uuid::new_v4();
        let mut session = Session::new(agent_id, serde_json::json!({}));

        session_repo.create(&session).await.unwrap();
        assert!(session.is_active());

        session.end(SessionStatus::Completed);
        session_repo.update(&session).await.unwrap();

        let updated = session_repo.get_by_id(session.id).await.unwrap().unwrap();
        assert!(!updated.is_active());
        assert!(updated.ended_at.is_some());
    }

    #[tokio::test]
    async fn test_adapter_session_tracking() {
        let mut adapter = IntegrationTestAdapter::new("session-test", AgentType::Codex);
        adapter.connect().await.unwrap();

        let session1 = Session::new(Uuid::new_v4(), serde_json::json!({}));
        let mut session2 = Session::new(Uuid::new_v4(), serde_json::json!({}));
        session2.end(SessionStatus::Completed);

        adapter.record_session(session1.clone());
        adapter.record_session(session2);

        let sessions = adapter.get_sessions().await.unwrap();
        assert_eq!(sessions.len(), 2);

        let active = adapter.get_active_session().await.unwrap();
        assert!(active.is_some());
        assert_eq!(active.unwrap().session_id, session1.id);

        let active_sessions = adapter.get_active_sessions().await.unwrap();
        assert_eq!(active_sessions.len(), 1);
    }

    #[tokio::test]
    async fn test_session_history_retrieval() {
        let adapter = IntegrationTestAdapter::new("history-test", AgentType::OpenCode);

        for _ in 0..5 {
            adapter.record_session(Session::new(Uuid::new_v4(), serde_json::json!({})));
        }

        let full_history = adapter.get_session_history(None, None).await.unwrap();
        assert_eq!(full_history.len(), 5);

        let limited_history = adapter.get_session_history(Some(3), None).await.unwrap();
        assert_eq!(limited_history.len(), 3);

        let session_id = full_history[0].session_id;
        let details = adapter.get_session_details(session_id).await.unwrap();
        assert!(details.is_some());
    }
}

mod cost_recording_flow {
    use super::*;

    #[tokio::test]
    async fn test_record_and_calculate_costs() {
        let db = Arc::new(MockInMemoryDatabase::new());
        let cost_repo = MockCostRepo::new(db.clone());

        let session_id = Uuid::new_v4();
        let agent_id = Uuid::new_v4();

        let cost_record = CostRecord::new(
            session_id,
            agent_id,
            "claude-opus-4".to_string(),
            10000,
            5000,
            0.525,
        );

        let created = cost_repo.create(&cost_record).await.unwrap();
        assert_eq!(created.input_tokens, 10000);
        assert_eq!(created.output_tokens, 5000);

        let retrieved = cost_repo.get_by_id(cost_record.id).await.unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.model_name, "claude-opus-4");
    }

    #[tokio::test]
    async fn test_adapter_cost_tracking() {
        let adapter = IntegrationTestAdapter::new("cost-test", AgentType::ClaudeCode);

        adapter.record_usage(10000, 5000);
        adapter.record_usage(5000, 2500);

        let usage = adapter.get_usage(None).await.unwrap();
        assert_eq!(usage.input_tokens, 15000);
        assert_eq!(usage.output_tokens, 7500);
        assert_eq!(usage.requests, 2);

        let cost = adapter
            .calculate_cost(10000, 5000, "claude-opus-4")
            .await
            .unwrap();
        let expected = (10000.0 / 1000.0) * 0.015 + (5000.0 / 1000.0) * 0.075;
        assert!((cost - expected).abs() < 0.0001);
    }

    #[tokio::test]
    async fn test_cost_across_different_models() {
        let adapter = IntegrationTestAdapter::new("multi-model", AgentType::Copilot);

        let opus_cost = adapter
            .calculate_cost(1000, 500, "claude-opus-4")
            .await
            .unwrap();
        let gpt4_cost = adapter.calculate_cost(1000, 500, "gpt-4").await.unwrap();
        let unknown_cost = adapter
            .calculate_cost(1000, 500, "unknown-model")
            .await
            .unwrap();

        assert!(opus_cost > gpt4_cost);
        assert!(gpt4_cost > unknown_cost);
    }

    #[tokio::test]
    async fn test_model_info_retrieval() {
        let adapter = IntegrationTestAdapter::new("model-info", AgentType::Cursor);

        let opus_info = adapter.get_model_info("claude-opus-4").await.unwrap();
        assert!(opus_info.is_some());
        let opus = opus_info.unwrap();
        assert_eq!(opus.provider, "anthropic");
        assert_eq!(opus.context_window, 200000);

        let gpt4_info = adapter.get_model_info("gpt-4").await.unwrap();
        assert!(gpt4_info.is_some());
        let gpt4 = gpt4_info.unwrap();
        assert_eq!(gpt4.provider, "openai");

        let unknown = adapter.get_model_info("unknown").await.unwrap();
        assert!(unknown.is_none());
    }
}

mod full_workflow_integration {
    use super::*;

    #[tokio::test]
    async fn test_complete_agent_session_cost_flow() {
        let db = Arc::new(MockInMemoryDatabase::new());
        let agent_repo = MockAgentRepo::new(db.clone());
        let session_repo = MockSessionRepo::new(db.clone());
        let cost_repo = MockCostRepo::new(db.clone());

        let agent = Agent::new(
            "claude-code".to_string(),
            AgentType::ClaudeCode,
            serde_json::json!({}),
        );
        let agent_id = agent.id;
        agent_repo.create(&agent).await.unwrap();

        let session = Session::new(agent_id, serde_json::json!({"project": "/my/project"}));
        let session_id = session.id;
        session_repo.create(&session).await.unwrap();

        let cost1 = CostRecord::new(
            session_id,
            agent_id,
            "claude-opus-4".to_string(),
            5000,
            2500,
            0.2625,
        );
        cost_repo.create(&cost1).await.unwrap();

        let cost2 = CostRecord::new(
            session_id,
            agent_id,
            "claude-opus-4".to_string(),
            3000,
            1500,
            0.1575,
        );
        cost_repo.create(&cost2).await.unwrap();

        let agents = agent_repo.list(None, None).await.unwrap();
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0].agent_type, AgentType::ClaudeCode);

        let sessions = session_repo.list(None, None).await.unwrap();
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].agent_id, agent_id);

        let costs = cost_repo.list(None, None).await.unwrap();
        assert_eq!(costs.len(), 2);

        let total_cost: f64 = costs.iter().map(|c| c.cost_usd).sum();
        assert!((total_cost - 0.42).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_multi_agent_session_workflow() {
        let db = Arc::new(MockInMemoryDatabase::new());
        let agent_repo = MockAgentRepo::new(db.clone());
        let session_repo = MockSessionRepo::new(db.clone());

        let claude_agent = Agent::new(
            "claude-code".to_string(),
            AgentType::ClaudeCode,
            serde_json::json!({}),
        );
        let codex_agent = Agent::new("codex".to_string(), AgentType::Codex, serde_json::json!({}));
        let cursor_agent = Agent::new(
            "cursor".to_string(),
            AgentType::Cursor,
            serde_json::json!({}),
        );

        agent_repo.create(&claude_agent).await.unwrap();
        agent_repo.create(&codex_agent).await.unwrap();
        agent_repo.create(&cursor_agent).await.unwrap();

        for agent in [&claude_agent, &codex_agent, &cursor_agent] {
            for i in 0..3 {
                let session = Session::new(
                    agent.id,
                    serde_json::json!({"project": format!("/project/{}", i)}),
                );
                session_repo.create(&session).await.unwrap();
            }
        }

        let agents = agent_repo.list(None, None).await.unwrap();
        assert_eq!(agents.len(), 3);

        let sessions = session_repo.list(None, None).await.unwrap();
        assert_eq!(sessions.len(), 9);

        let agent_count = agent_repo.count().await.unwrap();
        let session_count = session_repo.count().await.unwrap();
        assert_eq!(agent_count, 3);
        assert_eq!(session_count, 9);
    }

    #[tokio::test]
    async fn test_event_subscription_workflow() {
        let adapter = IntegrationTestAdapter::new("events-test", AgentType::Goose);

        let sub1 = adapter
            .subscribe_to_events(Box::new(|_event| {}))
            .await
            .unwrap();
        let sub2 = adapter
            .subscribe_to_events(Box::new(|_event| {}))
            .await
            .unwrap();

        assert_ne!(sub1, sub2);

        adapter.unsubscribe(sub1).await.unwrap();

        let subs = adapter.subscriptions.read().unwrap();
        assert!(!subs.contains(&sub1));
        assert!(subs.contains(&sub2));
    }
}

mod error_handling_flow {
    use super::*;

    #[tokio::test]
    async fn test_get_nonexistent_agent() {
        let db = Arc::new(MockInMemoryDatabase::new());
        let agent_repo = MockAgentRepo::new(db.clone());

        let result = agent_repo.get_by_id(Uuid::new_v4()).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_get_nonexistent_session() {
        let db = Arc::new(MockInMemoryDatabase::new());
        let session_repo = MockSessionRepo::new(db.clone());

        let result = session_repo.get_by_id(Uuid::new_v4()).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_delete_nonexistent_record() {
        let db = Arc::new(MockInMemoryDatabase::new());
        let agent_repo = MockAgentRepo::new(db.clone());

        let deleted = agent_repo.delete(Uuid::new_v4()).await.unwrap();
        assert!(!deleted);
    }

    #[tokio::test]
    async fn test_adapter_disconnected_state() {
        let mut adapter = IntegrationTestAdapter::new("disconnect-test", AgentType::ClaudeCode);

        adapter.connect().await.unwrap();
        assert!(adapter.health_check().await.unwrap());

        adapter.disconnect().await.unwrap();
        assert!(!adapter.health_check().await.unwrap());

        let info = adapter.get_info().await.unwrap();
        assert_eq!(info.status, AdapterStatus::Disconnected);
    }
}
