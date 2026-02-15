use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use tracing::{debug, info, warn};

use crate::error::RimuruResult;
use crate::models::ModelInfo;
use crate::sync::traits::ModelSyncProvider;

const GOOGLE_AI_API_BASE: &str = "https://generativelanguage.googleapis.com";

pub struct GoogleSyncProvider {
    client: Client,
    api_key: Option<String>,
}

impl GoogleSyncProvider {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            api_key: std::env::var("GOOGLE_API_KEY")
                .or_else(|_| std::env::var("GEMINI_API_KEY"))
                .ok(),
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
                "google".to_string(),
                "gemini-2.0-flash-exp".to_string(),
                0.0,
                0.0,
                1048576,
            ),
            ModelInfo::new(
                "google".to_string(),
                "gemini-2.0-flash".to_string(),
                0.0,
                0.0,
                1048576,
            ),
            ModelInfo::new(
                "google".to_string(),
                "gemini-2.0-flash-thinking-exp".to_string(),
                0.0,
                0.0,
                1048576,
            ),
            ModelInfo::new(
                "google".to_string(),
                "gemini-1.5-pro".to_string(),
                0.00125,
                0.005,
                2097152,
            ),
            ModelInfo::new(
                "google".to_string(),
                "gemini-1.5-pro-002".to_string(),
                0.00125,
                0.005,
                2097152,
            ),
            ModelInfo::new(
                "google".to_string(),
                "gemini-1.5-pro-001".to_string(),
                0.00125,
                0.005,
                2097152,
            ),
            ModelInfo::new(
                "google".to_string(),
                "gemini-1.5-flash".to_string(),
                0.000075,
                0.0003,
                1048576,
            ),
            ModelInfo::new(
                "google".to_string(),
                "gemini-1.5-flash-002".to_string(),
                0.000075,
                0.0003,
                1048576,
            ),
            ModelInfo::new(
                "google".to_string(),
                "gemini-1.5-flash-001".to_string(),
                0.000075,
                0.0003,
                1048576,
            ),
            ModelInfo::new(
                "google".to_string(),
                "gemini-1.5-flash-8b".to_string(),
                0.0000375,
                0.00015,
                1048576,
            ),
            ModelInfo::new(
                "google".to_string(),
                "gemini-1.0-pro".to_string(),
                0.0005,
                0.0015,
                32760,
            ),
            ModelInfo::new(
                "google".to_string(),
                "gemini-1.0-pro-001".to_string(),
                0.0005,
                0.0015,
                32760,
            ),
            ModelInfo::new(
                "google".to_string(),
                "gemini-pro".to_string(),
                0.0005,
                0.0015,
                32760,
            ),
            ModelInfo::new(
                "google".to_string(),
                "gemini-pro-vision".to_string(),
                0.0005,
                0.0015,
                16384,
            ),
            ModelInfo::new(
                "google".to_string(),
                "text-embedding-004".to_string(),
                0.00001,
                0.0,
                2048,
            ),
            ModelInfo::new(
                "google".to_string(),
                "embedding-001".to_string(),
                0.00001,
                0.0,
                2048,
            ),
        ]
    }

    async fn fetch_from_api(&self) -> RimuruResult<Option<Vec<ModelInfo>>> {
        let api_key = match &self.api_key {
            Some(key) => key,
            None => {
                debug!("No Google API key available, using known models");
                return Ok(None);
            }
        };

        let url = format!("{}/v1beta/models?key={}", GOOGLE_AI_API_BASE, api_key);

        let response = self
            .client
            .get(&url)
            .header("Content-Type", "application/json")
            .send()
            .await;

        match response {
            Ok(resp) => {
                if resp.status().is_success() {
                    match resp.json::<GoogleModelsResponse>().await {
                        Ok(data) => {
                            let models: Vec<ModelInfo> = data
                                .models
                                .into_iter()
                                .filter(|m| is_generative_model(&m.name))
                                .map(|m| {
                                    let model_name = extract_model_name(&m.name);
                                    let (input_price, output_price) =
                                        get_model_pricing(&model_name);
                                    let context_window = m.input_token_limit.unwrap_or(32760);
                                    ModelInfo::new(
                                        "google".to_string(),
                                        model_name,
                                        input_price,
                                        output_price,
                                        context_window,
                                    )
                                })
                                .collect();
                            Ok(Some(models))
                        }
                        Err(e) => {
                            warn!("Failed to parse Google API response: {}", e);
                            Ok(None)
                        }
                    }
                } else {
                    warn!("Google API returned status: {}", resp.status());
                    Ok(None)
                }
            }
            Err(e) => {
                warn!("Failed to fetch from Google API: {}", e);
                Ok(None)
            }
        }
    }
}

fn is_generative_model(name: &str) -> bool {
    name.contains("gemini") || name.contains("text-embedding") || name.contains("embedding")
}

fn extract_model_name(full_name: &str) -> String {
    full_name
        .strip_prefix("models/")
        .unwrap_or(full_name)
        .to_string()
}

fn get_model_pricing(model_name: &str) -> (f64, f64) {
    match model_name {
        name if name.contains("2.0-flash") => (0.0, 0.0),
        name if name.contains("1.5-pro") => (0.00125, 0.005),
        name if name.contains("1.5-flash-8b") => (0.0000375, 0.00015),
        name if name.contains("1.5-flash") => (0.000075, 0.0003),
        name if name.contains("1.0-pro") || name == "gemini-pro" => (0.0005, 0.0015),
        name if name.contains("pro-vision") => (0.0005, 0.0015),
        name if name.contains("embedding") => (0.00001, 0.0),
        _ => (0.0005, 0.0015),
    }
}

impl Default for GoogleSyncProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize)]
struct GoogleModelsResponse {
    models: Vec<GoogleModel>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GoogleModel {
    name: String,
    #[serde(default)]
    display_name: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    input_token_limit: Option<i32>,
    #[serde(default)]
    output_token_limit: Option<i32>,
    #[serde(default)]
    supported_generation_methods: Vec<String>,
}

#[async_trait]
impl ModelSyncProvider for GoogleSyncProvider {
    fn provider_name(&self) -> &str {
        "google"
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
        info!("Fetching Google AI models");

        if let Some(api_models) = self.fetch_from_api().await? {
            info!("Fetched {} models from Google API", api_models.len());
            return Ok(api_models);
        }

        let known_models = Self::get_known_models();
        info!(
            "Using {} known Google models (API not available)",
            known_models.len()
        );

        Ok(known_models)
    }

    async fn health_check(&self) -> RimuruResult<bool> {
        if self.api_key.is_none() {
            return Ok(true);
        }

        let url = format!(
            "{}/v1beta/models?key={}",
            GOOGLE_AI_API_BASE,
            self.api_key.as_ref().unwrap()
        );

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
        let provider = GoogleSyncProvider::new();
        assert_eq!(provider.provider_name(), "google");
    }

    #[test]
    fn test_is_official_source() {
        let provider = GoogleSyncProvider::new();
        assert!(provider.is_official_source());
    }

    #[test]
    fn test_priority() {
        let provider = GoogleSyncProvider::new();
        assert_eq!(provider.priority(), 1);
    }

    #[test]
    fn test_known_models() {
        let models = GoogleSyncProvider::get_known_models();
        assert!(!models.is_empty());

        let gemini_pro = models.iter().find(|m| m.model_name == "gemini-1.5-pro");
        assert!(gemini_pro.is_some());
        let gemini_pro = gemini_pro.unwrap();
        assert_eq!(gemini_pro.provider, "google");
        assert_eq!(gemini_pro.input_price_per_1k, 0.00125);
        assert_eq!(gemini_pro.output_price_per_1k, 0.005);
        assert_eq!(gemini_pro.context_window, 2097152);

        let gemini_flash = models.iter().find(|m| m.model_name == "gemini-1.5-flash");
        assert!(gemini_flash.is_some());
        let gemini_flash = gemini_flash.unwrap();
        assert_eq!(gemini_flash.input_price_per_1k, 0.000075);
        assert_eq!(gemini_flash.context_window, 1048576);

        let gemini_2_flash = models.iter().find(|m| m.model_name == "gemini-2.0-flash");
        assert!(gemini_2_flash.is_some());
    }

    #[test]
    fn test_is_generative_model() {
        assert!(is_generative_model("models/gemini-1.5-pro"));
        assert!(is_generative_model("models/gemini-2.0-flash"));
        assert!(is_generative_model("models/text-embedding-004"));
        assert!(!is_generative_model("models/some-other-model"));
    }

    #[test]
    fn test_extract_model_name() {
        assert_eq!(
            extract_model_name("models/gemini-1.5-pro"),
            "gemini-1.5-pro"
        );
        assert_eq!(extract_model_name("gemini-1.5-pro"), "gemini-1.5-pro");
    }

    #[test]
    fn test_get_model_pricing() {
        let (input, output) = get_model_pricing("gemini-1.5-pro");
        assert_eq!(input, 0.00125);
        assert_eq!(output, 0.005);

        let (input, output) = get_model_pricing("gemini-1.5-flash");
        assert_eq!(input, 0.000075);
        assert_eq!(output, 0.0003);

        let (input, output) = get_model_pricing("gemini-2.0-flash");
        assert_eq!(input, 0.0);
        assert_eq!(output, 0.0);

        let (input, output) = get_model_pricing("text-embedding-004");
        assert_eq!(input, 0.00001);
        assert_eq!(output, 0.0);
    }

    #[tokio::test]
    async fn test_fetch_models_without_api_key() {
        let provider = GoogleSyncProvider::new();
        let models = provider.fetch_models().await.unwrap();
        assert!(!models.is_empty());

        for model in &models {
            assert_eq!(model.provider, "google");
            assert!(model.context_window > 0);
            assert!(model.input_price_per_1k >= 0.0);
            assert!(model.output_price_per_1k >= 0.0);
        }
    }

    #[tokio::test]
    async fn test_health_check_without_api_key() {
        let provider = GoogleSyncProvider::new();
        let healthy = provider.health_check().await.unwrap();
        assert!(healthy);
    }
}
