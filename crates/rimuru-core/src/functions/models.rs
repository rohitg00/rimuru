use chrono::Utc;
use iii_sdk::III;
use serde_json::{json, Value};

use crate::models::{ModelInfo, ModelProvider, ModelSyncStatus};
use crate::state::StateKV;

pub fn register(iii: &III, kv: &StateKV) {
    register_list(iii, kv);
    register_sync(iii, kv);
    register_get(iii, kv);
}

fn hardcoded_models() -> Vec<ModelInfo> {
    let now = Utc::now();
    vec![
        ModelInfo {
            id: "claude-opus-4-6".into(),
            name: "Claude Opus 4.6".into(),
            provider: ModelProvider::Anthropic,
            input_price_per_million: 15.0,
            output_price_per_million: 75.0,
            cache_read_price_per_million: Some(1.5),
            cache_write_price_per_million: Some(18.75),
            context_window: 200_000,
            max_output_tokens: Some(32_000),
            supports_vision: true,
            supports_tools: true,
            last_synced: now,
        },
        ModelInfo {
            id: "claude-sonnet-4-6".into(),
            name: "Claude Sonnet 4.6".into(),
            provider: ModelProvider::Anthropic,
            input_price_per_million: 3.0,
            output_price_per_million: 15.0,
            cache_read_price_per_million: Some(0.3),
            cache_write_price_per_million: Some(3.75),
            context_window: 200_000,
            max_output_tokens: Some(64_000),
            supports_vision: true,
            supports_tools: true,
            last_synced: now,
        },
        ModelInfo {
            id: "claude-haiku-3-5".into(),
            name: "Claude Haiku 3.5".into(),
            provider: ModelProvider::Anthropic,
            input_price_per_million: 0.8,
            output_price_per_million: 4.0,
            cache_read_price_per_million: Some(0.08),
            cache_write_price_per_million: Some(1.0),
            context_window: 200_000,
            max_output_tokens: Some(8_192),
            supports_vision: true,
            supports_tools: true,
            last_synced: now,
        },
        ModelInfo {
            id: "gpt-4o".into(),
            name: "GPT-4o".into(),
            provider: ModelProvider::OpenAI,
            input_price_per_million: 2.5,
            output_price_per_million: 10.0,
            cache_read_price_per_million: Some(1.25),
            cache_write_price_per_million: None,
            context_window: 128_000,
            max_output_tokens: Some(16_384),
            supports_vision: true,
            supports_tools: true,
            last_synced: now,
        },
        ModelInfo {
            id: "gpt-4o-mini".into(),
            name: "GPT-4o Mini".into(),
            provider: ModelProvider::OpenAI,
            input_price_per_million: 0.15,
            output_price_per_million: 0.6,
            cache_read_price_per_million: Some(0.075),
            cache_write_price_per_million: None,
            context_window: 128_000,
            max_output_tokens: Some(16_384),
            supports_vision: true,
            supports_tools: true,
            last_synced: now,
        },
        ModelInfo {
            id: "o3".into(),
            name: "o3".into(),
            provider: ModelProvider::OpenAI,
            input_price_per_million: 10.0,
            output_price_per_million: 40.0,
            cache_read_price_per_million: Some(2.5),
            cache_write_price_per_million: None,
            context_window: 200_000,
            max_output_tokens: Some(100_000),
            supports_vision: true,
            supports_tools: true,
            last_synced: now,
        },
        ModelInfo {
            id: "gemini-2.5-pro".into(),
            name: "Gemini 2.5 Pro".into(),
            provider: ModelProvider::Google,
            input_price_per_million: 1.25,
            output_price_per_million: 10.0,
            cache_read_price_per_million: Some(0.315),
            cache_write_price_per_million: Some(4.5),
            context_window: 1_000_000,
            max_output_tokens: Some(65_536),
            supports_vision: true,
            supports_tools: true,
            last_synced: now,
        },
        ModelInfo {
            id: "gemini-2.5-flash".into(),
            name: "Gemini 2.5 Flash".into(),
            provider: ModelProvider::Google,
            input_price_per_million: 0.15,
            output_price_per_million: 0.6,
            cache_read_price_per_million: Some(0.0375),
            cache_write_price_per_million: Some(1.0),
            context_window: 1_000_000,
            max_output_tokens: Some(65_536),
            supports_vision: true,
            supports_tools: true,
            last_synced: now,
        },
    ]
}

fn register_list(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function("rimuru.models.list", move |input: Value| {
        let kv = kv.clone();
        async move {
            let stored: Vec<ModelInfo> = kv
                .list("model_info")
                .await
                .map_err(|e| iii_sdk::IIIError::Handler(e.to_string()))?;

            let models = if stored.is_empty() {
                hardcoded_models()
            } else {
                stored
            };

            let provider_filter = input
                .get("provider")
                .and_then(|v| v.as_str())
                .and_then(|s| serde_json::from_value::<ModelProvider>(json!(s)).ok());

            let filtered: Vec<&ModelInfo> = models
                .iter()
                .filter(|m| {
                    provider_filter
                        .as_ref()
                        .map_or(true, |p| m.provider == *p)
                })
                .collect();

            Ok(json!({
                "models": filtered,
                "total": filtered.len()
            }))
        }
    });
}

fn register_sync(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function("rimuru.models.sync", move |input: Value| {
        let kv = kv.clone();
        async move {
            let provider_filter = input
                .get("provider")
                .and_then(|v| v.as_str())
                .and_then(|s| serde_json::from_value::<ModelProvider>(json!(s)).ok());

            let all_models = hardcoded_models();

            let models_to_sync: Vec<&ModelInfo> = all_models
                .iter()
                .filter(|m| {
                    provider_filter
                        .as_ref()
                        .map_or(true, |p| m.provider == *p)
                })
                .collect();

            let mut synced_count = 0usize;
            let mut sync_statuses: Vec<ModelSyncStatus> = Vec::new();
            let mut providers_seen = std::collections::HashSet::new();

            for model in &models_to_sync {
                let key = model.key();
                kv.set("model_info", &key, model)
                    .await
                    .map_err(|e| iii_sdk::IIIError::Handler(e.to_string()))?;
                synced_count += 1;

                if providers_seen.insert(model.provider) {
                    let provider_key = model.provider_key();
                    let count = models_to_sync
                        .iter()
                        .filter(|m| m.provider == model.provider)
                        .count();

                    let status = ModelSyncStatus {
                        provider: model.provider,
                        last_sync: Some(Utc::now()),
                        model_count: count,
                        error: None,
                    };

                    kv.set("model_sync", provider_key, &status)
                        .await
                        .map_err(|e| iii_sdk::IIIError::Handler(e.to_string()))?;

                    sync_statuses.push(status);
                }
            }

            Ok(json!({
                "synced": synced_count,
                "providers": sync_statuses,
                "timestamp": Utc::now().to_rfc3339()
            }))
        }
    });
}

fn register_get(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function("rimuru.models.get", move |input: Value| {
        let kv = kv.clone();
        async move {
            let model_id = input
                .get("model_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| iii_sdk::IIIError::Handler("model_id is required".into()))?;

            let stored: Vec<ModelInfo> = kv
                .list("model_info")
                .await
                .map_err(|e| iii_sdk::IIIError::Handler(e.to_string()))?;

            let all_models = if stored.is_empty() {
                hardcoded_models()
            } else {
                stored
            };

            let model = all_models.iter().find(|m| m.id == model_id).cloned();

            match model {
                Some(m) => Ok(json!({
                    "model": m,
                    "cost_example": {
                        "1k_input_1k_output": m.calculate_cost(1000, 1000),
                        "10k_input_4k_output": m.calculate_cost(10_000, 4_000),
                        "100k_input_8k_output": m.calculate_cost(100_000, 8_000),
                    }
                })),
                None => Err(iii_sdk::IIIError::Handler(format!(
                    "model not found: {}",
                    model_id
                ))),
            }
        }
    });
}
