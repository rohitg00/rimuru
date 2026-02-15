use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use tracing::{info, warn};

use crate::error::RimuruResult;
use crate::models::ModelInfo;
use crate::sync::aggregator::ModelAggregator;
use crate::sync::traits::ModelSyncProvider;

const LITELLM_PRICING_URL: &str =
    "https://raw.githubusercontent.com/BerriAI/litellm/main/model_prices_and_context_window.json";

pub struct LiteLLMSyncProvider {
    client: Client,
}

impl LiteLLMSyncProvider {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    async fn fetch_from_github(&self) -> RimuruResult<Vec<ModelInfo>> {
        let response = self
            .client
            .get(LITELLM_PRICING_URL)
            .header("Accept", "application/json")
            .send()
            .await;

        match response {
            Ok(resp) => {
                if resp.status().is_success() {
                    match resp.json::<HashMap<String, LiteLLMModelInfo>>().await {
                        Ok(data) => {
                            let models: Vec<ModelInfo> = data
                                .into_iter()
                                .filter(|(key, model)| {
                                    !key.starts_with("sample_spec")
                                        && model.input_cost_per_token.is_some()
                                })
                                .map(|(key, model)| {
                                    let (provider, model_name) = parse_model_key(&key);
                                    let input_price =
                                        model.input_cost_per_token.unwrap_or(0.0) * 1000.0;
                                    let output_price =
                                        model.output_cost_per_token.unwrap_or(0.0) * 1000.0;
                                    let context_window = model.max_tokens.unwrap_or(4096);

                                    ModelInfo::new(
                                        ModelAggregator::normalize_provider_name(&provider),
                                        model_name,
                                        input_price,
                                        output_price,
                                        context_window,
                                    )
                                })
                                .collect();

                            Ok(models)
                        }
                        Err(e) => {
                            warn!("Failed to parse LiteLLM pricing data: {}", e);
                            Ok(Vec::new())
                        }
                    }
                } else {
                    warn!("LiteLLM pricing fetch returned status: {}", resp.status());
                    Ok(Vec::new())
                }
            }
            Err(e) => {
                warn!("Failed to fetch LiteLLM pricing data: {}", e);
                Ok(Vec::new())
            }
        }
    }
}

fn parse_model_key(key: &str) -> (String, String) {
    if let Some(slash_pos) = key.find('/') {
        let provider = &key[..slash_pos];
        let model_name = &key[slash_pos + 1..];
        (provider.to_string(), model_name.to_string())
    } else {
        let provider = infer_provider_from_model(key);
        (provider, key.to_string())
    }
}

fn infer_provider_from_model(model_name: &str) -> String {
    match model_name {
        name if name.starts_with("claude") => "anthropic".to_string(),
        name if name.starts_with("gpt") || name.starts_with("o1") || name.starts_with("o3") => {
            "openai".to_string()
        }
        name if name.starts_with("gemini") => "google".to_string(),
        name if name.starts_with("llama") => "meta".to_string(),
        name if name.starts_with("mistral") || name.starts_with("mixtral") => "mistral".to_string(),
        name if name.starts_with("command") => "cohere".to_string(),
        _ => "unknown".to_string(),
    }
}

impl Default for LiteLLMSyncProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
struct LiteLLMModelInfo {
    #[serde(default)]
    max_tokens: Option<i32>,
    #[serde(default)]
    max_input_tokens: Option<i32>,
    #[serde(default)]
    max_output_tokens: Option<i32>,
    #[serde(default)]
    input_cost_per_token: Option<f64>,
    #[serde(default)]
    output_cost_per_token: Option<f64>,
    #[serde(default)]
    litellm_provider: Option<String>,
    #[serde(default)]
    mode: Option<String>,
    #[serde(default)]
    supports_function_calling: Option<bool>,
    #[serde(default)]
    supports_vision: Option<bool>,
    #[serde(default)]
    supports_parallel_function_calling: Option<bool>,
}

#[async_trait]
impl ModelSyncProvider for LiteLLMSyncProvider {
    fn provider_name(&self) -> &str {
        "litellm"
    }

    fn supports_streaming(&self) -> bool {
        false
    }

    fn is_official_source(&self) -> bool {
        false
    }

    fn priority(&self) -> u8 {
        60
    }

    async fn fetch_models(&self) -> RimuruResult<Vec<ModelInfo>> {
        info!("Fetching models from LiteLLM registry");

        let models = self.fetch_from_github().await?;

        if models.is_empty() {
            warn!("No models fetched from LiteLLM registry");
        } else {
            info!("Fetched {} models from LiteLLM registry", models.len());
        }

        Ok(models)
    }

    async fn health_check(&self) -> RimuruResult<bool> {
        let response = self
            .client
            .head(LITELLM_PRICING_URL)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await;

        match response {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(_) => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_name() {
        let provider = LiteLLMSyncProvider::new();
        assert_eq!(provider.provider_name(), "litellm");
    }

    #[test]
    fn test_is_official_source() {
        let provider = LiteLLMSyncProvider::new();
        assert!(!provider.is_official_source());
    }

    #[test]
    fn test_priority() {
        let provider = LiteLLMSyncProvider::new();
        assert_eq!(provider.priority(), 60);
    }

    #[test]
    fn test_parse_model_key_with_slash() {
        let (provider, name) = parse_model_key("anthropic/claude-3-opus");
        assert_eq!(provider, "anthropic");
        assert_eq!(name, "claude-3-opus");

        let (provider, name) = parse_model_key("openai/gpt-4-turbo");
        assert_eq!(provider, "openai");
        assert_eq!(name, "gpt-4-turbo");
    }

    #[test]
    fn test_parse_model_key_without_slash() {
        let (provider, name) = parse_model_key("gpt-4");
        assert_eq!(provider, "openai");
        assert_eq!(name, "gpt-4");

        let (provider, name) = parse_model_key("claude-3-opus");
        assert_eq!(provider, "anthropic");
        assert_eq!(name, "claude-3-opus");
    }

    #[test]
    fn test_infer_provider_from_model() {
        assert_eq!(infer_provider_from_model("claude-3-opus"), "anthropic");
        assert_eq!(infer_provider_from_model("gpt-4"), "openai");
        assert_eq!(infer_provider_from_model("o1-preview"), "openai");
        assert_eq!(infer_provider_from_model("o3-mini"), "openai");
        assert_eq!(infer_provider_from_model("gemini-pro"), "google");
        assert_eq!(infer_provider_from_model("llama-3-70b"), "meta");
        assert_eq!(infer_provider_from_model("mistral-large"), "mistral");
        assert_eq!(infer_provider_from_model("mixtral-8x7b"), "mistral");
        assert_eq!(infer_provider_from_model("command-r"), "cohere");
        assert_eq!(infer_provider_from_model("unknown-model"), "unknown");
    }

    #[tokio::test]
    async fn test_health_check() {
        let provider = LiteLLMSyncProvider::new();
        let result = provider.health_check().await;
        assert!(result.is_ok());
    }
}
