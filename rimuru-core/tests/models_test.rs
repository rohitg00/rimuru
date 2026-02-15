use chrono::Utc;
use rimuru_core::models::{
    Agent, AgentType, CostRecord, CostSummary, MetricsSnapshot, ModelInfo, Session, SessionStatus,
    SystemMetrics,
};
use uuid::Uuid;

mod agent_tests {
    use super::*;

    #[test]
    fn test_agent_type_serialization() {
        let agent_types = vec![
            AgentType::ClaudeCode,
            AgentType::Codex,
            AgentType::Copilot,
            AgentType::Goose,
            AgentType::OpenCode,
            AgentType::Cursor,
        ];

        for agent_type in agent_types {
            let json = serde_json::to_string(&agent_type).unwrap();
            let deserialized: AgentType = serde_json::from_str(&json).unwrap();
            assert_eq!(agent_type, deserialized);
        }
    }

    #[test]
    fn test_agent_type_yaml_serialization() {
        let agent_type = AgentType::ClaudeCode;
        let yaml = serde_yaml::to_string(&agent_type).unwrap();
        assert!(yaml.contains("claude_code"));
        let deserialized: AgentType = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(agent_type, deserialized);
    }

    #[test]
    fn test_agent_serialization_roundtrip() {
        let config = serde_json::json!({
            "model": "claude-3-opus",
            "max_tokens": 4096,
            "temperature": 0.7
        });
        let agent = Agent::new(
            "test-agent".to_string(),
            AgentType::ClaudeCode,
            config.clone(),
        );

        let json = serde_json::to_string(&agent).unwrap();
        let deserialized: Agent = serde_json::from_str(&json).unwrap();

        assert_eq!(agent.id, deserialized.id);
        assert_eq!(agent.name, deserialized.name);
        assert_eq!(agent.agent_type, deserialized.agent_type);
        assert_eq!(agent.config, deserialized.config);
    }

    #[test]
    fn test_agent_with_complex_config() {
        let config = serde_json::json!({
            "nested": {
                "deeply": {
                    "value": [1, 2, 3]
                }
            },
            "array": ["a", "b", "c"],
            "boolean": true,
            "null_field": null
        });
        let agent = Agent::new("complex-config".to_string(), AgentType::OpenCode, config);

        let json = serde_json::to_string(&agent).unwrap();
        let deserialized: Agent = serde_json::from_str(&json).unwrap();

        assert_eq!(
            deserialized.config["nested"]["deeply"]["value"][0],
            serde_json::json!(1)
        );
        assert_eq!(deserialized.config["array"][1], serde_json::json!("b"));
    }

    #[test]
    fn test_agent_timestamps() {
        let before = Utc::now();
        let agent = Agent::new(
            "timestamp-test".to_string(),
            AgentType::Codex,
            serde_json::json!({}),
        );
        let after = Utc::now();

        assert!(agent.created_at >= before && agent.created_at <= after);
        assert!(agent.updated_at >= before && agent.updated_at <= after);
        assert_eq!(agent.created_at, agent.updated_at);
    }

    #[test]
    fn test_agent_uuid_uniqueness() {
        let agent1 = Agent::new(
            "agent1".to_string(),
            AgentType::ClaudeCode,
            serde_json::json!({}),
        );
        let agent2 = Agent::new(
            "agent2".to_string(),
            AgentType::ClaudeCode,
            serde_json::json!({}),
        );

        assert_ne!(agent1.id, agent2.id);
    }
}

mod session_tests {
    use super::*;

    #[test]
    fn test_session_status_serialization() {
        let statuses = vec![
            SessionStatus::Active,
            SessionStatus::Completed,
            SessionStatus::Failed,
            SessionStatus::Terminated,
        ];

        for status in statuses {
            let json = serde_json::to_string(&status).unwrap();
            let deserialized: SessionStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(status, deserialized);
        }
    }

    #[test]
    fn test_session_serialization_roundtrip() {
        let agent_id = Uuid::new_v4();
        let metadata = serde_json::json!({
            "project": "/home/user/project",
            "model": "claude-3-opus",
            "context_tokens": 5000
        });
        let session = Session::new(agent_id, metadata.clone());

        let json = serde_json::to_string(&session).unwrap();
        let deserialized: Session = serde_json::from_str(&json).unwrap();

        assert_eq!(session.id, deserialized.id);
        assert_eq!(session.agent_id, deserialized.agent_id);
        assert_eq!(session.status, deserialized.status);
        assert_eq!(session.metadata, deserialized.metadata);
    }

    #[test]
    fn test_session_lifecycle() {
        let agent_id = Uuid::new_v4();
        let mut session = Session::new(agent_id, serde_json::json!({}));

        assert!(session.is_active());
        assert!(session.ended_at.is_none());
        assert!(session.duration_seconds().is_none());

        session.end(SessionStatus::Completed);

        assert!(!session.is_active());
        assert!(session.ended_at.is_some());
        assert!(session.duration_seconds().is_some());
        assert!(session.duration_seconds().unwrap() >= 0);
    }

    #[test]
    fn test_session_end_with_different_statuses() {
        let agent_id = Uuid::new_v4();

        let mut session_completed = Session::new(agent_id, serde_json::json!({}));
        session_completed.end(SessionStatus::Completed);
        assert_eq!(session_completed.status, SessionStatus::Completed);

        let mut session_failed = Session::new(agent_id, serde_json::json!({}));
        session_failed.end(SessionStatus::Failed);
        assert_eq!(session_failed.status, SessionStatus::Failed);

        let mut session_terminated = Session::new(agent_id, serde_json::json!({}));
        session_terminated.end(SessionStatus::Terminated);
        assert_eq!(session_terminated.status, SessionStatus::Terminated);
    }

    #[test]
    fn test_session_yaml_serialization() {
        let session = Session::new(Uuid::new_v4(), serde_json::json!({"key": "value"}));
        let yaml = serde_yaml::to_string(&session).unwrap();
        let deserialized: Session = serde_yaml::from_str(&yaml).unwrap();

        assert_eq!(session.id, deserialized.id);
        assert_eq!(session.status, deserialized.status);
    }
}

mod cost_tests {
    use super::*;

    #[test]
    fn test_cost_record_serialization_roundtrip() {
        let session_id = Uuid::new_v4();
        let agent_id = Uuid::new_v4();
        let record = CostRecord::new(
            session_id,
            agent_id,
            "claude-3-opus".to_string(),
            10000,
            5000,
            0.15,
        );

        let json = serde_json::to_string(&record).unwrap();
        let deserialized: CostRecord = serde_json::from_str(&json).unwrap();

        assert_eq!(record.id, deserialized.id);
        assert_eq!(record.session_id, deserialized.session_id);
        assert_eq!(record.agent_id, deserialized.agent_id);
        assert_eq!(record.model_name, deserialized.model_name);
        assert_eq!(record.input_tokens, deserialized.input_tokens);
        assert_eq!(record.output_tokens, deserialized.output_tokens);
        assert!((record.cost_usd - deserialized.cost_usd).abs() < 0.0001);
    }

    #[test]
    fn test_cost_record_total_tokens() {
        let record = CostRecord::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "gpt-4".to_string(),
            1000,
            500,
            0.05,
        );
        assert_eq!(record.total_tokens(), 1500);
    }

    #[test]
    fn test_cost_record_zero_tokens() {
        let record = CostRecord::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "test-model".to_string(),
            0,
            0,
            0.0,
        );
        assert_eq!(record.total_tokens(), 0);
    }

    #[test]
    fn test_cost_summary_serialization() {
        let summary = CostSummary {
            total_cost_usd: 10.50,
            total_input_tokens: 100000,
            total_output_tokens: 50000,
            record_count: 100,
        };

        let json = serde_json::to_string(&summary).unwrap();
        let deserialized: CostSummary = serde_json::from_str(&json).unwrap();

        assert!((summary.total_cost_usd - deserialized.total_cost_usd).abs() < 0.0001);
        assert_eq!(summary.total_input_tokens, deserialized.total_input_tokens);
        assert_eq!(
            summary.total_output_tokens,
            deserialized.total_output_tokens
        );
        assert_eq!(summary.record_count, deserialized.record_count);
    }

    #[test]
    fn test_cost_summary_calculations() {
        let summary = CostSummary {
            total_cost_usd: 5.0,
            total_input_tokens: 10000,
            total_output_tokens: 5000,
            record_count: 50,
        };

        assert_eq!(summary.total_tokens(), 15000);
        assert!((summary.average_cost_per_request() - 0.1).abs() < 0.0001);
    }

    #[test]
    fn test_cost_summary_empty() {
        let summary = CostSummary::default();

        assert_eq!(summary.total_tokens(), 0);
        assert_eq!(summary.average_cost_per_request(), 0.0);
    }

    #[test]
    fn test_cost_summary_yaml_serialization() {
        let summary = CostSummary {
            total_cost_usd: 25.75,
            total_input_tokens: 250000,
            total_output_tokens: 125000,
            record_count: 500,
        };

        let yaml = serde_yaml::to_string(&summary).unwrap();
        let deserialized: CostSummary = serde_yaml::from_str(&yaml).unwrap();

        assert!((summary.total_cost_usd - deserialized.total_cost_usd).abs() < 0.0001);
    }
}

mod model_info_tests {
    use super::*;

    #[test]
    fn test_model_info_serialization_roundtrip() {
        let model = ModelInfo::new(
            "anthropic".to_string(),
            "claude-3-opus".to_string(),
            0.015,
            0.075,
            200000,
        );

        let json = serde_json::to_string(&model).unwrap();
        let deserialized: ModelInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(model.id, deserialized.id);
        assert_eq!(model.provider, deserialized.provider);
        assert_eq!(model.model_name, deserialized.model_name);
        assert!((model.input_price_per_1k - deserialized.input_price_per_1k).abs() < 0.0001);
        assert!((model.output_price_per_1k - deserialized.output_price_per_1k).abs() < 0.0001);
        assert_eq!(model.context_window, deserialized.context_window);
    }

    #[test]
    fn test_model_info_calculate_cost() {
        let model = ModelInfo::new(
            "openai".to_string(),
            "gpt-4".to_string(),
            0.03,
            0.06,
            128000,
        );

        let cost = model.calculate_cost(10000, 5000);
        let expected = (10000.0 / 1000.0) * 0.03 + (5000.0 / 1000.0) * 0.06;

        assert!((cost - expected).abs() < 0.0001);
    }

    #[test]
    fn test_model_info_calculate_cost_zero_tokens() {
        let model = ModelInfo::new(
            "test".to_string(),
            "test-model".to_string(),
            0.01,
            0.02,
            4096,
        );

        let cost = model.calculate_cost(0, 0);
        assert_eq!(cost, 0.0);
    }

    #[test]
    fn test_model_info_full_name() {
        let model = ModelInfo::new(
            "anthropic".to_string(),
            "claude-3-sonnet".to_string(),
            0.003,
            0.015,
            200000,
        );

        assert_eq!(model.full_name(), "anthropic/claude-3-sonnet");
    }

    #[test]
    fn test_model_info_yaml_serialization() {
        let model = ModelInfo::new(
            "google".to_string(),
            "gemini-pro".to_string(),
            0.00025,
            0.0005,
            32000,
        );

        let yaml = serde_yaml::to_string(&model).unwrap();
        let deserialized: ModelInfo = serde_yaml::from_str(&yaml).unwrap();

        assert_eq!(model.provider, deserialized.provider);
        assert_eq!(model.model_name, deserialized.model_name);
    }
}

mod system_metrics_tests {
    use super::*;

    #[test]
    fn test_system_metrics_serialization_roundtrip() {
        let metrics = SystemMetrics::new(75.5, 8192, 16384, 5);

        let json = serde_json::to_string(&metrics).unwrap();
        let deserialized: SystemMetrics = serde_json::from_str(&json).unwrap();

        assert!((metrics.cpu_percent - deserialized.cpu_percent).abs() < 0.01);
        assert_eq!(metrics.memory_used_mb, deserialized.memory_used_mb);
        assert_eq!(metrics.memory_total_mb, deserialized.memory_total_mb);
        assert_eq!(metrics.active_sessions, deserialized.active_sessions);
    }

    #[test]
    fn test_system_metrics_memory_percent() {
        let metrics = SystemMetrics::new(50.0, 8192, 16384, 0);
        assert!((metrics.memory_percent() - 50.0).abs() < 0.01);

        let quarter_used = SystemMetrics::new(50.0, 4096, 16384, 0);
        assert!((quarter_used.memory_percent() - 25.0).abs() < 0.01);
    }

    #[test]
    fn test_system_metrics_memory_percent_zero_total() {
        let metrics = SystemMetrics::new(50.0, 0, 0, 0);
        assert_eq!(metrics.memory_percent(), 0.0);
    }

    #[test]
    fn test_system_metrics_memory_available() {
        let metrics = SystemMetrics::new(50.0, 12000, 16384, 3);
        assert_eq!(metrics.memory_available_mb(), 4384);
    }

    #[test]
    fn test_metrics_snapshot_from_system_metrics() {
        let metrics = SystemMetrics::new(65.5, 6000, 16000, 4);
        let snapshot: MetricsSnapshot = metrics.into();

        assert!((snapshot.cpu_percent - 65.5).abs() < 0.01);
        assert_eq!(snapshot.memory_used_mb, 6000);
        assert_eq!(snapshot.memory_total_mb, 16000);
        assert!((snapshot.memory_percent - 37.5).abs() < 0.01);
        assert_eq!(snapshot.active_sessions, 4);
    }

    #[test]
    fn test_metrics_snapshot_serialization() {
        let snapshot = MetricsSnapshot {
            cpu_percent: 42.5,
            memory_used_mb: 7500,
            memory_total_mb: 16000,
            memory_percent: 46.875,
            active_sessions: 2,
        };

        let json = serde_json::to_string(&snapshot).unwrap();
        let deserialized: MetricsSnapshot = serde_json::from_str(&json).unwrap();

        assert!((snapshot.cpu_percent - deserialized.cpu_percent).abs() < 0.01);
        assert_eq!(snapshot.active_sessions, deserialized.active_sessions);
    }

    #[test]
    fn test_metrics_snapshot_default() {
        let snapshot = MetricsSnapshot::default();

        assert_eq!(snapshot.cpu_percent, 0.0);
        assert_eq!(snapshot.memory_used_mb, 0);
        assert_eq!(snapshot.memory_total_mb, 0);
        assert_eq!(snapshot.memory_percent, 0.0);
        assert_eq!(snapshot.active_sessions, 0);
    }

    #[test]
    fn test_system_metrics_yaml_serialization() {
        let metrics = SystemMetrics::new(88.2, 14000, 16384, 8);

        let yaml = serde_yaml::to_string(&metrics).unwrap();
        let deserialized: SystemMetrics = serde_yaml::from_str(&yaml).unwrap();

        assert!((metrics.cpu_percent - deserialized.cpu_percent).abs() < 0.01);
        assert_eq!(metrics.memory_used_mb, deserialized.memory_used_mb);
    }
}
