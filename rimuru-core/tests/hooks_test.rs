use rimuru_core::hooks::{
    Hook, HookConfig, HookContext, HookData, HookExecution, HookHandlerInfo, HookResult,
};
use rimuru_core::models::{CostRecord, MetricsSnapshot, Session, SessionStatus};
use uuid::Uuid;

mod hook_tests {
    use super::*;

    #[test]
    fn test_hook_name() {
        assert_eq!(Hook::PreSessionStart.name(), "pre_session_start");
        assert_eq!(Hook::PostSessionEnd.name(), "post_session_end");
        assert_eq!(Hook::OnCostRecorded.name(), "on_cost_recorded");
        assert_eq!(Hook::OnMetricsCollected.name(), "on_metrics_collected");
        assert_eq!(Hook::OnAgentConnect.name(), "on_agent_connect");
        assert_eq!(Hook::OnAgentDisconnect.name(), "on_agent_disconnect");
        assert_eq!(Hook::OnSyncComplete.name(), "on_sync_complete");
        assert_eq!(Hook::OnPluginLoaded.name(), "on_plugin_loaded");
        assert_eq!(Hook::OnPluginUnloaded.name(), "on_plugin_unloaded");
        assert_eq!(Hook::OnConfigChanged.name(), "on_config_changed");
        assert_eq!(Hook::OnError.name(), "on_error");
        assert_eq!(Hook::Custom("my_hook".to_string()).name(), "my_hook");
    }

    #[test]
    fn test_hook_from_name() {
        assert_eq!(Hook::from_name("pre_session_start"), Hook::PreSessionStart);
        assert_eq!(Hook::from_name("post_session_end"), Hook::PostSessionEnd);
        assert_eq!(Hook::from_name("on_cost_recorded"), Hook::OnCostRecorded);
        assert_eq!(
            Hook::from_name("on_metrics_collected"),
            Hook::OnMetricsCollected
        );
        assert_eq!(Hook::from_name("on_agent_connect"), Hook::OnAgentConnect);
        assert_eq!(
            Hook::from_name("on_agent_disconnect"),
            Hook::OnAgentDisconnect
        );
        assert_eq!(Hook::from_name("on_sync_complete"), Hook::OnSyncComplete);
        assert_eq!(Hook::from_name("on_plugin_loaded"), Hook::OnPluginLoaded);
        assert_eq!(
            Hook::from_name("on_plugin_unloaded"),
            Hook::OnPluginUnloaded
        );
        assert_eq!(Hook::from_name("on_config_changed"), Hook::OnConfigChanged);
        assert_eq!(Hook::from_name("on_error"), Hook::OnError);
        assert_eq!(
            Hook::from_name("custom_hook"),
            Hook::Custom("custom_hook".to_string())
        );
    }

    #[test]
    fn test_hook_all_standard() {
        let standard = Hook::all_standard();

        assert_eq!(standard.len(), 11);
        assert!(standard.contains(&Hook::PreSessionStart));
        assert!(standard.contains(&Hook::PostSessionEnd));
        assert!(standard.contains(&Hook::OnCostRecorded));
        assert!(standard.contains(&Hook::OnMetricsCollected));
        assert!(standard.contains(&Hook::OnAgentConnect));
        assert!(standard.contains(&Hook::OnAgentDisconnect));
        assert!(standard.contains(&Hook::OnSyncComplete));
        assert!(standard.contains(&Hook::OnPluginLoaded));
        assert!(standard.contains(&Hook::OnPluginUnloaded));
        assert!(standard.contains(&Hook::OnConfigChanged));
        assert!(standard.contains(&Hook::OnError));
    }

    #[test]
    fn test_hook_display() {
        assert_eq!(Hook::PreSessionStart.to_string(), "pre_session_start");
        assert_eq!(
            Hook::Custom("display_test".to_string()).to_string(),
            "display_test"
        );
    }

    #[test]
    fn test_hook_serialization() {
        let standard_hooks = vec![
            Hook::PreSessionStart,
            Hook::PostSessionEnd,
            Hook::OnCostRecorded,
            Hook::OnMetricsCollected,
            Hook::OnAgentConnect,
        ];

        for hook in standard_hooks {
            let json = serde_json::to_string(&hook).unwrap();
            let deserialized: Hook = serde_json::from_str(&json).unwrap();
            assert_eq!(hook, deserialized);
        }
    }

    #[test]
    fn test_hook_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(Hook::PreSessionStart);
        set.insert(Hook::PostSessionEnd);
        set.insert(Hook::PreSessionStart);

        assert_eq!(set.len(), 2);
    }
}

mod hook_data_tests {
    use super::*;

    #[test]
    fn test_hook_data_session() {
        let session = Session::new(Uuid::new_v4(), serde_json::json!({}));
        let data = HookData::Session(session.clone());

        if let HookData::Session(s) = data {
            assert_eq!(s.id, session.id);
        } else {
            panic!("Expected Session data");
        }
    }

    #[test]
    fn test_hook_data_cost() {
        let cost = CostRecord::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "claude-3-opus".to_string(),
            1000,
            500,
            0.05,
        );
        let data = HookData::Cost(cost.clone());

        if let HookData::Cost(c) = data {
            assert_eq!(c.id, cost.id);
            assert_eq!(c.model_name, "claude-3-opus");
        } else {
            panic!("Expected Cost data");
        }
    }

    #[test]
    fn test_hook_data_metrics() {
        let metrics = MetricsSnapshot {
            cpu_percent: 45.5,
            memory_used_mb: 8192,
            memory_total_mb: 16384,
            memory_percent: 50.0,
            active_sessions: 3,
        };
        let data = HookData::Metrics(metrics.clone());

        if let HookData::Metrics(m) = data {
            assert!((m.cpu_percent - 45.5).abs() < 0.01);
            assert_eq!(m.active_sessions, 3);
        } else {
            panic!("Expected Metrics data");
        }
    }

    #[test]
    fn test_hook_data_agent() {
        let agent_id = Uuid::new_v4();
        let data = HookData::Agent {
            agent_id,
            agent_name: "test-agent".to_string(),
            agent_type: "claude_code".to_string(),
        };

        if let HookData::Agent {
            agent_id: id,
            agent_name,
            agent_type,
        } = data
        {
            assert_eq!(id, agent_id);
            assert_eq!(agent_name, "test-agent");
            assert_eq!(agent_type, "claude_code");
        } else {
            panic!("Expected Agent data");
        }
    }

    #[test]
    fn test_hook_data_sync() {
        let data = HookData::Sync {
            provider: "anthropic".to_string(),
            models_synced: 15,
            duration_ms: 250,
        };

        if let HookData::Sync {
            provider,
            models_synced,
            duration_ms,
        } = data
        {
            assert_eq!(provider, "anthropic");
            assert_eq!(models_synced, 15);
            assert_eq!(duration_ms, 250);
        } else {
            panic!("Expected Sync data");
        }
    }

    #[test]
    fn test_hook_data_plugin() {
        let data = HookData::Plugin {
            plugin_id: "my-plugin".to_string(),
            plugin_name: "My Plugin".to_string(),
        };

        if let HookData::Plugin {
            plugin_id,
            plugin_name,
        } = data
        {
            assert_eq!(plugin_id, "my-plugin");
            assert_eq!(plugin_name, "My Plugin");
        } else {
            panic!("Expected Plugin data");
        }
    }

    #[test]
    fn test_hook_data_config() {
        let data = HookData::Config {
            changed_keys: vec!["key1".to_string(), "key2".to_string()],
        };

        if let HookData::Config { changed_keys } = data {
            assert_eq!(changed_keys.len(), 2);
            assert!(changed_keys.contains(&"key1".to_string()));
        } else {
            panic!("Expected Config data");
        }
    }

    #[test]
    fn test_hook_data_error() {
        let data = HookData::Error {
            error_code: "E1001".to_string(),
            error_message: "Connection failed".to_string(),
            source: Some("database".to_string()),
        };

        if let HookData::Error {
            error_code,
            error_message,
            source,
        } = data
        {
            assert_eq!(error_code, "E1001");
            assert_eq!(error_message, "Connection failed");
            assert_eq!(source, Some("database".to_string()));
        } else {
            panic!("Expected Error data");
        }
    }

    #[test]
    fn test_hook_data_custom() {
        let custom = serde_json::json!({"key": "value", "number": 42});
        let data = HookData::Custom(custom.clone());

        if let HookData::Custom(value) = data {
            assert_eq!(value["key"], "value");
            assert_eq!(value["number"], 42);
        } else {
            panic!("Expected Custom data");
        }
    }

    #[test]
    fn test_hook_data_none() {
        let data = HookData::None;
        assert!(matches!(data, HookData::None));
    }

    #[test]
    fn test_hook_data_default() {
        let data = HookData::default();
        assert!(matches!(data, HookData::None));
    }

    #[test]
    fn test_hook_data_serialization() {
        let session = Session::new(Uuid::new_v4(), serde_json::json!({}));
        let data = HookData::Session(session);

        let json = serde_json::to_string(&data).unwrap();
        let deserialized: HookData = serde_json::from_str(&json).unwrap();

        assert!(matches!(deserialized, HookData::Session(_)));
    }
}

mod hook_context_tests {
    use super::*;

    #[test]
    fn test_hook_context_new() {
        let ctx = HookContext::new(Hook::OnCostRecorded, HookData::None);

        assert_eq!(ctx.hook, Hook::OnCostRecorded);
        assert!(matches!(ctx.data, HookData::None));
        assert!(ctx.source.is_empty());
        assert!(ctx.metadata.is_empty());
    }

    #[test]
    fn test_hook_context_with_source() {
        let ctx =
            HookContext::new(Hook::PreSessionStart, HookData::None).with_source("test_source");

        assert_eq!(ctx.source, "test_source");
    }

    #[test]
    fn test_hook_context_with_correlation_id() {
        let correlation_id = Uuid::new_v4();
        let ctx = HookContext::new(Hook::PostSessionEnd, HookData::None)
            .with_correlation_id(correlation_id);

        assert_eq!(ctx.correlation_id, correlation_id);
    }

    #[test]
    fn test_hook_context_with_metadata() {
        let ctx = HookContext::new(Hook::OnError, HookData::None)
            .with_metadata("key1", "value1")
            .with_metadata("key2", 42)
            .with_metadata("key3", true);

        assert_eq!(
            ctx.get_metadata::<String>("key1"),
            Some("value1".to_string())
        );
        assert_eq!(ctx.get_metadata::<i32>("key2"), Some(42));
        assert_eq!(ctx.get_metadata::<bool>("key3"), Some(true));
    }

    #[test]
    fn test_hook_context_get_missing_metadata() {
        let ctx = HookContext::new(Hook::OnSyncComplete, HookData::None);
        let missing: Option<String> = ctx.get_metadata("nonexistent");
        assert!(missing.is_none());
    }

    #[test]
    fn test_hook_context_session_start() {
        let session = Session::new(Uuid::new_v4(), serde_json::json!({}));
        let ctx = HookContext::session_start(session.clone());

        assert_eq!(ctx.hook, Hook::PreSessionStart);
        assert_eq!(ctx.source, "session_manager");
        if let HookData::Session(s) = ctx.data {
            assert_eq!(s.id, session.id);
        } else {
            panic!("Expected Session data");
        }
    }

    #[test]
    fn test_hook_context_session_end() {
        let mut session = Session::new(Uuid::new_v4(), serde_json::json!({}));
        session.end(SessionStatus::Completed);
        let ctx = HookContext::session_end(session.clone());

        assert_eq!(ctx.hook, Hook::PostSessionEnd);
        assert_eq!(ctx.source, "session_manager");
    }

    #[test]
    fn test_hook_context_cost_recorded() {
        let cost = CostRecord::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "model".to_string(),
            100,
            50,
            0.01,
        );
        let ctx = HookContext::cost_recorded(cost.clone());

        assert_eq!(ctx.hook, Hook::OnCostRecorded);
        assert_eq!(ctx.source, "cost_tracker");
    }

    #[test]
    fn test_hook_context_metrics_collected() {
        let metrics = MetricsSnapshot::default();
        let ctx = HookContext::metrics_collected(metrics);

        assert_eq!(ctx.hook, Hook::OnMetricsCollected);
        assert_eq!(ctx.source, "metrics_collector");
    }

    #[test]
    fn test_hook_context_agent_connect() {
        let agent_id = Uuid::new_v4();
        let ctx = HookContext::agent_connect(agent_id, "test-agent", "claude_code");

        assert_eq!(ctx.hook, Hook::OnAgentConnect);
        assert_eq!(ctx.source, "adapter_manager");
    }

    #[test]
    fn test_hook_context_agent_disconnect() {
        let agent_id = Uuid::new_v4();
        let ctx = HookContext::agent_disconnect(agent_id, "test-agent", "opencode");

        assert_eq!(ctx.hook, Hook::OnAgentDisconnect);
        assert_eq!(ctx.source, "adapter_manager");
    }

    #[test]
    fn test_hook_context_sync_complete() {
        let ctx = HookContext::sync_complete("anthropic", 20, 500);

        assert_eq!(ctx.hook, Hook::OnSyncComplete);
        assert_eq!(ctx.source, "sync_scheduler");
        if let HookData::Sync {
            provider,
            models_synced,
            duration_ms,
        } = ctx.data
        {
            assert_eq!(provider, "anthropic");
            assert_eq!(models_synced, 20);
            assert_eq!(duration_ms, 500);
        } else {
            panic!("Expected Sync data");
        }
    }

    #[test]
    fn test_hook_context_error() {
        let ctx = HookContext::error(
            "E1001",
            "Database connection failed",
            Some("db".to_string()),
        );

        assert_eq!(ctx.hook, Hook::OnError);
        assert_eq!(ctx.source, "error_handler");
    }

    #[test]
    fn test_hook_context_serialization() {
        let ctx = HookContext::new(Hook::OnCostRecorded, HookData::None)
            .with_source("test")
            .with_metadata("key", "value");

        let json = serde_json::to_string(&ctx).unwrap();
        let deserialized: HookContext = serde_json::from_str(&json).unwrap();

        assert_eq!(ctx.hook, deserialized.hook);
        assert_eq!(ctx.source, deserialized.source);
    }
}

mod hook_result_tests {
    use super::*;

    #[test]
    fn test_hook_result_ok() {
        let result = HookResult::ok();
        assert!(result.is_continue());
        assert!(!result.is_abort());
        assert!(!result.is_modified());
        assert!(!result.is_skip());
    }

    #[test]
    fn test_hook_result_abort() {
        let result = HookResult::abort("Validation failed");
        assert!(!result.is_continue());
        assert!(result.is_abort());

        if let HookResult::Abort { reason } = result {
            assert_eq!(reason, "Validation failed");
        } else {
            panic!("Expected Abort result");
        }
    }

    #[test]
    fn test_hook_result_modified() {
        let data = HookData::Custom(serde_json::json!({"modified": true}));
        let result = HookResult::modified(data.clone());

        assert!(result.is_modified());
        assert!(result.get_modified_data().is_some());
    }

    #[test]
    fn test_hook_result_modified_with_message() {
        let data = HookData::None;
        let result = HookResult::modified_with_message(data, "Data was transformed");

        if let HookResult::Modified { message, .. } = result {
            assert_eq!(message, Some("Data was transformed".to_string()));
        } else {
            panic!("Expected Modified result");
        }
    }

    #[test]
    fn test_hook_result_skip() {
        let result = HookResult::skip();
        assert!(result.is_skip());
        assert!(!result.is_continue());
    }

    #[test]
    fn test_hook_result_default() {
        let result = HookResult::default();
        assert!(result.is_continue());
    }

    #[test]
    fn test_hook_result_get_modified_data() {
        let continue_result = HookResult::Continue;
        assert!(continue_result.get_modified_data().is_none());

        let abort_result = HookResult::abort("reason");
        assert!(abort_result.get_modified_data().is_none());

        let data = HookData::None;
        let modified_result = HookResult::modified(data);
        assert!(modified_result.get_modified_data().is_some());
    }

    #[test]
    fn test_hook_result_serialization() {
        let results = vec![
            HookResult::ok(),
            HookResult::abort("test"),
            HookResult::modified(HookData::None),
            HookResult::skip(),
        ];

        for result in results {
            let json = serde_json::to_string(&result).unwrap();
            let _: HookResult = serde_json::from_str(&json).unwrap();
        }
    }
}

mod hook_execution_tests {
    use super::*;

    #[test]
    fn test_hook_execution_new() {
        let execution = HookExecution::new(Hook::PreSessionStart, "test_handler");

        assert_eq!(execution.hook, Hook::PreSessionStart);
        assert_eq!(execution.handler_name, "test_handler");
        assert!(execution.completed_at.is_none());
        assert!(execution.result.is_none());
        assert!(execution.error.is_none());
        assert!(execution.duration_ms.is_none());
    }

    #[test]
    fn test_hook_execution_complete() {
        let execution = HookExecution::new(Hook::OnCostRecorded, "cost_handler");
        let completed = execution.complete(HookResult::ok());

        assert!(completed.completed_at.is_some());
        assert!(completed.result.is_some());
        assert!(completed.duration_ms.is_some());
        assert!(completed.is_successful());
    }

    #[test]
    fn test_hook_execution_fail() {
        let execution = HookExecution::new(Hook::OnError, "error_handler");
        let failed = execution.fail("Handler crashed");

        assert!(failed.completed_at.is_some());
        assert!(failed.error.is_some());
        assert_eq!(failed.error, Some("Handler crashed".to_string()));
        assert!(!failed.is_successful());
    }

    #[test]
    fn test_hook_execution_is_successful() {
        let success = HookExecution::new(Hook::OnSyncComplete, "sync").complete(HookResult::ok());
        assert!(success.is_successful());

        let failure = HookExecution::new(Hook::OnSyncComplete, "sync").fail("error");
        assert!(!failure.is_successful());

        let incomplete = HookExecution::new(Hook::OnSyncComplete, "sync");
        assert!(!incomplete.is_successful());
    }

    #[test]
    fn test_hook_execution_serialization() {
        let execution =
            HookExecution::new(Hook::OnMetricsCollected, "metrics").complete(HookResult::ok());

        let json = serde_json::to_string(&execution).unwrap();
        let deserialized: HookExecution = serde_json::from_str(&json).unwrap();

        assert_eq!(execution.hook, deserialized.hook);
        assert_eq!(execution.handler_name, deserialized.handler_name);
    }
}

mod hook_handler_info_tests {
    use super::*;

    #[test]
    fn test_hook_handler_info_new() {
        let info = HookHandlerInfo::new("my_handler", Hook::OnCostRecorded);

        assert_eq!(info.name, "my_handler");
        assert_eq!(info.hook, Hook::OnCostRecorded);
        assert_eq!(info.priority, 0);
        assert!(info.enabled);
        assert!(info.plugin_id.is_none());
        assert!(info.description.is_none());
    }

    #[test]
    fn test_hook_handler_info_with_priority() {
        let info =
            HookHandlerInfo::new("priority_handler", Hook::PreSessionStart).with_priority(100);

        assert_eq!(info.priority, 100);
    }

    #[test]
    fn test_hook_handler_info_with_description() {
        let info = HookHandlerInfo::new("described_handler", Hook::PostSessionEnd)
            .with_description("Handles session completion");

        assert_eq!(
            info.description,
            Some("Handles session completion".to_string())
        );
    }

    #[test]
    fn test_hook_handler_info_from_plugin() {
        let info = HookHandlerInfo::new("plugin_handler", Hook::OnError).from_plugin("my-plugin");

        assert_eq!(info.plugin_id, Some("my-plugin".to_string()));
    }

    #[test]
    fn test_hook_handler_info_builder_chain() {
        let info = HookHandlerInfo::new("full_handler", Hook::OnAgentConnect)
            .with_priority(50)
            .with_description("Handles agent connections")
            .from_plugin("agent-monitor");

        assert_eq!(info.name, "full_handler");
        assert_eq!(info.hook, Hook::OnAgentConnect);
        assert_eq!(info.priority, 50);
        assert_eq!(
            info.description,
            Some("Handles agent connections".to_string())
        );
        assert_eq!(info.plugin_id, Some("agent-monitor".to_string()));
    }

    #[test]
    fn test_hook_handler_info_serialization() {
        let info = HookHandlerInfo::new("serialize_test", Hook::OnSyncComplete)
            .with_priority(10)
            .from_plugin("test-plugin");

        let json = serde_json::to_string(&info).unwrap();
        let deserialized: HookHandlerInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(info.name, deserialized.name);
        assert_eq!(info.hook, deserialized.hook);
        assert_eq!(info.priority, deserialized.priority);
        assert_eq!(info.plugin_id, deserialized.plugin_id);
    }
}

mod hook_config_tests {
    use super::*;

    #[test]
    fn test_hook_config_default() {
        let config = HookConfig::default();

        assert_eq!(config.timeout_ms, 5000);
        assert!(config.enabled);
        assert_eq!(config.max_handlers, 100);
        assert!(!config.parallel_execution);
    }

    #[test]
    fn test_hook_config_new() {
        let config = HookConfig::new();
        assert!(config.enabled);
    }

    #[test]
    fn test_hook_config_with_timeout() {
        let config = HookConfig::new().with_timeout(10000);
        assert_eq!(config.timeout_ms, 10000);
    }

    #[test]
    fn test_hook_config_disabled() {
        let config = HookConfig::new().disabled();
        assert!(!config.enabled);
    }

    #[test]
    fn test_hook_config_with_max_handlers() {
        let config = HookConfig::new().with_max_handlers(50);
        assert_eq!(config.max_handlers, 50);
    }

    #[test]
    fn test_hook_config_parallel() {
        let config = HookConfig::new().parallel();
        assert!(config.parallel_execution);
    }

    #[test]
    fn test_hook_config_builder_chain() {
        let config = HookConfig::new()
            .with_timeout(15000)
            .with_max_handlers(200)
            .parallel();

        assert_eq!(config.timeout_ms, 15000);
        assert_eq!(config.max_handlers, 200);
        assert!(config.parallel_execution);
        assert!(config.enabled);
    }

    #[test]
    fn test_hook_config_serialization() {
        let config = HookConfig::new()
            .with_timeout(8000)
            .with_max_handlers(75)
            .parallel();

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: HookConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.timeout_ms, deserialized.timeout_ms);
        assert_eq!(config.max_handlers, deserialized.max_handlers);
        assert_eq!(config.parallel_execution, deserialized.parallel_execution);
    }
}
