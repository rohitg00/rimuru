#![allow(dead_code, unused_imports, unused_variables, unused_mut)]

use rimuru_core::sync::{
    ConflictResolution, ModelAggregator, ModelCapability, ModelSource, ProviderSyncStatus,
    RateLimitConfig, SyncError, SyncHistory, SyncHistoryEntry, SyncModuleConfig, SyncResult,
    SyncStatus,
};

mod sync_result_tests {
    use super::*;

    #[test]
    fn test_sync_result_new() {
        let result = SyncResult::new("anthropic");
        assert_eq!(result.provider, "anthropic");
        assert!(result.success);
        assert!(result.errors.is_empty());
        assert_eq!(result.models_updated, 0);
        assert_eq!(result.models_added, 0);
        assert_eq!(result.models_unchanged, 0);
    }

    #[test]
    fn test_sync_result_with_error() {
        let result =
            SyncResult::new("openai").with_error(SyncError::api_error("Connection timeout"));

        assert!(!result.success);
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].code, "API_ERROR");
        assert!(result.errors[0].recoverable);
    }

    #[test]
    fn test_sync_result_multiple_errors() {
        let result = SyncResult::new("google")
            .with_error(SyncError::api_error("Error 1"))
            .with_error(SyncError::parse_error("Error 2"));

        assert!(!result.success);
        assert_eq!(result.errors.len(), 2);
    }

    #[test]
    fn test_sync_result_total_models() {
        let mut result = SyncResult::new("test");
        result.models_updated = 5;
        result.models_added = 3;
        result.models_unchanged = 10;

        assert_eq!(result.total_models(), 18);
    }

    #[test]
    fn test_sync_result_serialization() {
        let result = SyncResult::new("anthropic");
        let json = serde_json::to_string(&result).unwrap();
        let deserialized: SyncResult = serde_json::from_str(&json).unwrap();

        assert_eq!(result.provider, deserialized.provider);
        assert_eq!(result.success, deserialized.success);
    }
}

mod sync_error_tests {
    use super::*;

    #[test]
    fn test_sync_error_api() {
        let error = SyncError::api_error("Connection refused");
        assert_eq!(error.code, "API_ERROR");
        assert_eq!(error.message, "Connection refused");
        assert!(error.recoverable);
    }

    #[test]
    fn test_sync_error_parse() {
        let error = SyncError::parse_error("Invalid JSON");
        assert_eq!(error.code, "PARSE_ERROR");
        assert_eq!(error.message, "Invalid JSON");
        assert!(!error.recoverable);
    }

    #[test]
    fn test_sync_error_rate_limit() {
        let error = SyncError::rate_limit("Too many requests");
        assert_eq!(error.code, "RATE_LIMIT");
        assert_eq!(error.message, "Too many requests");
        assert!(error.recoverable);
    }

    #[test]
    fn test_sync_error_auth() {
        let error = SyncError::auth_error("Invalid API key");
        assert_eq!(error.code, "AUTH_ERROR");
        assert_eq!(error.message, "Invalid API key");
        assert!(!error.recoverable);
    }

    #[test]
    fn test_sync_error_custom() {
        let error = SyncError::new("CUSTOM_ERROR", "Custom message", true);
        assert_eq!(error.code, "CUSTOM_ERROR");
        assert_eq!(error.message, "Custom message");
        assert!(error.recoverable);
    }

    #[test]
    fn test_sync_error_serialization() {
        let error = SyncError::api_error("Test error");
        let json = serde_json::to_string(&error).unwrap();
        let deserialized: SyncError = serde_json::from_str(&json).unwrap();

        assert_eq!(error.code, deserialized.code);
        assert_eq!(error.message, deserialized.message);
        assert_eq!(error.recoverable, deserialized.recoverable);
    }
}

mod sync_config_tests {
    use super::*;

    #[test]
    fn test_sync_config_default() {
        let config = SyncModuleConfig::default();

        assert!(config.enabled);
        assert_eq!(config.interval_secs, 6 * 60 * 60);
        assert_eq!(config.retry_max_attempts, 3);
        assert_eq!(config.retry_base_delay_secs, 60);
        assert_eq!(config.retry_max_delay_secs, 3600);
    }

    #[test]
    fn test_sync_provider_config_default() {
        let config = SyncModuleConfig::default();

        assert!(config.providers.anthropic);
        assert!(config.providers.openai);
        assert!(config.providers.google);
        assert!(config.providers.openrouter);
        assert!(config.providers.litellm);
    }

    #[test]
    fn test_rate_limit_config_default() {
        let config = RateLimitConfig::default();

        assert_eq!(config.anthropic_rpm, 60);
        assert_eq!(config.openai_rpm, 60);
        assert_eq!(config.google_rpm, 60);
        assert_eq!(config.openrouter_rpm, 100);
        assert_eq!(config.litellm_rpm, 100);
    }

    #[test]
    fn test_sync_config_serialization() {
        let config = SyncModuleConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: SyncModuleConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.enabled, deserialized.enabled);
        assert_eq!(config.interval_secs, deserialized.interval_secs);
    }

    #[test]
    fn test_sync_config_yaml_serialization() {
        let config = SyncModuleConfig::default();
        let yaml = serde_yaml::to_string(&config).unwrap();
        let deserialized: SyncModuleConfig = serde_yaml::from_str(&yaml).unwrap();

        assert_eq!(config.enabled, deserialized.enabled);
        assert_eq!(config.providers.anthropic, deserialized.providers.anthropic);
    }
}

mod sync_status_tests {
    use super::*;

    #[test]
    fn test_sync_status_default() {
        let status = SyncStatus::default();

        assert!(!status.is_running);
        assert!(status.last_full_sync.is_none());
        assert!(status.next_scheduled_sync.is_none());
        assert!(status.provider_status.is_empty());
    }

    #[test]
    fn test_provider_sync_status_new() {
        let status = ProviderSyncStatus::new("anthropic", true);

        assert_eq!(status.provider, "anthropic");
        assert!(status.enabled);
        assert!(status.last_success);
        assert_eq!(status.models_count, 0);
        assert_eq!(status.consecutive_failures, 0);
        assert!(status.last_error.is_none());
    }

    #[test]
    fn test_provider_sync_status_disabled() {
        let status = ProviderSyncStatus::new("litellm", false);
        assert!(!status.enabled);
    }

    #[test]
    fn test_sync_status_with_providers() {
        let mut status = SyncStatus::default();
        status.provider_status.insert(
            "anthropic".to_string(),
            ProviderSyncStatus::new("anthropic", true),
        );
        status.provider_status.insert(
            "openai".to_string(),
            ProviderSyncStatus::new("openai", true),
        );

        assert_eq!(status.provider_status.len(), 2);
        assert!(status.provider_status.contains_key("anthropic"));
        assert!(status.provider_status.contains_key("openai"));
    }
}

mod sync_history_tests {
    use super::*;

    #[test]
    fn test_sync_history_default() {
        let history = SyncHistory::default();
        assert!(history.entries.is_empty());
    }

    #[test]
    fn test_sync_history_add_entry() {
        let mut history = SyncHistory::default();

        history.add_entry(SyncHistoryEntry::success("anthropic", 10, 100));
        history.add_entry(SyncHistoryEntry::failure("openai", "timeout", 50));

        assert_eq!(history.entries.len(), 2);
    }

    #[test]
    fn test_sync_history_recent() {
        let mut history = SyncHistory::default();

        history.add_entry(SyncHistoryEntry::success("a", 1, 10));
        history.add_entry(SyncHistoryEntry::success("b", 2, 20));
        history.add_entry(SyncHistoryEntry::success("c", 3, 30));
        history.add_entry(SyncHistoryEntry::success("d", 4, 40));

        let recent = history.recent(2);
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].provider, "c");
        assert_eq!(recent[1].provider, "d");
    }

    #[test]
    fn test_sync_history_success_failure_count() {
        let mut history = SyncHistory::default();

        history.add_entry(SyncHistoryEntry::success("a", 1, 10));
        history.add_entry(SyncHistoryEntry::success("b", 2, 20));
        history.add_entry(SyncHistoryEntry::failure("c", "error", 30));

        assert_eq!(history.success_count(), 2);
        assert_eq!(history.failure_count(), 1);
    }

    #[test]
    fn test_sync_history_entry_success() {
        let entry = SyncHistoryEntry::success("anthropic", 15, 250);

        assert!(entry.success);
        assert_eq!(entry.provider, "anthropic");
        assert_eq!(entry.models_synced, 15);
        assert_eq!(entry.duration_ms, 250);
        assert!(entry.error_message.is_none());
    }

    #[test]
    fn test_sync_history_entry_failure() {
        let entry = SyncHistoryEntry::failure("openai", "Rate limited", 100);

        assert!(!entry.success);
        assert_eq!(entry.provider, "openai");
        assert_eq!(entry.models_synced, 0);
        assert_eq!(entry.duration_ms, 100);
        assert_eq!(entry.error_message, Some("Rate limited".to_string()));
    }

    #[test]
    fn test_sync_history_max_entries() {
        let mut history = SyncHistory::default();

        for i in 0..1005 {
            history.add_entry(SyncHistoryEntry::success(&format!("p{}", i), i, 10));
        }

        assert_eq!(history.entries.len(), 1000);
        assert_eq!(history.entries[0].provider, "p5");
    }
}

mod model_capability_tests {
    use super::*;

    #[test]
    fn test_model_capabilities() {
        let capabilities = vec![
            ModelCapability::TextGeneration,
            ModelCapability::Vision,
            ModelCapability::FunctionCalling,
            ModelCapability::JsonMode,
            ModelCapability::Embedding,
            ModelCapability::CodeGeneration,
            ModelCapability::MultiModal,
            ModelCapability::Streaming,
            ModelCapability::FineTuning,
        ];

        for cap in &capabilities {
            let json = serde_json::to_string(cap).unwrap();
            let deserialized: ModelCapability = serde_json::from_str(&json).unwrap();
            assert_eq!(*cap, deserialized);
        }
    }
}

mod model_source_tests {
    use super::*;

    #[test]
    fn test_model_source_priority() {
        assert!(ModelSource::OfficialApi.priority() < ModelSource::OfficialDocs.priority());
        assert!(ModelSource::OfficialDocs.priority() < ModelSource::OpenRouter.priority());
        assert!(ModelSource::OpenRouter.priority() < ModelSource::LiteLLM.priority());
        assert!(ModelSource::LiteLLM.priority() < ModelSource::Manual.priority());
    }

    #[test]
    fn test_model_source_serialization() {
        let sources = vec![
            ModelSource::OfficialApi,
            ModelSource::OfficialDocs,
            ModelSource::OpenRouter,
            ModelSource::LiteLLM,
            ModelSource::Manual,
        ];

        for source in sources {
            let json = serde_json::to_string(&source).unwrap();
            let deserialized: ModelSource = serde_json::from_str(&json).unwrap();
            assert_eq!(source, deserialized);
        }
    }
}

mod model_aggregator_tests {
    use super::*;
    use rimuru_core::models::ModelInfo;

    fn create_test_model(
        provider: &str,
        name: &str,
        input_price: f64,
        output_price: f64,
    ) -> ModelInfo {
        ModelInfo::new(
            provider.to_string(),
            name.to_string(),
            input_price,
            output_price,
            128000,
        )
    }

    #[test]
    fn test_conflict_resolution_variants() {
        assert_eq!(
            ConflictResolution::default(),
            ConflictResolution::OfficialFirst
        );

        let resolutions = vec![
            ConflictResolution::OfficialFirst,
            ConflictResolution::MostRecent,
            ConflictResolution::LowestPrice,
            ConflictResolution::HighestContextWindow,
        ];

        for resolution in &resolutions {
            let aggregator = ModelAggregator::with_resolution(*resolution);
            assert!(aggregator.merge_models(vec![]).is_empty());
        }
    }

    #[test]
    fn test_aggregator_new() {
        let aggregator = ModelAggregator::new();
        assert!(aggregator.merge_models(vec![]).is_empty());
    }

    #[test]
    fn test_aggregator_merge_models() {
        let aggregator = ModelAggregator::new();

        let official_models = vec![create_test_model(
            "anthropic",
            "claude-3-opus",
            0.015,
            0.075,
        )];

        let community_models = vec![create_test_model(
            "anthropic",
            "claude-3-opus",
            0.020,
            0.080,
        )];

        let merged = aggregator.merge_models(vec![
            (ModelSource::OfficialApi, official_models),
            (ModelSource::OpenRouter, community_models),
        ]);

        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].input_price_per_1k, 0.015);
    }

    #[test]
    fn test_aggregator_deduplicate_models() {
        let aggregator = ModelAggregator::new();

        let mut model1 = create_test_model("anthropic", "claude-3-opus", 0.015, 0.075);
        let mut model2 = create_test_model("anthropic", "claude-3-opus", 0.016, 0.076);

        model2.last_synced = model1.last_synced + chrono::Duration::hours(1);

        let deduped = aggregator.deduplicate_models(vec![model1, model2]);

        assert_eq!(deduped.len(), 1);
        assert_eq!(deduped[0].input_price_per_1k, 0.016);
    }

    #[test]
    fn test_normalize_provider_name() {
        assert_eq!(
            ModelAggregator::normalize_provider_name("Anthropic"),
            "anthropic"
        );
        assert_eq!(ModelAggregator::normalize_provider_name("OPENAI"), "openai");
        assert_eq!(ModelAggregator::normalize_provider_name("Google"), "google");
        assert_eq!(ModelAggregator::normalize_provider_name("gemini"), "google");
        assert_eq!(
            ModelAggregator::normalize_provider_name("Claude"),
            "anthropic"
        );
        assert_eq!(ModelAggregator::normalize_provider_name("GPT"), "openai");
    }

    #[test]
    fn test_normalize_model_name() {
        assert_eq!(
            ModelAggregator::normalize_model_name("Claude 3 Opus"),
            "claude-3-opus"
        );
        assert_eq!(
            ModelAggregator::normalize_model_name("GPT_4_Turbo"),
            "gpt-4-turbo"
        );
        assert_eq!(ModelAggregator::normalize_model_name("gpt-4o"), "gpt-4o");
    }

    #[test]
    fn test_validate_pricing_valid() {
        let model = create_test_model("anthropic", "claude-3-opus", 0.015, 0.075);
        assert!(ModelAggregator::validate_pricing(&model));
    }

    #[test]
    fn test_validate_pricing_negative_input() {
        let model = create_test_model("test", "model", -0.01, 0.03);
        assert!(!ModelAggregator::validate_pricing(&model));
    }

    #[test]
    fn test_validate_pricing_negative_output() {
        let model = create_test_model("test", "model", 0.01, -0.03);
        assert!(!ModelAggregator::validate_pricing(&model));
    }

    #[test]
    fn test_validate_pricing_zero_context() {
        let mut model = create_test_model("test", "model", 0.01, 0.03);
        model.context_window = 0;
        assert!(!ModelAggregator::validate_pricing(&model));
    }

    #[test]
    fn test_filter_valid_models() {
        let aggregator = ModelAggregator::new();

        let valid_model = create_test_model("anthropic", "claude", 0.01, 0.03);
        let invalid_model = create_test_model("test", "bad", -0.01, 0.03);

        let filtered = aggregator.filter_valid_models(vec![valid_model.clone(), invalid_model]);

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].provider, "anthropic");
    }

    #[test]
    fn test_conflict_resolution_most_recent() {
        let aggregator = ModelAggregator::with_resolution(ConflictResolution::MostRecent);

        let older = create_test_model("anthropic", "claude-3-opus", 0.015, 0.075);
        let mut newer = create_test_model("anthropic", "claude-3-opus", 0.020, 0.080);

        newer.last_synced = older.last_synced + chrono::Duration::hours(1);

        let merged = aggregator.merge_models(vec![
            (ModelSource::OfficialApi, vec![older]),
            (ModelSource::OpenRouter, vec![newer]),
        ]);

        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].input_price_per_1k, 0.020);
    }

    #[test]
    fn test_conflict_resolution_lowest_price() {
        let aggregator = ModelAggregator::with_resolution(ConflictResolution::LowestPrice);

        let expensive = create_test_model("anthropic", "claude-3-opus", 0.020, 0.080);
        let cheaper = create_test_model("anthropic", "claude-3-opus", 0.015, 0.075);

        let merged = aggregator.merge_models(vec![
            (ModelSource::OfficialApi, vec![expensive]),
            (ModelSource::OpenRouter, vec![cheaper]),
        ]);

        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].input_price_per_1k, 0.015);
    }

    #[test]
    fn test_conflict_resolution_highest_context() {
        let aggregator = ModelAggregator::with_resolution(ConflictResolution::HighestContextWindow);

        let smaller = create_test_model("anthropic", "claude-3-opus", 0.015, 0.075);
        let mut larger = create_test_model("anthropic", "claude-3-opus", 0.015, 0.075);
        larger.context_window = 200000;

        let merged = aggregator.merge_models(vec![
            (ModelSource::OfficialApi, vec![smaller]),
            (ModelSource::OpenRouter, vec![larger]),
        ]);

        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].context_window, 200000);
    }
}

mod mock_sync_provider_tests {
    use async_trait::async_trait;
    use rimuru_core::error::RimuruResult;
    use rimuru_core::models::ModelInfo;
    use rimuru_core::sync::ModelSyncProvider;

    struct MockSyncProvider {
        name: String,
        models: Vec<ModelInfo>,
        should_fail: bool,
    }

    impl MockSyncProvider {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                models: Vec::new(),
                should_fail: false,
            }
        }

        fn with_models(mut self, models: Vec<ModelInfo>) -> Self {
            self.models = models;
            self
        }

        fn failing(mut self) -> Self {
            self.should_fail = true;
            self
        }
    }

    #[async_trait]
    impl ModelSyncProvider for MockSyncProvider {
        fn provider_name(&self) -> &str {
            &self.name
        }

        async fn fetch_models(&self) -> RimuruResult<Vec<ModelInfo>> {
            if self.should_fail {
                Err(rimuru_core::error::RimuruError::ApiRequestFailed(
                    "Mock failure".to_string(),
                ))
            } else {
                Ok(self.models.clone())
            }
        }

        fn is_official_source(&self) -> bool {
            true
        }

        fn priority(&self) -> u8 {
            10
        }
    }

    #[tokio::test]
    async fn test_mock_provider_fetch_models() {
        let model = ModelInfo::new(
            "test".to_string(),
            "test-model".to_string(),
            0.01,
            0.02,
            4096,
        );

        let provider = MockSyncProvider::new("test-provider").with_models(vec![model]);

        let models = provider.fetch_models().await.unwrap();
        assert_eq!(models.len(), 1);
        assert_eq!(models[0].model_name, "test-model");
    }

    #[tokio::test]
    async fn test_mock_provider_failure() {
        let provider = MockSyncProvider::new("failing").failing();

        let result = provider.fetch_models().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mock_provider_health_check() {
        let provider = MockSyncProvider::new("test");
        assert!(provider.health_check().await.unwrap());
    }

    #[test]
    fn test_mock_provider_name() {
        let provider = MockSyncProvider::new("my-provider");
        assert_eq!(provider.provider_name(), "my-provider");
    }

    #[test]
    fn test_mock_provider_is_official_source() {
        let provider = MockSyncProvider::new("test");
        assert!(provider.is_official_source());
    }

    #[test]
    fn test_mock_provider_priority() {
        let provider = MockSyncProvider::new("test");
        assert_eq!(provider.priority(), 10);
    }

    #[test]
    fn test_mock_provider_supports_streaming() {
        let provider = MockSyncProvider::new("test");
        assert!(!provider.supports_streaming());
    }
}
