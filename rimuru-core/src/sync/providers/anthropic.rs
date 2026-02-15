use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use tracing::{debug, info, warn};

use crate::error::RimuruResult;
use crate::models::ModelInfo;
use crate::sync::traits::ModelSyncProvider;

const ANTHROPIC_API_BASE: &str = "https://api.anthropic.com";
const ANTHROPIC_API_VERSION: &str = "2024-01-01";

pub struct AnthropicSyncProvider {
    client: Client,
    api_key: Option<String>,
}

impl AnthropicSyncProvider {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            api_key: std::env::var("ANTHROPIC_API_KEY").ok(),
        }
    }

    pub fn with_api_key(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key: Some(api_key),
        }
    }

    fn get_known_models() -> Vec<ModelInfo> {
        vec![
            ModelInfo::new(
                "anthropic".to_string(),
                "claude-opus-4-5-20251101".to_string(),
                0.015,
                0.075,
                200000,
            ),
            ModelInfo::new(
                "anthropic".to_string(),
                "claude-sonnet-4-20250514".to_string(),
                0.003,
                0.015,
                200000,
            ),
            ModelInfo::new(
                "anthropic".to_string(),
                "claude-3-5-sonnet-20241022".to_string(),
                0.003,
                0.015,
                200000,
            ),
            ModelInfo::new(
                "anthropic".to_string(),
                "claude-3-5-sonnet-20240620".to_string(),
                0.003,
                0.015,
                200000,
            ),
            ModelInfo::new(
                "anthropic".to_string(),
                "claude-3-5-haiku-20241022".to_string(),
                0.0008,
                0.004,
                200000,
            ),
            ModelInfo::new(
                "anthropic".to_string(),
                "claude-3-opus-20240229".to_string(),
                0.015,
                0.075,
                200000,
            ),
            ModelInfo::new(
                "anthropic".to_string(),
                "claude-3-sonnet-20240229".to_string(),
                0.003,
                0.015,
                200000,
            ),
            ModelInfo::new(
                "anthropic".to_string(),
                "claude-3-haiku-20240307".to_string(),
                0.00025,
                0.00125,
                200000,
            ),
            ModelInfo::new(
                "anthropic".to_string(),
                "claude-2.1".to_string(),
                0.008,
                0.024,
                200000,
            ),
            ModelInfo::new(
                "anthropic".to_string(),
                "claude-2.0".to_string(),
                0.008,
                0.024,
                100000,
            ),
            ModelInfo::new(
                "anthropic".to_string(),
                "claude-instant-1.2".to_string(),
                0.0008,
                0.0024,
                100000,
            ),
        ]
    }

    async fn fetch_from_api(&self) -> RimuruResult<Option<Vec<ModelInfo>>> {
        let api_key = match &self.api_key {
            Some(key) => key,
            None => {
                debug!("No Anthropic API key available, using known models");
                return Ok(None);
            }
        };

        let response = self
            .client
            .get(format!("{}/v1/models", ANTHROPIC_API_BASE))
            .header("x-api-key", api_key)
            .header("anthropic-version", ANTHROPIC_API_VERSION)
            .header("content-type", "application/json")
            .send()
            .await;

        match response {
            Ok(resp) => {
                if resp.status().is_success() {
                    match resp.json::<AnthropicModelsResponse>().await {
                        Ok(data) => {
                            let models: Vec<ModelInfo> = data
                                .data
                                .into_iter()
                                .map(|m| {
                                    let (input_price, output_price) = get_model_pricing(&m.id);
                                    ModelInfo::new(
                                        "anthropic".to_string(),
                                        m.id,
                                        input_price,
                                        output_price,
                                        m.context_window.unwrap_or(200000),
                                    )
                                })
                                .collect();
                            Ok(Some(models))
                        }
                        Err(e) => {
                            warn!("Failed to parse Anthropic API response: {}", e);
                            Ok(None)
                        }
                    }
                } else if resp.status().as_u16() == 404 {
                    debug!("Anthropic models endpoint not available, using known models");
                    Ok(None)
                } else {
                    warn!("Anthropic API returned status: {}", resp.status());
                    Ok(None)
                }
            }
            Err(e) => {
                warn!("Failed to fetch from Anthropic API: {}", e);
                Ok(None)
            }
        }
    }
}

fn get_model_pricing(model_id: &str) -> (f64, f64) {
    match model_id {
        id if id.contains("opus-4-5") => (0.015, 0.075),
        id if id.contains("sonnet-4") => (0.003, 0.015),
        id if id.contains("3-5-sonnet") => (0.003, 0.015),
        id if id.contains("3-5-haiku") => (0.0008, 0.004),
        id if id.contains("3-opus") => (0.015, 0.075),
        id if id.contains("3-sonnet") => (0.003, 0.015),
        id if id.contains("3-haiku") => (0.00025, 0.00125),
        id if id.contains("2.1") => (0.008, 0.024),
        id if id.contains("2.0") => (0.008, 0.024),
        id if id.contains("instant") => (0.0008, 0.0024),
        _ => (0.003, 0.015),
    }
}

impl Default for AnthropicSyncProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
struct AnthropicModelsResponse {
    data: Vec<AnthropicModel>,
}

#[derive(Debug, Deserialize)]
struct AnthropicModel {
    id: String,
    #[serde(rename = "type")]
    model_type: Option<String>,
    display_name: Option<String>,
    context_window: Option<i32>,
    #[serde(default)]
    created_at: Option<String>,
}

#[async_trait]
impl ModelSyncProvider for AnthropicSyncProvider {
    fn provider_name(&self) -> &str {
        "anthropic"
    }

    fn supports_streaming(&self) -> bool {
        false
    }

    fn is_official_source(&self) -> bool {
        true
    }

    fn priority(&self) -> u8 {
        1
    }

    async fn fetch_models(&self) -> RimuruResult<Vec<ModelInfo>> {
        info!("Fetching Anthropic models");

        if let Some(api_models) = self.fetch_from_api().await? {
            info!("Fetched {} models from Anthropic API", api_models.len());
            return Ok(api_models);
        }

        let known_models = Self::get_known_models();
        info!(
            "Using {} known Anthropic models (API not available)",
            known_models.len()
        );

        Ok(known_models)
    }

    async fn health_check(&self) -> RimuruResult<bool> {
        if self.api_key.is_none() {
            return Ok(true);
        }

        let response = self
            .client
            .get(format!("{}/v1/models", ANTHROPIC_API_BASE))
            .header("x-api-key", self.api_key.as_ref().unwrap())
            .header("anthropic-version", ANTHROPIC_API_VERSION)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await;

        match response {
            Ok(resp) => Ok(resp.status().is_success() || resp.status().as_u16() == 404),
            Err(_) => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_name() {
        let provider = AnthropicSyncProvider::new();
        assert_eq!(provider.provider_name(), "anthropic");
    }

    #[test]
    fn test_is_official_source() {
        let provider = AnthropicSyncProvider::new();
        assert!(provider.is_official_source());
    }

    #[test]
    fn test_priority() {
        let provider = AnthropicSyncProvider::new();
        assert_eq!(provider.priority(), 1);
    }

    #[test]
    fn test_known_models() {
        let models = AnthropicSyncProvider::get_known_models();
        assert!(!models.is_empty());

        let opus = models.iter().find(|m| m.model_name.contains("opus-4-5"));
        assert!(opus.is_some());
        let opus = opus.unwrap();
        assert_eq!(opus.provider, "anthropic");
        assert_eq!(opus.input_price_per_1k, 0.015);
        assert_eq!(opus.output_price_per_1k, 0.075);
        assert_eq!(opus.context_window, 200000);

        let sonnet = models.iter().find(|m| m.model_name.contains("3-5-sonnet"));
        assert!(sonnet.is_some());
        let sonnet = sonnet.unwrap();
        assert_eq!(sonnet.input_price_per_1k, 0.003);

        let haiku = models.iter().find(|m| m.model_name.contains("3-haiku"));
        assert!(haiku.is_some());
        let haiku = haiku.unwrap();
        assert_eq!(haiku.input_price_per_1k, 0.00025);
    }

    #[test]
    fn test_get_model_pricing() {
        let (input, output) = get_model_pricing("claude-opus-4-5-20251101");
        assert_eq!(input, 0.015);
        assert_eq!(output, 0.075);

        let (input, output) = get_model_pricing("claude-3-5-sonnet-20241022");
        assert_eq!(input, 0.003);
        assert_eq!(output, 0.015);

        let (input, output) = get_model_pricing("claude-3-haiku-20240307");
        assert_eq!(input, 0.00025);
        assert_eq!(output, 0.00125);

        let (input, output) = get_model_pricing("unknown-model");
        assert_eq!(input, 0.003);
        assert_eq!(output, 0.015);
    }

    #[tokio::test]
    async fn test_fetch_models_without_api_key() {
        let provider = AnthropicSyncProvider::new();
        let models = provider.fetch_models().await.unwrap();
        assert!(!models.is_empty());

        for model in &models {
            assert_eq!(model.provider, "anthropic");
            assert!(model.context_window > 0);
            assert!(model.input_price_per_1k >= 0.0);
            assert!(model.output_price_per_1k >= 0.0);
        }
    }

    #[tokio::test]
    async fn test_health_check_without_api_key() {
        let provider = AnthropicSyncProvider::new();
        let healthy = provider.health_check().await.unwrap();
        assert!(healthy);
    }
}
