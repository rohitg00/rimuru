#![allow(
    dead_code,
    unused_imports,
    unused_variables,
    unused_mut,
    clippy::field_reassign_with_default
)]

use async_trait::async_trait;
use chrono::{Duration, Utc};
use rimuru_core::models::ModelInfo;
use rimuru_core::sync::{
    ExtendedModelInfo, ModelCapability, ModelSource, ProviderSyncStatus, SyncError, SyncHistory,
    SyncHistoryEntry, SyncModuleConfig as SyncConfig, SyncResult,
};
use rimuru_core::{ModelSyncProvider, RimuruResult};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

struct MockModelSyncProvider {
    name: String,
    models: Arc<RwLock<Vec<ModelInfo>>>,
    should_fail: Arc<RwLock<bool>>,
    sync_count: Arc<RwLock<u32>>,
}

impl MockModelSyncProvider {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            models: Arc::new(RwLock::new(Vec::new())),
            should_fail: Arc::new(RwLock::new(false)),
            sync_count: Arc::new(RwLock::new(0)),
        }
    }

    fn add_model(&self, model: ModelInfo) {
        self.models.write().unwrap().push(model);
    }

    fn set_should_fail(&self, fail: bool) {
        *self.should_fail.write().unwrap() = fail;
    }

    fn get_sync_count(&self) -> u32 {
        *self.sync_count.read().unwrap()
    }

    fn create_test_model(name: &str, provider: &str) -> ModelInfo {
        ModelInfo::new(provider.to_string(), name.to_string(), 0.01, 0.03, 128000)
    }

    fn get_status(&self) -> ProviderSyncStatus {
        ProviderSyncStatus::new(&self.name, !*self.should_fail.read().unwrap())
    }
}

#[async_trait]
impl ModelSyncProvider for MockModelSyncProvider {
    fn provider_name(&self) -> &str {
        &self.name
    }

    async fn fetch_models(&self) -> RimuruResult<Vec<ModelInfo>> {
        *self.sync_count.write().unwrap() += 1;

        if *self.should_fail.read().unwrap() {
            return Err(rimuru_core::RimuruError::ApiRequestFailed(
                "Mock provider sync failure".to_string(),
            ));
        }

        Ok(self.models.read().unwrap().clone())
    }

    fn supports_streaming(&self) -> bool {
        true
    }
}

struct MockSyncHistory {
    history: Arc<RwLock<SyncHistory>>,
}

impl MockSyncHistory {
    fn new() -> Self {
        Self {
            history: Arc::new(RwLock::new(SyncHistory::default())),
        }
    }

    fn add_entry(&self, entry: SyncHistoryEntry) {
        self.history.write().unwrap().add_entry(entry);
    }

    fn get_entries(&self) -> Vec<SyncHistoryEntry> {
        self.history.read().unwrap().entries.clone()
    }

    fn clear(&self) {
        *self.history.write().unwrap() = SyncHistory::default();
    }

    fn success_count(&self) -> usize {
        self.history.read().unwrap().success_count()
    }

    fn failure_count(&self) -> usize {
        self.history.read().unwrap().failure_count()
    }
}

mod single_provider_sync {
    use super::*;

    #[tokio::test]
    async fn test_sync_empty_provider() {
        let provider = MockModelSyncProvider::new("empty-provider");

        let models = provider.fetch_models().await.unwrap();
        assert!(models.is_empty());
        assert_eq!(provider.get_sync_count(), 1);
    }

    #[tokio::test]
    async fn test_sync_provider_with_models() {
        let provider = MockModelSyncProvider::new("anthropic");

        provider.add_model(MockModelSyncProvider::create_test_model(
            "claude-3-opus",
            "anthropic",
        ));
        provider.add_model(MockModelSyncProvider::create_test_model(
            "claude-3-sonnet",
            "anthropic",
        ));
        provider.add_model(MockModelSyncProvider::create_test_model(
            "claude-3-haiku",
            "anthropic",
        ));

        let models = provider.fetch_models().await.unwrap();
        assert_eq!(models.len(), 3);

        let opus = models.iter().find(|m| m.model_name == "claude-3-opus");
        assert!(opus.is_some());
        assert_eq!(opus.unwrap().provider, "anthropic");
    }

    #[tokio::test]
    async fn test_sync_provider_failure() {
        let provider = MockModelSyncProvider::new("failing-provider");
        provider.set_should_fail(true);

        let result = provider.fetch_models().await;
        assert!(result.is_err());

        let status = provider.get_status();
        assert!(!status.enabled);
    }

    #[tokio::test]
    async fn test_provider_status_ready() {
        let provider = MockModelSyncProvider::new("ready-provider");

        let status = provider.get_status();
        assert!(status.enabled);
    }

    #[tokio::test]
    async fn test_provider_supports_streaming() {
        let provider = MockModelSyncProvider::new("streaming-provider");
        assert!(provider.supports_streaming());
    }
}

mod multi_provider_sync {
    use super::*;

    #[tokio::test]
    async fn test_sync_multiple_providers_sequentially() {
        let anthropic = MockModelSyncProvider::new("anthropic");
        anthropic.add_model(MockModelSyncProvider::create_test_model(
            "claude-3-opus",
            "anthropic",
        ));
        anthropic.add_model(MockModelSyncProvider::create_test_model(
            "claude-3-sonnet",
            "anthropic",
        ));

        let openai = MockModelSyncProvider::new("openai");
        openai.add_model(MockModelSyncProvider::create_test_model("gpt-4", "openai"));
        openai.add_model(MockModelSyncProvider::create_test_model(
            "gpt-4-turbo",
            "openai",
        ));

        let google = MockModelSyncProvider::new("google");
        google.add_model(MockModelSyncProvider::create_test_model(
            "gemini-pro",
            "google",
        ));

        let providers: Vec<&dyn ModelSyncProvider> = vec![&anthropic, &openai, &google];

        let mut all_models = Vec::new();
        for provider in providers {
            let models = provider.fetch_models().await.unwrap();
            all_models.extend(models);
        }

        assert_eq!(all_models.len(), 5);

        let anthropic_models: Vec<_> = all_models
            .iter()
            .filter(|m| m.provider == "anthropic")
            .collect();
        assert_eq!(anthropic_models.len(), 2);

        let openai_models: Vec<_> = all_models
            .iter()
            .filter(|m| m.provider == "openai")
            .collect();
        assert_eq!(openai_models.len(), 2);

        let google_models: Vec<_> = all_models
            .iter()
            .filter(|m| m.provider == "google")
            .collect();
        assert_eq!(google_models.len(), 1);
    }

    #[tokio::test]
    async fn test_sync_with_partial_failure() {
        let working = MockModelSyncProvider::new("working");
        working.add_model(MockModelSyncProvider::create_test_model(
            "model-a", "working",
        ));

        let failing = MockModelSyncProvider::new("failing");
        failing.set_should_fail(true);

        let another_working = MockModelSyncProvider::new("another");
        another_working.add_model(MockModelSyncProvider::create_test_model(
            "model-b", "another",
        ));

        let providers: Vec<&MockModelSyncProvider> = vec![&working, &failing, &another_working];

        let mut successful_models = Vec::new();
        let mut failed_providers = Vec::new();

        for provider in providers {
            match provider.fetch_models().await {
                Ok(models) => successful_models.extend(models),
                Err(_) => failed_providers.push(provider.provider_name().to_string()),
            }
        }

        assert_eq!(successful_models.len(), 2);
        assert_eq!(failed_providers.len(), 1);
        assert!(failed_providers.contains(&"failing".to_string()));
    }

    #[tokio::test]
    async fn test_sync_count_tracking() {
        let provider = MockModelSyncProvider::new("counted");
        provider.add_model(MockModelSyncProvider::create_test_model("model", "counted"));

        assert_eq!(provider.get_sync_count(), 0);

        provider.fetch_models().await.unwrap();
        assert_eq!(provider.get_sync_count(), 1);

        provider.fetch_models().await.unwrap();
        provider.fetch_models().await.unwrap();
        assert_eq!(provider.get_sync_count(), 3);
    }
}

mod model_aggregation {
    use super::*;

    #[tokio::test]
    async fn test_aggregate_models_from_providers() {
        let anthropic = MockModelSyncProvider::new("anthropic");
        anthropic.add_model(ModelInfo::new(
            "anthropic".to_string(),
            "claude-3-opus".to_string(),
            0.015,
            0.075,
            200000,
        ));

        let openai = MockModelSyncProvider::new("openai");
        openai.add_model(ModelInfo::new(
            "openai".to_string(),
            "gpt-4-turbo".to_string(),
            0.01,
            0.03,
            128000,
        ));

        let models1 = anthropic.fetch_models().await.unwrap();
        let models2 = openai.fetch_models().await.unwrap();

        let mut all_models: HashMap<String, ModelInfo> = HashMap::new();
        for model in models1.into_iter().chain(models2.into_iter()) {
            all_models.insert(format!("{}:{}", model.provider, model.model_name), model);
        }

        assert_eq!(all_models.len(), 2);
        assert!(all_models.contains_key("anthropic:claude-3-opus"));
        assert!(all_models.contains_key("openai:gpt-4-turbo"));
    }

    #[test]
    fn test_model_conflict_resolution_prefer_official() {
        let official = ExtendedModelInfo {
            model_name: "claude-3-opus".to_string(),
            provider: "anthropic".to_string(),
            model_id: "anthropic/claude-3-opus".to_string(),
            input_price_per_1k: 0.015,
            output_price_per_1k: 0.075,
            context_window: 200000,
            max_output_tokens: Some(4096),
            capabilities: vec![],
            source: ModelSource::OfficialApi,
            last_updated: Utc::now() - Duration::hours(1),
            deprecation_date: None,
        };

        let community = ExtendedModelInfo {
            model_name: "claude-3-opus".to_string(),
            provider: "anthropic".to_string(),
            model_id: "anthropic/claude-3-opus".to_string(),
            input_price_per_1k: 0.02,
            output_price_per_1k: 0.08,
            context_window: 200000,
            max_output_tokens: Some(4096),
            capabilities: vec![],
            source: ModelSource::OpenRouter,
            last_updated: Utc::now(),
            deprecation_date: None,
        };

        fn resolve_conflict(a: &ExtendedModelInfo, b: &ExtendedModelInfo) -> ExtendedModelInfo {
            if a.source.priority() <= b.source.priority() {
                a.clone()
            } else {
                b.clone()
            }
        }

        let resolved = resolve_conflict(&official, &community);
        assert!(matches!(resolved.source, ModelSource::OfficialApi));
        assert!((resolved.input_price_per_1k - 0.015).abs() < 0.001);
    }

    #[test]
    fn test_model_conflict_resolution_prefer_newer() {
        let older = ExtendedModelInfo {
            model_name: "model".to_string(),
            provider: "test".to_string(),
            model_id: "test/model".to_string(),
            input_price_per_1k: 0.01,
            output_price_per_1k: 0.03,
            context_window: 128000,
            max_output_tokens: Some(4096),
            capabilities: vec![],
            source: ModelSource::OpenRouter,
            last_updated: Utc::now() - Duration::days(7),
            deprecation_date: None,
        };

        let newer = ExtendedModelInfo {
            model_name: "model".to_string(),
            provider: "test".to_string(),
            model_id: "test/model".to_string(),
            input_price_per_1k: 0.015,
            output_price_per_1k: 0.045,
            context_window: 128000,
            max_output_tokens: Some(4096),
            capabilities: vec![],
            source: ModelSource::OpenRouter,
            last_updated: Utc::now(),
            deprecation_date: None,
        };

        fn resolve_conflict_by_date(
            a: &ExtendedModelInfo,
            b: &ExtendedModelInfo,
        ) -> ExtendedModelInfo {
            if a.last_updated > b.last_updated {
                a.clone()
            } else {
                b.clone()
            }
        }

        let resolved = resolve_conflict_by_date(&older, &newer);
        assert!((resolved.input_price_per_1k - 0.015).abs() < 0.001);
    }
}

mod sync_history {
    use super::*;

    #[tokio::test]
    async fn test_record_sync_history() {
        let history = MockSyncHistory::new();

        let entry = SyncHistoryEntry::success("anthropic", 5, 1500);
        history.add_entry(entry);

        let entries = history.get_entries();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].provider, "anthropic");
        assert_eq!(entries[0].models_synced, 5);
    }

    #[tokio::test]
    async fn test_record_failed_sync() {
        let history = MockSyncHistory::new();

        let entry = SyncHistoryEntry::failure("openai", "Connection timeout", 500);
        history.add_entry(entry);

        let entries = history.get_entries();
        assert_eq!(entries.len(), 1);
        assert!(!entries[0].success);
        assert!(entries[0].error_message.is_some());
    }

    #[tokio::test]
    async fn test_sync_history_multiple_entries() {
        let history = MockSyncHistory::new();

        for i in 0..5 {
            let entry = if i % 2 == 0 {
                SyncHistoryEntry::success(&format!("provider-{}", i), 10, 100)
            } else {
                SyncHistoryEntry::failure(&format!("provider-{}", i), "Error", 50)
            };
            history.add_entry(entry);
        }

        let entries = history.get_entries();
        assert_eq!(entries.len(), 5);
        assert_eq!(history.success_count(), 3);
        assert_eq!(history.failure_count(), 2);
    }

    #[tokio::test]
    async fn test_clear_sync_history() {
        let history = MockSyncHistory::new();

        history.add_entry(SyncHistoryEntry::success("test", 5, 100));
        assert_eq!(history.get_entries().len(), 1);

        history.clear();
        assert!(history.get_entries().is_empty());
    }
}

mod http_sync_tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_http_sync_endpoint() {
        let mock_server = MockServer::start().await;

        let response_body = r#"{
            "models": [
                {
                    "name": "gpt-4",
                    "pricing": {
                        "input": 0.03,
                        "output": 0.06
                    },
                    "context_length": 128000
                },
                {
                    "name": "gpt-4-turbo",
                    "pricing": {
                        "input": 0.01,
                        "output": 0.03
                    },
                    "context_length": 128000
                }
            ]
        }"#;

        Mock::given(method("GET"))
            .and(path("/v1/models"))
            .respond_with(ResponseTemplate::new(200).set_body_string(response_body))
            .mount(&mock_server)
            .await;

        let client = reqwest::Client::new();
        let response = client
            .get(format!("{}/v1/models", mock_server.uri()))
            .send()
            .await
            .unwrap();

        assert!(response.status().is_success());

        let body: serde_json::Value = response.json().await.unwrap();
        let models = body["models"].as_array().unwrap();
        assert_eq!(models.len(), 2);
    }

    #[tokio::test]
    async fn test_mock_http_sync_failure() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v1/models"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
            .mount(&mock_server)
            .await;

        let client = reqwest::Client::new();
        let response = client
            .get(format!("{}/v1/models", mock_server.uri()))
            .send()
            .await
            .unwrap();

        assert!(!response.status().is_success());
        assert_eq!(response.status().as_u16(), 500);
    }

    #[tokio::test]
    async fn test_mock_http_rate_limiting() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/v1/models"))
            .respond_with(
                ResponseTemplate::new(429)
                    .set_body_string("Too Many Requests")
                    .insert_header("Retry-After", "60"),
            )
            .mount(&mock_server)
            .await;

        let client = reqwest::Client::new();
        let response = client
            .get(format!("{}/v1/models", mock_server.uri()))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status().as_u16(), 429);

        let retry_after = response.headers().get("Retry-After");
        assert!(retry_after.is_some());
    }
}

mod model_capabilities {
    use super::*;

    #[test]
    fn test_model_capability_variants() {
        let capabilities = [
            ModelCapability::TextGeneration,
            ModelCapability::CodeGeneration,
            ModelCapability::Vision,
            ModelCapability::FunctionCalling,
        ];

        assert_eq!(capabilities.len(), 4);
    }

    #[test]
    fn test_model_with_multiple_capabilities() {
        let model = ExtendedModelInfo {
            model_name: "claude-3-opus".to_string(),
            provider: "anthropic".to_string(),
            model_id: "anthropic/claude-3-opus".to_string(),
            input_price_per_1k: 0.015,
            output_price_per_1k: 0.075,
            context_window: 200000,
            max_output_tokens: Some(4096),
            capabilities: vec![
                ModelCapability::TextGeneration,
                ModelCapability::CodeGeneration,
                ModelCapability::Vision,
                ModelCapability::FunctionCalling,
            ],
            source: ModelSource::OfficialApi,
            last_updated: Utc::now(),
            deprecation_date: None,
        };

        assert_eq!(model.capabilities.len(), 4);
        assert!(model
            .capabilities
            .contains(&ModelCapability::TextGeneration));
        assert!(model.capabilities.contains(&ModelCapability::Vision));
    }

    #[test]
    fn test_model_source_priority() {
        assert!(ModelSource::OfficialApi.priority() < ModelSource::OfficialDocs.priority());
        assert!(ModelSource::OfficialDocs.priority() < ModelSource::OpenRouter.priority());
        assert!(ModelSource::OpenRouter.priority() < ModelSource::LiteLLM.priority());
        assert!(ModelSource::LiteLLM.priority() < ModelSource::Manual.priority());
    }

    #[test]
    fn test_deprecated_model() {
        let model = ExtendedModelInfo {
            model_name: "gpt-3.5-turbo-0301".to_string(),
            provider: "openai".to_string(),
            model_id: "openai/gpt-3.5-turbo-0301".to_string(),
            input_price_per_1k: 0.0015,
            output_price_per_1k: 0.002,
            context_window: 4096,
            max_output_tokens: Some(4096),
            capabilities: vec![ModelCapability::TextGeneration],
            source: ModelSource::OfficialApi,
            last_updated: Utc::now(),
            deprecation_date: Some(Utc::now() + Duration::days(30)),
        };

        assert!(model.deprecation_date.is_some());
    }
}

mod sync_config {
    use super::*;

    #[test]
    fn test_sync_config_defaults() {
        let config = SyncConfig::default();

        assert!(config.enabled);
        assert!(config.providers.anthropic);
        assert!(config.providers.openai);
        assert!(config.providers.google);
    }

    #[test]
    fn test_sync_config_modification() {
        let mut config = SyncConfig::default();
        config.enabled = false;

        assert!(!config.enabled);
    }
}

mod sync_result_handling {
    use super::*;

    #[test]
    fn test_sync_result_success() {
        let result = SyncResult::new("anthropic");

        assert!(result.success);
        assert!(result.errors.is_empty());
        assert_eq!(result.total_models(), 0);
    }

    #[test]
    fn test_sync_result_with_error() {
        let result =
            SyncResult::new("openai").with_error(SyncError::api_error("Connection refused"));

        assert!(!result.success);
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0].code, "API_ERROR");
    }

    #[test]
    fn test_provider_sync_status() {
        let status = ProviderSyncStatus::new("anthropic", true);

        assert_eq!(status.provider, "anthropic");
        assert!(status.enabled);
        assert!(status.last_success);
        assert_eq!(status.consecutive_failures, 0);
    }

    #[test]
    fn test_sync_error_variants() {
        let api_err = SyncError::api_error("API down");
        assert_eq!(api_err.code, "API_ERROR");
        assert!(api_err.recoverable);

        let parse_err = SyncError::parse_error("Invalid JSON");
        assert_eq!(parse_err.code, "PARSE_ERROR");
        assert!(!parse_err.recoverable);

        let rate_err = SyncError::rate_limit("Too many requests");
        assert_eq!(rate_err.code, "RATE_LIMIT");
        assert!(rate_err.recoverable);

        let auth_err = SyncError::auth_error("Invalid API key");
        assert_eq!(auth_err.code, "AUTH_ERROR");
        assert!(!auth_err.recoverable);
    }
}

mod sync_history_operations {
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
        assert_eq!(history.success_count(), 1);
        assert_eq!(history.failure_count(), 1);
    }

    #[test]
    fn test_sync_history_recent() {
        let mut history = SyncHistory::default();
        for i in 0..10 {
            history.add_entry(SyncHistoryEntry::success(
                &format!("provider-{}", i),
                i,
                100,
            ));
        }

        let recent = history.recent(3);
        assert_eq!(recent.len(), 3);
    }
}
