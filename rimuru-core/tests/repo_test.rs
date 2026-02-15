#![allow(
    dead_code,
    unused_imports,
    unused_variables,
    unused_mut,
    clippy::map_entry
)]

use rimuru_core::models::{
    Agent, AgentType, CostRecord, ModelInfo, Session, SessionStatus, SystemMetrics,
};
use rimuru_core::repo::Repository;
use uuid::Uuid;

mod mock_repository {
    use super::*;
    use async_trait::async_trait;
    use rimuru_core::db::DatabaseError;
    use std::collections::HashMap;
    use std::sync::{Arc, RwLock};

    pub struct MockAgentRepository {
        agents: Arc<RwLock<HashMap<Uuid, Agent>>>,
    }

    impl MockAgentRepository {
        pub fn new() -> Self {
            Self {
                agents: Arc::new(RwLock::new(HashMap::new())),
            }
        }

        pub fn create(&self, agent: &Agent) -> Result<Agent, DatabaseError> {
            let mut agents = self.agents.write().unwrap();
            agents.insert(agent.id, agent.clone());
            Ok(agent.clone())
        }

        pub fn update(&self, agent: &Agent) -> Result<Option<Agent>, DatabaseError> {
            let mut agents = self.agents.write().unwrap();
            if agents.contains_key(&agent.id) {
                agents.insert(agent.id, agent.clone());
                Ok(Some(agent.clone()))
            } else {
                Ok(None)
            }
        }

        pub fn get_by_type(&self, agent_type: AgentType) -> Result<Vec<Agent>, DatabaseError> {
            let agents = self.agents.read().unwrap();
            let filtered: Vec<Agent> = agents
                .values()
                .filter(|a| a.agent_type == agent_type)
                .cloned()
                .collect();
            Ok(filtered)
        }

        pub fn get_by_name(&self, name: &str) -> Result<Option<Agent>, DatabaseError> {
            let agents = self.agents.read().unwrap();
            let found = agents.values().find(|a| a.name == name).cloned();
            Ok(found)
        }

        pub fn count(&self) -> Result<i64, DatabaseError> {
            let agents = self.agents.read().unwrap();
            Ok(agents.len() as i64)
        }
    }

    #[async_trait]
    impl Repository for MockAgentRepository {
        type Entity = Agent;
        type Id = Uuid;

        async fn get_by_id(&self, id: Uuid) -> Result<Option<Agent>, DatabaseError> {
            let agents = self.agents.read().unwrap();
            Ok(agents.get(&id).cloned())
        }

        async fn get_all(&self) -> Result<Vec<Agent>, DatabaseError> {
            let agents = self.agents.read().unwrap();
            Ok(agents.values().cloned().collect())
        }

        async fn delete(&self, id: Uuid) -> Result<bool, DatabaseError> {
            let mut agents = self.agents.write().unwrap();
            Ok(agents.remove(&id).is_some())
        }
    }

    pub struct MockSessionRepository {
        sessions: Arc<RwLock<HashMap<Uuid, Session>>>,
    }

    impl MockSessionRepository {
        pub fn new() -> Self {
            Self {
                sessions: Arc::new(RwLock::new(HashMap::new())),
            }
        }

        pub fn create(&self, session: &Session) -> Result<Session, DatabaseError> {
            let mut sessions = self.sessions.write().unwrap();
            sessions.insert(session.id, session.clone());
            Ok(session.clone())
        }

        pub fn update(&self, session: &Session) -> Result<Option<Session>, DatabaseError> {
            let mut sessions = self.sessions.write().unwrap();
            if sessions.contains_key(&session.id) {
                sessions.insert(session.id, session.clone());
                Ok(Some(session.clone()))
            } else {
                Ok(None)
            }
        }

        pub fn get_by_agent_id(&self, agent_id: Uuid) -> Result<Vec<Session>, DatabaseError> {
            let sessions = self.sessions.read().unwrap();
            let filtered: Vec<Session> = sessions
                .values()
                .filter(|s| s.agent_id == agent_id)
                .cloned()
                .collect();
            Ok(filtered)
        }

        pub fn get_active(&self) -> Result<Vec<Session>, DatabaseError> {
            let sessions = self.sessions.read().unwrap();
            let active: Vec<Session> = sessions
                .values()
                .filter(|s| s.status == SessionStatus::Active)
                .cloned()
                .collect();
            Ok(active)
        }
    }

    #[async_trait]
    impl Repository for MockSessionRepository {
        type Entity = Session;
        type Id = Uuid;

        async fn get_by_id(&self, id: Uuid) -> Result<Option<Session>, DatabaseError> {
            let sessions = self.sessions.read().unwrap();
            Ok(sessions.get(&id).cloned())
        }

        async fn get_all(&self) -> Result<Vec<Session>, DatabaseError> {
            let sessions = self.sessions.read().unwrap();
            Ok(sessions.values().cloned().collect())
        }

        async fn delete(&self, id: Uuid) -> Result<bool, DatabaseError> {
            let mut sessions = self.sessions.write().unwrap();
            Ok(sessions.remove(&id).is_some())
        }
    }

    pub struct MockCostRepository {
        costs: Arc<RwLock<HashMap<Uuid, CostRecord>>>,
    }

    impl MockCostRepository {
        pub fn new() -> Self {
            Self {
                costs: Arc::new(RwLock::new(HashMap::new())),
            }
        }

        pub fn create(&self, cost: &CostRecord) -> Result<CostRecord, DatabaseError> {
            let mut costs = self.costs.write().unwrap();
            costs.insert(cost.id, cost.clone());
            Ok(cost.clone())
        }

        pub fn get_by_session_id(
            &self,
            session_id: Uuid,
        ) -> Result<Vec<CostRecord>, DatabaseError> {
            let costs = self.costs.read().unwrap();
            let filtered: Vec<CostRecord> = costs
                .values()
                .filter(|c| c.session_id == session_id)
                .cloned()
                .collect();
            Ok(filtered)
        }

        pub fn get_by_agent_id(&self, agent_id: Uuid) -> Result<Vec<CostRecord>, DatabaseError> {
            let costs = self.costs.read().unwrap();
            let filtered: Vec<CostRecord> = costs
                .values()
                .filter(|c| c.agent_id == agent_id)
                .cloned()
                .collect();
            Ok(filtered)
        }

        pub fn get_total_cost(&self) -> f64 {
            let costs = self.costs.read().unwrap();
            costs.values().map(|c| c.cost_usd).sum()
        }
    }

    #[async_trait]
    impl Repository for MockCostRepository {
        type Entity = CostRecord;
        type Id = Uuid;

        async fn get_by_id(&self, id: Uuid) -> Result<Option<CostRecord>, DatabaseError> {
            let costs = self.costs.read().unwrap();
            Ok(costs.get(&id).cloned())
        }

        async fn get_all(&self) -> Result<Vec<CostRecord>, DatabaseError> {
            let costs = self.costs.read().unwrap();
            Ok(costs.values().cloned().collect())
        }

        async fn delete(&self, id: Uuid) -> Result<bool, DatabaseError> {
            let mut costs = self.costs.write().unwrap();
            Ok(costs.remove(&id).is_some())
        }
    }

    pub struct MockModelRepository {
        models: Arc<RwLock<HashMap<Uuid, ModelInfo>>>,
    }

    impl MockModelRepository {
        pub fn new() -> Self {
            Self {
                models: Arc::new(RwLock::new(HashMap::new())),
            }
        }

        pub fn create(&self, model: &ModelInfo) -> Result<ModelInfo, DatabaseError> {
            let mut models = self.models.write().unwrap();
            models.insert(model.id, model.clone());
            Ok(model.clone())
        }

        pub fn get_by_provider(&self, provider: &str) -> Result<Vec<ModelInfo>, DatabaseError> {
            let models = self.models.read().unwrap();
            let filtered: Vec<ModelInfo> = models
                .values()
                .filter(|m| m.provider == provider)
                .cloned()
                .collect();
            Ok(filtered)
        }

        pub fn get_by_name(&self, model_name: &str) -> Result<Option<ModelInfo>, DatabaseError> {
            let models = self.models.read().unwrap();
            let found = models
                .values()
                .find(|m| m.model_name == model_name)
                .cloned();
            Ok(found)
        }
    }

    #[async_trait]
    impl Repository for MockModelRepository {
        type Entity = ModelInfo;
        type Id = Uuid;

        async fn get_by_id(&self, id: Uuid) -> Result<Option<ModelInfo>, DatabaseError> {
            let models = self.models.read().unwrap();
            Ok(models.get(&id).cloned())
        }

        async fn get_all(&self) -> Result<Vec<ModelInfo>, DatabaseError> {
            let models = self.models.read().unwrap();
            Ok(models.values().cloned().collect())
        }

        async fn delete(&self, id: Uuid) -> Result<bool, DatabaseError> {
            let mut models = self.models.write().unwrap();
            Ok(models.remove(&id).is_some())
        }
    }
}

mod agent_repository_tests {
    use super::*;
    use mock_repository::MockAgentRepository;

    #[tokio::test]
    async fn test_create_and_get_agent() {
        let repo = MockAgentRepository::new();
        let agent = Agent::new(
            "test-agent".to_string(),
            AgentType::ClaudeCode,
            serde_json::json!({"model": "claude-3"}),
        );

        repo.create(&agent).unwrap();

        let retrieved = repo.get_by_id(agent.id).await.unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, agent.id);
        assert_eq!(retrieved.name, "test-agent");
        assert_eq!(retrieved.agent_type, AgentType::ClaudeCode);
    }

    #[tokio::test]
    async fn test_get_nonexistent_agent() {
        let repo = MockAgentRepository::new();
        let result = repo.get_by_id(Uuid::new_v4()).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_get_all_agents() {
        let repo = MockAgentRepository::new();

        let agent1 = Agent::new(
            "agent1".to_string(),
            AgentType::ClaudeCode,
            serde_json::json!({}),
        );
        let agent2 = Agent::new(
            "agent2".to_string(),
            AgentType::OpenCode,
            serde_json::json!({}),
        );
        let agent3 = Agent::new(
            "agent3".to_string(),
            AgentType::Codex,
            serde_json::json!({}),
        );

        repo.create(&agent1).unwrap();
        repo.create(&agent2).unwrap();
        repo.create(&agent3).unwrap();

        let all = repo.get_all().await.unwrap();
        assert_eq!(all.len(), 3);
    }

    #[tokio::test]
    async fn test_delete_agent() {
        let repo = MockAgentRepository::new();
        let agent = Agent::new(
            "to-delete".to_string(),
            AgentType::Copilot,
            serde_json::json!({}),
        );

        repo.create(&agent).unwrap();
        assert!(repo.get_by_id(agent.id).await.unwrap().is_some());

        let deleted = repo.delete(agent.id).await.unwrap();
        assert!(deleted);

        assert!(repo.get_by_id(agent.id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_delete_nonexistent_agent() {
        let repo = MockAgentRepository::new();
        let deleted = repo.delete(Uuid::new_v4()).await.unwrap();
        assert!(!deleted);
    }

    #[test]
    fn test_update_agent() {
        let repo = MockAgentRepository::new();
        let mut agent = Agent::new(
            "original".to_string(),
            AgentType::Goose,
            serde_json::json!({}),
        );

        repo.create(&agent).unwrap();

        agent.name = "updated".to_string();
        agent.config = serde_json::json!({"updated": true});

        let updated = repo.update(&agent).unwrap();
        assert!(updated.is_some());
        let updated = updated.unwrap();
        assert_eq!(updated.name, "updated");
        assert_eq!(updated.config["updated"], serde_json::json!(true));
    }

    #[test]
    fn test_get_by_type() {
        let repo = MockAgentRepository::new();

        let claude1 = Agent::new(
            "claude1".to_string(),
            AgentType::ClaudeCode,
            serde_json::json!({}),
        );
        let claude2 = Agent::new(
            "claude2".to_string(),
            AgentType::ClaudeCode,
            serde_json::json!({}),
        );
        let opencode = Agent::new(
            "opencode".to_string(),
            AgentType::OpenCode,
            serde_json::json!({}),
        );

        repo.create(&claude1).unwrap();
        repo.create(&claude2).unwrap();
        repo.create(&opencode).unwrap();

        let claude_agents = repo.get_by_type(AgentType::ClaudeCode).unwrap();
        assert_eq!(claude_agents.len(), 2);

        let opencode_agents = repo.get_by_type(AgentType::OpenCode).unwrap();
        assert_eq!(opencode_agents.len(), 1);

        let codex_agents = repo.get_by_type(AgentType::Codex).unwrap();
        assert_eq!(codex_agents.len(), 0);
    }

    #[test]
    fn test_get_by_name() {
        let repo = MockAgentRepository::new();

        let agent = Agent::new(
            "unique-name".to_string(),
            AgentType::Cursor,
            serde_json::json!({}),
        );
        repo.create(&agent).unwrap();

        let found = repo.get_by_name("unique-name").unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, agent.id);

        let not_found = repo.get_by_name("nonexistent").unwrap();
        assert!(not_found.is_none());
    }

    #[test]
    fn test_count() {
        let repo = MockAgentRepository::new();
        assert_eq!(repo.count().unwrap(), 0);

        repo.create(&Agent::new(
            "a1".to_string(),
            AgentType::ClaudeCode,
            serde_json::json!({}),
        ))
        .unwrap();
        assert_eq!(repo.count().unwrap(), 1);

        repo.create(&Agent::new(
            "a2".to_string(),
            AgentType::OpenCode,
            serde_json::json!({}),
        ))
        .unwrap();
        assert_eq!(repo.count().unwrap(), 2);
    }
}

mod session_repository_tests {
    use super::*;
    use mock_repository::MockSessionRepository;

    #[tokio::test]
    async fn test_create_and_get_session() {
        let repo = MockSessionRepository::new();
        let agent_id = Uuid::new_v4();
        let session = Session::new(agent_id, serde_json::json!({"project": "/test"}));

        repo.create(&session).unwrap();

        let retrieved = repo.get_by_id(session.id).await.unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.agent_id, agent_id);
        assert_eq!(retrieved.status, SessionStatus::Active);
    }

    #[tokio::test]
    async fn test_update_session_status() {
        let repo = MockSessionRepository::new();
        let mut session = Session::new(Uuid::new_v4(), serde_json::json!({}));

        repo.create(&session).unwrap();

        session.end(SessionStatus::Completed);
        repo.update(&session).unwrap();

        let retrieved = repo.get_by_id(session.id).await.unwrap().unwrap();
        assert_eq!(retrieved.status, SessionStatus::Completed);
        assert!(retrieved.ended_at.is_some());
    }

    #[test]
    fn test_get_by_agent_id() {
        let repo = MockSessionRepository::new();
        let agent1 = Uuid::new_v4();
        let agent2 = Uuid::new_v4();

        let session1 = Session::new(agent1, serde_json::json!({}));
        let session2 = Session::new(agent1, serde_json::json!({}));
        let session3 = Session::new(agent2, serde_json::json!({}));

        repo.create(&session1).unwrap();
        repo.create(&session2).unwrap();
        repo.create(&session3).unwrap();

        let agent1_sessions = repo.get_by_agent_id(agent1).unwrap();
        assert_eq!(agent1_sessions.len(), 2);

        let agent2_sessions = repo.get_by_agent_id(agent2).unwrap();
        assert_eq!(agent2_sessions.len(), 1);
    }

    #[test]
    fn test_get_active_sessions() {
        let repo = MockSessionRepository::new();
        let agent_id = Uuid::new_v4();

        let session1 = Session::new(agent_id, serde_json::json!({}));
        let mut session2 = Session::new(agent_id, serde_json::json!({}));
        session2.end(SessionStatus::Completed);
        let mut session3 = Session::new(agent_id, serde_json::json!({}));
        session3.end(SessionStatus::Failed);

        repo.create(&session1).unwrap();
        repo.create(&session2).unwrap();
        repo.create(&session3).unwrap();

        let active = repo.get_active().unwrap();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].id, session1.id);
    }

    #[tokio::test]
    async fn test_delete_session() {
        let repo = MockSessionRepository::new();
        let session = Session::new(Uuid::new_v4(), serde_json::json!({}));

        repo.create(&session).unwrap();
        assert!(repo.get_by_id(session.id).await.unwrap().is_some());

        repo.delete(session.id).await.unwrap();
        assert!(repo.get_by_id(session.id).await.unwrap().is_none());
    }
}

mod cost_repository_tests {
    use super::*;
    use mock_repository::MockCostRepository;

    #[tokio::test]
    async fn test_create_and_get_cost() {
        let repo = MockCostRepository::new();
        let cost = CostRecord::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "claude-3-opus".to_string(),
            10000,
            5000,
            0.15,
        );

        repo.create(&cost).unwrap();

        let retrieved = repo.get_by_id(cost.id).await.unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.model_name, "claude-3-opus");
        assert_eq!(retrieved.input_tokens, 10000);
        assert_eq!(retrieved.output_tokens, 5000);
    }

    #[test]
    fn test_get_by_session_id() {
        let repo = MockCostRepository::new();
        let session1 = Uuid::new_v4();
        let session2 = Uuid::new_v4();
        let agent_id = Uuid::new_v4();

        let cost1 = CostRecord::new(session1, agent_id, "model1".to_string(), 100, 50, 0.01);
        let cost2 = CostRecord::new(session1, agent_id, "model2".to_string(), 200, 100, 0.02);
        let cost3 = CostRecord::new(session2, agent_id, "model1".to_string(), 300, 150, 0.03);

        repo.create(&cost1).unwrap();
        repo.create(&cost2).unwrap();
        repo.create(&cost3).unwrap();

        let session1_costs = repo.get_by_session_id(session1).unwrap();
        assert_eq!(session1_costs.len(), 2);

        let session2_costs = repo.get_by_session_id(session2).unwrap();
        assert_eq!(session2_costs.len(), 1);
    }

    #[test]
    fn test_get_by_agent_id() {
        let repo = MockCostRepository::new();
        let agent1 = Uuid::new_v4();
        let agent2 = Uuid::new_v4();

        let cost1 = CostRecord::new(Uuid::new_v4(), agent1, "model".to_string(), 100, 50, 0.01);
        let cost2 = CostRecord::new(Uuid::new_v4(), agent1, "model".to_string(), 200, 100, 0.02);
        let cost3 = CostRecord::new(Uuid::new_v4(), agent2, "model".to_string(), 300, 150, 0.03);

        repo.create(&cost1).unwrap();
        repo.create(&cost2).unwrap();
        repo.create(&cost3).unwrap();

        let agent1_costs = repo.get_by_agent_id(agent1).unwrap();
        assert_eq!(agent1_costs.len(), 2);
    }

    #[test]
    fn test_get_total_cost() {
        let repo = MockCostRepository::new();

        let cost1 = CostRecord::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "m".to_string(),
            100,
            50,
            0.10,
        );
        let cost2 = CostRecord::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "m".to_string(),
            200,
            100,
            0.20,
        );
        let cost3 = CostRecord::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "m".to_string(),
            300,
            150,
            0.30,
        );

        repo.create(&cost1).unwrap();
        repo.create(&cost2).unwrap();
        repo.create(&cost3).unwrap();

        let total = repo.get_total_cost();
        assert!((total - 0.60).abs() < 0.0001);
    }

    #[tokio::test]
    async fn test_delete_cost() {
        let repo = MockCostRepository::new();
        let cost = CostRecord::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "model".to_string(),
            100,
            50,
            0.01,
        );

        repo.create(&cost).unwrap();
        assert!(repo.get_by_id(cost.id).await.unwrap().is_some());

        repo.delete(cost.id).await.unwrap();
        assert!(repo.get_by_id(cost.id).await.unwrap().is_none());
    }
}

mod model_repository_tests {
    use super::*;
    use mock_repository::MockModelRepository;

    #[tokio::test]
    async fn test_create_and_get_model() {
        let repo = MockModelRepository::new();
        let model = ModelInfo::new(
            "anthropic".to_string(),
            "claude-3-opus".to_string(),
            0.015,
            0.075,
            200000,
        );

        repo.create(&model).unwrap();

        let retrieved = repo.get_by_id(model.id).await.unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.provider, "anthropic");
        assert_eq!(retrieved.model_name, "claude-3-opus");
    }

    #[test]
    fn test_get_by_provider() {
        let repo = MockModelRepository::new();

        let model1 = ModelInfo::new(
            "anthropic".to_string(),
            "claude-3-opus".to_string(),
            0.015,
            0.075,
            200000,
        );
        let model2 = ModelInfo::new(
            "anthropic".to_string(),
            "claude-3-sonnet".to_string(),
            0.003,
            0.015,
            200000,
        );
        let model3 = ModelInfo::new(
            "openai".to_string(),
            "gpt-4".to_string(),
            0.03,
            0.06,
            128000,
        );

        repo.create(&model1).unwrap();
        repo.create(&model2).unwrap();
        repo.create(&model3).unwrap();

        let anthropic_models = repo.get_by_provider("anthropic").unwrap();
        assert_eq!(anthropic_models.len(), 2);

        let openai_models = repo.get_by_provider("openai").unwrap();
        assert_eq!(openai_models.len(), 1);

        let google_models = repo.get_by_provider("google").unwrap();
        assert_eq!(google_models.len(), 0);
    }

    #[test]
    fn test_get_by_name() {
        let repo = MockModelRepository::new();

        let model = ModelInfo::new(
            "openai".to_string(),
            "gpt-4-turbo".to_string(),
            0.01,
            0.03,
            128000,
        );
        repo.create(&model).unwrap();

        let found = repo.get_by_name("gpt-4-turbo").unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().provider, "openai");

        let not_found = repo.get_by_name("nonexistent").unwrap();
        assert!(not_found.is_none());
    }

    #[tokio::test]
    async fn test_get_all_models() {
        let repo = MockModelRepository::new();

        repo.create(&ModelInfo::new(
            "a".to_string(),
            "m1".to_string(),
            0.01,
            0.02,
            4096,
        ))
        .unwrap();
        repo.create(&ModelInfo::new(
            "b".to_string(),
            "m2".to_string(),
            0.01,
            0.02,
            4096,
        ))
        .unwrap();
        repo.create(&ModelInfo::new(
            "c".to_string(),
            "m3".to_string(),
            0.01,
            0.02,
            4096,
        ))
        .unwrap();

        let all = repo.get_all().await.unwrap();
        assert_eq!(all.len(), 3);
    }

    #[tokio::test]
    async fn test_delete_model() {
        let repo = MockModelRepository::new();
        let model = ModelInfo::new(
            "test".to_string(),
            "test-model".to_string(),
            0.01,
            0.02,
            4096,
        );

        repo.create(&model).unwrap();
        assert!(repo.get_by_id(model.id).await.unwrap().is_some());

        repo.delete(model.id).await.unwrap();
        assert!(repo.get_by_id(model.id).await.unwrap().is_none());
    }
}
