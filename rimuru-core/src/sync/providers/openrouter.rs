use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use tracing::{info, warn};

use crate::error::RimuruResult;
use crate::models::ModelInfo;
use crate::sync::aggregator::ModelAggregator;
use crate::sync::traits::ModelSyncProvider;

const OPENROUTER_API_BASE: &str = "https://openrouter.ai/api/v1";

pub struct OpenRouterSyncProvider {
    client: Client,
    api_key: Option<String>,
}

impl OpenRouterSyncProvider {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            api_key: std::env::var("OPENROUTER_API_KEY").ok(),
        }
    }

    pub fn with_api_key(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key: Some(api_key),
        }
    }

    async fn fetch_from_api(&self) -> RimuruResult<Vec<ModelInfo>> {
        let url = format!("{}/models", OPENROUTER_API_BASE);

        let mut request = self
            .client
            .get(&url)
            .header("Content-Type", "application/json");

        if let Some(key) = &self.api_key {
            request = request.header("Authorization", format!("Bearer {}", key));
        }

        let response = request.send().await;

        match response {
            Ok(resp) => {
                if resp.status().is_success() {
                    match resp.json::<OpenRouterModelsResponse>().await {
                        Ok(data) => {
                            let models: Vec<ModelInfo> = data
                                .data
                                .into_iter()
                                .filter(|m| m.pricing.prompt.parse::<f64>().unwrap_or(0.0) >= 0.0)
                                .map(|m| {
                                    let (provider, model_name) = parse_model_id(&m.id);
                                    let input_price =
                                        m.pricing.prompt.parse::<f64>().unwrap_or(0.0) * 1000.0;
                                    let output_price =
                                        m.pricing.completion.parse::<f64>().unwrap_or(0.0) * 1000.0;
                                    let context_window = m.context_length.unwrap_or(4096);

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
                            warn!("Failed to parse OpenRouter API response: {}", e);
                            Ok(Vec::new())
                        }
                    }
                } else {
                    warn!("OpenRouter API returned status: {}", resp.status());
                    Ok(Vec::new())
                }
            }
            Err(e) => {
                warn!("Failed to fetch from OpenRouter API: {}", e);
                Ok(Vec::new())
            }
        }
    }
}

fn parse_model_id(model_id: &str) -> (String, String) {
    if let Some(slash_pos) = model_id.find('/') {
        let provider = &model_id[..slash_pos];
        let model_name = &model_id[slash_pos + 1..];
        (provider.to_string(), model_name.to_string())
    } else {
        ("unknown".to_string(), model_id.to_string())
    }
}

impl Default for OpenRouterSyncProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
struct OpenRouterModelsResponse {
    data: Vec<OpenRouterModel>,
}

#[derive(Debug, Deserialize)]
struct OpenRouterModel {
    id: String,
    name: Option<String>,
    #[serde(default)]
    description: Option<String>,
    context_length: Option<i32>,
    pricing: OpenRouterPricing,
    #[serde(default)]
    top_provider: Option<TopProvider>,
    #[serde(default)]
    per_request_limits: Option<PerRequestLimits>,
}

#[derive(Debug, Deserialize)]
struct OpenRouterPricing {
    prompt: String,
    completion: String,
    #[serde(default)]
    image: Option<String>,
    #[serde(default)]
    request: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TopProvider {
    #[serde(default)]
    max_completion_tokens: Option<i32>,
    #[serde(default)]
    is_moderated: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct PerRequestLimits {
    #[serde(default)]
    prompt_tokens: Option<String>,
    #[serde(default)]
    completion_tokens: Option<String>,
}

#[async_trait]
impl ModelSyncProvider for OpenRouterSyncProvider {
    fn provider_name(&self) -> &str {
        "openrouter"
    }

    fn supports_streaming(&self) -> bool {
        false
    }

    fn is_official_source(&self) -> bool {
        false
    }

    fn priority(&self) -> u8 {
        50
    }

    async fn fetch_models(&self) -> RimuruResult<Vec<ModelInfo>> {
        info!("Fetching models from OpenRouter");

        let models = self.fetch_from_api().await?;

        if models.is_empty() {
            warn!("No models fetched from OpenRouter");
        } else {
            info!("Fetched {} models from OpenRouter", models.len());
        }

        Ok(models)
    }

    async fn health_check(&self) -> RimuruResult<bool> {
        let url = format!("{}/models", OPENROUTER_API_BASE);

        let response = self
            .client
            .get(&url)
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
        let provider = OpenRouterSyncProvider::new();
        assert_eq!(provider.provider_name(), "openrouter");
    }

    #[test]
    fn test_is_official_source() {
        let provider = OpenRouterSyncProvider::new();
        assert!(!provider.is_official_source());
    }

    #[test]
    fn test_priority() {
        let provider = OpenRouterSyncProvider::new();
        assert_eq!(provider.priority(), 50);
    }

    #[test]
    fn test_parse_model_id() {
        let (provider, name) = parse_model_id("anthropic/claude-3-opus");
        assert_eq!(provider, "anthropic");
        assert_eq!(name, "claude-3-opus");

        let (provider, name) = parse_model_id("openai/gpt-4");
        assert_eq!(provider, "openai");
        assert_eq!(name, "gpt-4");

        let (provider, name) = parse_model_id("mistralai/mistral-large");
        assert_eq!(provider, "mistralai");
        assert_eq!(name, "mistral-large");

        let (provider, name) = parse_model_id("some-model-without-slash");
        assert_eq!(provider, "unknown");
        assert_eq!(name, "some-model-without-slash");
    }

    #[test]
    fn test_parse_model_id_with_multiple_slashes() {
        let (provider, name) = parse_model_id("meta-llama/llama-3-70b/instruct");
        assert_eq!(provider, "meta-llama");
        assert_eq!(name, "llama-3-70b/instruct");
    }

    #[tokio::test]
    async fn test_health_check() {
        let provider = OpenRouterSyncProvider::new();
        let result = provider.health_check().await;
        assert!(result.is_ok());
    }
}
