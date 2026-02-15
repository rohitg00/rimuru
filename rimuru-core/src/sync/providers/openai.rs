use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use tracing::{debug, info, warn};

use crate::error::RimuruResult;
use crate::models::ModelInfo;
use crate::sync::traits::ModelSyncProvider;

const OPENAI_API_BASE: &str = "https://api.openai.com";

pub struct OpenAISyncProvider {
    client: Client,
    api_key: Option<String>,
}

impl OpenAISyncProvider {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            api_key: std::env::var("OPENAI_API_KEY").ok(),
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
                "openai".to_string(),
                "gpt-4o".to_string(),
                0.005,
                0.015,
                128000,
            ),
            ModelInfo::new(
                "openai".to_string(),
                "gpt-4o-2024-11-20".to_string(),
                0.005,
                0.015,
                128000,
            ),
            ModelInfo::new(
                "openai".to_string(),
                "gpt-4o-2024-08-06".to_string(),
                0.005,
                0.015,
                128000,
            ),
            ModelInfo::new(
                "openai".to_string(),
                "gpt-4o-mini".to_string(),
                0.00015,
                0.0006,
                128000,
            ),
            ModelInfo::new(
                "openai".to_string(),
                "gpt-4o-mini-2024-07-18".to_string(),
                0.00015,
                0.0006,
                128000,
            ),
            ModelInfo::new("openai".to_string(), "o1".to_string(), 0.015, 0.060, 200000),
            ModelInfo::new(
                "openai".to_string(),
                "o1-2024-12-17".to_string(),
                0.015,
                0.060,
                200000,
            ),
            ModelInfo::new(
                "openai".to_string(),
                "o1-mini".to_string(),
                0.003,
                0.012,
                128000,
            ),
            ModelInfo::new(
                "openai".to_string(),
                "o1-mini-2024-09-12".to_string(),
                0.003,
                0.012,
                128000,
            ),
            ModelInfo::new(
                "openai".to_string(),
                "o1-preview".to_string(),
                0.015,
                0.060,
                128000,
            ),
            ModelInfo::new(
                "openai".to_string(),
                "o1-preview-2024-09-12".to_string(),
                0.015,
                0.060,
                128000,
            ),
            ModelInfo::new(
                "openai".to_string(),
                "o3-mini".to_string(),
                0.0011,
                0.0044,
                200000,
            ),
            ModelInfo::new(
                "openai".to_string(),
                "o3-mini-2025-01-31".to_string(),
                0.0011,
                0.0044,
                200000,
            ),
            ModelInfo::new(
                "openai".to_string(),
                "gpt-4-turbo".to_string(),
                0.01,
                0.03,
                128000,
            ),
            ModelInfo::new(
                "openai".to_string(),
                "gpt-4-turbo-2024-04-09".to_string(),
                0.01,
                0.03,
                128000,
            ),
            ModelInfo::new(
                "openai".to_string(),
                "gpt-4-turbo-preview".to_string(),
                0.01,
                0.03,
                128000,
            ),
            ModelInfo::new("openai".to_string(), "gpt-4".to_string(), 0.03, 0.06, 8192),
            ModelInfo::new(
                "openai".to_string(),
                "gpt-4-0613".to_string(),
                0.03,
                0.06,
                8192,
            ),
            ModelInfo::new(
                "openai".to_string(),
                "gpt-4-32k".to_string(),
                0.06,
                0.12,
                32768,
            ),
            ModelInfo::new(
                "openai".to_string(),
                "gpt-4-32k-0613".to_string(),
                0.06,
                0.12,
                32768,
            ),
            ModelInfo::new(
                "openai".to_string(),
                "gpt-3.5-turbo".to_string(),
                0.0005,
                0.0015,
                16385,
            ),
            ModelInfo::new(
                "openai".to_string(),
                "gpt-3.5-turbo-0125".to_string(),
                0.0005,
                0.0015,
                16385,
            ),
            ModelInfo::new(
                "openai".to_string(),
                "gpt-3.5-turbo-1106".to_string(),
                0.001,
                0.002,
                16385,
            ),
            ModelInfo::new(
                "openai".to_string(),
                "gpt-3.5-turbo-instruct".to_string(),
                0.0015,
                0.002,
                4096,
            ),
            ModelInfo::new(
                "openai".to_string(),
                "text-embedding-3-small".to_string(),
                0.00002,
                0.0,
                8191,
            ),
            ModelInfo::new(
                "openai".to_string(),
                "text-embedding-3-large".to_string(),
                0.00013,
                0.0,
                8191,
            ),
            ModelInfo::new(
                "openai".to_string(),
                "text-embedding-ada-002".to_string(),
                0.0001,
                0.0,
                8191,
            ),
        ]
    }

    async fn fetch_from_api(&self) -> RimuruResult<Option<Vec<ModelInfo>>> {
        let api_key = match &self.api_key {
            Some(key) => key,
            None => {
                debug!("No OpenAI API key available, using known models");
                return Ok(None);
            }
        };

        let response = self
            .client
            .get(format!("{}/v1/models", OPENAI_API_BASE))
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .send()
            .await;

        match response {
            Ok(resp) => {
                if resp.status().is_success() {
                    match resp.json::<OpenAIModelsResponse>().await {
                        Ok(data) => {
                            let models: Vec<ModelInfo> = data
                                .data
                                .into_iter()
                                .filter(|m| is_chat_model(&m.id))
                                .map(|m| {
                                    let (input_price, output_price, context) =
                                        get_model_details(&m.id);
                                    ModelInfo::new(
                                        "openai".to_string(),
                                        m.id,
                                        input_price,
                                        output_price,
                                        context,
                                    )
                                })
                                .collect();
                            Ok(Some(models))
                        }
                        Err(e) => {
                            warn!("Failed to parse OpenAI API response: {}", e);
                            Ok(None)
                        }
                    }
                } else {
                    warn!("OpenAI API returned status: {}", resp.status());
                    Ok(None)
                }
            }
            Err(e) => {
                warn!("Failed to fetch from OpenAI API: {}", e);
                Ok(None)
            }
        }
    }
}

fn is_chat_model(model_id: &str) -> bool {
    model_id.starts_with("gpt-")
        || model_id.starts_with("o1")
        || model_id.starts_with("o3")
        || model_id.starts_with("text-embedding")
        || model_id.starts_with("chatgpt")
}

fn get_model_details(model_id: &str) -> (f64, f64, i32) {
    match model_id {
        id if id.starts_with("gpt-4o-mini") => (0.00015, 0.0006, 128000),
        id if id.starts_with("gpt-4o") => (0.005, 0.015, 128000),
        id if id == "o1" || id.starts_with("o1-2024") => (0.015, 0.060, 200000),
        id if id.starts_with("o1-mini") => (0.003, 0.012, 128000),
        id if id.starts_with("o1-preview") => (0.015, 0.060, 128000),
        id if id.starts_with("o3-mini") => (0.0011, 0.0044, 200000),
        id if id.contains("gpt-4-turbo") => (0.01, 0.03, 128000),
        id if id.starts_with("gpt-4-32k") => (0.06, 0.12, 32768),
        id if id.starts_with("gpt-4") => (0.03, 0.06, 8192),
        id if id.contains("gpt-3.5-turbo-instruct") => (0.0015, 0.002, 4096),
        id if id.starts_with("gpt-3.5-turbo") => (0.0005, 0.0015, 16385),
        id if id.contains("text-embedding-3-small") => (0.00002, 0.0, 8191),
        id if id.contains("text-embedding-3-large") => (0.00013, 0.0, 8191),
        id if id.contains("text-embedding-ada") => (0.0001, 0.0, 8191),
        _ => (0.001, 0.002, 4096),
    }
}

impl Default for OpenAISyncProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
struct OpenAIModelsResponse {
    data: Vec<OpenAIModel>,
}

#[derive(Debug, Deserialize)]
struct OpenAIModel {
    id: String,
    object: String,
    created: Option<i64>,
    owned_by: Option<String>,
}

#[async_trait]
impl ModelSyncProvider for OpenAISyncProvider {
    fn provider_name(&self) -> &str {
        "openai"
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
        info!("Fetching OpenAI models");

        if let Some(api_models) = self.fetch_from_api().await? {
            info!("Fetched {} models from OpenAI API", api_models.len());
            return Ok(api_models);
        }

        let known_models = Self::get_known_models();
        info!(
            "Using {} known OpenAI models (API not available)",
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
            .get(format!("{}/v1/models", OPENAI_API_BASE))
            .header(
                "Authorization",
                format!("Bearer {}", self.api_key.as_ref().unwrap()),
            )
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
        let provider = OpenAISyncProvider::new();
        assert_eq!(provider.provider_name(), "openai");
    }

    #[test]
    fn test_is_official_source() {
        let provider = OpenAISyncProvider::new();
        assert!(provider.is_official_source());
    }

    #[test]
    fn test_priority() {
        let provider = OpenAISyncProvider::new();
        assert_eq!(provider.priority(), 1);
    }

    #[test]
    fn test_known_models() {
        let models = OpenAISyncProvider::get_known_models();
        assert!(!models.is_empty());

        let gpt4o = models.iter().find(|m| m.model_name == "gpt-4o");
        assert!(gpt4o.is_some());
        let gpt4o = gpt4o.unwrap();
        assert_eq!(gpt4o.provider, "openai");
        assert_eq!(gpt4o.input_price_per_1k, 0.005);
        assert_eq!(gpt4o.output_price_per_1k, 0.015);
        assert_eq!(gpt4o.context_window, 128000);

        let gpt4o_mini = models.iter().find(|m| m.model_name == "gpt-4o-mini");
        assert!(gpt4o_mini.is_some());
        let gpt4o_mini = gpt4o_mini.unwrap();
        assert_eq!(gpt4o_mini.input_price_per_1k, 0.00015);

        let o1 = models.iter().find(|m| m.model_name == "o1");
        assert!(o1.is_some());
        let o1 = o1.unwrap();
        assert_eq!(o1.input_price_per_1k, 0.015);
        assert_eq!(o1.output_price_per_1k, 0.060);

        let o3_mini = models.iter().find(|m| m.model_name == "o3-mini");
        assert!(o3_mini.is_some());

        let embedding = models
            .iter()
            .find(|m| m.model_name == "text-embedding-3-small");
        assert!(embedding.is_some());
    }

    #[test]
    fn test_is_chat_model() {
        assert!(is_chat_model("gpt-4"));
        assert!(is_chat_model("gpt-4o"));
        assert!(is_chat_model("gpt-3.5-turbo"));
        assert!(is_chat_model("o1"));
        assert!(is_chat_model("o1-mini"));
        assert!(is_chat_model("o3-mini"));
        assert!(is_chat_model("text-embedding-3-small"));
        assert!(!is_chat_model("whisper-1"));
        assert!(!is_chat_model("dall-e-3"));
    }

    #[test]
    fn test_get_model_details() {
        let (input, output, context) = get_model_details("gpt-4o");
        assert_eq!(input, 0.005);
        assert_eq!(output, 0.015);
        assert_eq!(context, 128000);

        let (input, output, context) = get_model_details("gpt-4o-mini");
        assert_eq!(input, 0.00015);
        assert_eq!(output, 0.0006);
        assert_eq!(context, 128000);

        let (input, output, context) = get_model_details("o1");
        assert_eq!(input, 0.015);
        assert_eq!(output, 0.060);
        assert_eq!(context, 200000);

        let (input, output, context) = get_model_details("gpt-4-turbo");
        assert_eq!(input, 0.01);
        assert_eq!(output, 0.03);
        assert_eq!(context, 128000);
    }

    #[tokio::test]
    async fn test_fetch_models_without_api_key() {
        let provider = OpenAISyncProvider::new();
        let models = provider.fetch_models().await.unwrap();
        assert!(!models.is_empty());

        for model in &models {
            assert_eq!(model.provider, "openai");
            assert!(model.context_window > 0);
            assert!(model.input_price_per_1k >= 0.0);
            assert!(model.output_price_per_1k >= 0.0);
        }
    }

    #[tokio::test]
    async fn test_health_check_without_api_key() {
        let provider = OpenAISyncProvider::new();
        let healthy = provider.health_check().await.unwrap();
        assert!(healthy);
    }
}
