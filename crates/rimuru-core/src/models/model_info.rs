use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelProvider {
    Anthropic,
    OpenAI,
    Google,
    OpenRouter,
    LiteLLM,
}

impl std::fmt::Display for ModelProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Anthropic => write!(f, "Anthropic"),
            Self::OpenAI => write!(f, "OpenAI"),
            Self::Google => write!(f, "Google"),
            Self::OpenRouter => write!(f, "OpenRouter"),
            Self::LiteLLM => write!(f, "LiteLLM"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub provider: ModelProvider,
    pub input_price_per_million: f64,
    pub output_price_per_million: f64,
    pub cache_read_price_per_million: Option<f64>,
    pub cache_write_price_per_million: Option<f64>,
    pub context_window: u64,
    pub max_output_tokens: Option<u64>,
    pub supports_vision: bool,
    pub supports_tools: bool,
    pub last_synced: DateTime<Utc>,
}

impl ModelInfo {
    pub fn key(&self) -> String {
        format!("{}::{}", self.provider_key(), self.id)
    }

    pub fn provider_key(&self) -> &'static str {
        match self.provider {
            ModelProvider::Anthropic => "anthropic",
            ModelProvider::OpenAI => "openai",
            ModelProvider::Google => "google",
            ModelProvider::OpenRouter => "openrouter",
            ModelProvider::LiteLLM => "litellm",
        }
    }

    pub fn calculate_cost(&self, input_tokens: u64, output_tokens: u64) -> f64 {
        let input_cost = (input_tokens as f64 / 1_000_000.0) * self.input_price_per_million;
        let output_cost = (output_tokens as f64 / 1_000_000.0) * self.output_price_per_million;
        input_cost + output_cost
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelSyncStatus {
    pub provider: ModelProvider,
    pub last_sync: Option<DateTime<Utc>>,
    pub model_count: usize,
    pub error: Option<String>,
}
