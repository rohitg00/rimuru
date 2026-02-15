use crate::error::RimuruResult;
use crate::models::ModelInfo;
use std::collections::HashMap;

pub struct OpenCodeCostCalculator {
    models: HashMap<String, Vec<ModelInfo>>,
}

impl Default for OpenCodeCostCalculator {
    fn default() -> Self {
        Self::new()
    }
}

impl OpenCodeCostCalculator {
    pub fn new() -> Self {
        let mut models: HashMap<String, Vec<ModelInfo>> = HashMap::new();

        models.insert(
            "anthropic".to_string(),
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
                    "claude-sonnet-4-20251101".to_string(),
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
            ],
        );

        models.insert(
            "openai".to_string(),
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
                    "gpt-4o-mini".to_string(),
                    0.00015,
                    0.0006,
                    128000,
                ),
                ModelInfo::new(
                    "openai".to_string(),
                    "gpt-4-turbo".to_string(),
                    0.01,
                    0.03,
                    128000,
                ),
                ModelInfo::new("openai".to_string(), "gpt-4".to_string(), 0.03, 0.06, 8192),
                ModelInfo::new(
                    "openai".to_string(),
                    "gpt-3.5-turbo".to_string(),
                    0.0005,
                    0.0015,
                    16385,
                ),
                ModelInfo::new("openai".to_string(), "o1".to_string(), 0.015, 0.06, 200000),
                ModelInfo::new(
                    "openai".to_string(),
                    "o1-mini".to_string(),
                    0.003,
                    0.012,
                    128000,
                ),
                ModelInfo::new(
                    "openai".to_string(),
                    "o3-mini".to_string(),
                    0.0011,
                    0.0044,
                    200000,
                ),
            ],
        );

        models.insert(
            "google".to_string(),
            vec![
                ModelInfo::new(
                    "google".to_string(),
                    "gemini-2.0-flash".to_string(),
                    0.0001,
                    0.0004,
                    1000000,
                ),
                ModelInfo::new(
                    "google".to_string(),
                    "gemini-1.5-pro".to_string(),
                    0.00125,
                    0.005,
                    2000000,
                ),
                ModelInfo::new(
                    "google".to_string(),
                    "gemini-1.5-flash".to_string(),
                    0.000075,
                    0.0003,
                    1000000,
                ),
            ],
        );

        models.insert(
            "groq".to_string(),
            vec![
                ModelInfo::new(
                    "groq".to_string(),
                    "llama-3.3-70b-versatile".to_string(),
                    0.00059,
                    0.00079,
                    128000,
                ),
                ModelInfo::new(
                    "groq".to_string(),
                    "llama-3.1-8b-instant".to_string(),
                    0.00005,
                    0.00008,
                    128000,
                ),
                ModelInfo::new(
                    "groq".to_string(),
                    "mixtral-8x7b-32768".to_string(),
                    0.00024,
                    0.00024,
                    32768,
                ),
            ],
        );

        models.insert(
            "deepseek".to_string(),
            vec![
                ModelInfo::new(
                    "deepseek".to_string(),
                    "deepseek-chat".to_string(),
                    0.00014,
                    0.00028,
                    64000,
                ),
                ModelInfo::new(
                    "deepseek".to_string(),
                    "deepseek-coder".to_string(),
                    0.00014,
                    0.00028,
                    64000,
                ),
                ModelInfo::new(
                    "deepseek".to_string(),
                    "deepseek-reasoner".to_string(),
                    0.00055,
                    0.00219,
                    64000,
                ),
            ],
        );

        models.insert(
            "xai".to_string(),
            vec![
                ModelInfo::new("xai".to_string(), "grok-2".to_string(), 0.002, 0.01, 131072),
                ModelInfo::new(
                    "xai".to_string(),
                    "grok-beta".to_string(),
                    0.005,
                    0.015,
                    131072,
                ),
            ],
        );

        models.insert(
            "mistral".to_string(),
            vec![
                ModelInfo::new(
                    "mistral".to_string(),
                    "mistral-large".to_string(),
                    0.002,
                    0.006,
                    128000,
                ),
                ModelInfo::new(
                    "mistral".to_string(),
                    "mistral-small".to_string(),
                    0.0002,
                    0.0006,
                    32000,
                ),
                ModelInfo::new(
                    "mistral".to_string(),
                    "codestral".to_string(),
                    0.0003,
                    0.0009,
                    32000,
                ),
            ],
        );

        models.insert(
            "ollama".to_string(),
            vec![ModelInfo::new(
                "ollama".to_string(),
                "local".to_string(),
                0.0,
                0.0,
                128000,
            )],
        );

        Self { models }
    }

    pub fn calculate_cost(
        &self,
        input_tokens: i64,
        output_tokens: i64,
        model_name: &str,
        provider: Option<&str>,
    ) -> RimuruResult<f64> {
        if let Some(model) = self.get_model(model_name, provider) {
            Ok(model.calculate_cost(input_tokens, output_tokens))
        } else {
            let default_model = self.get_default_model();
            Ok(default_model.calculate_cost(input_tokens, output_tokens))
        }
    }

    pub fn get_model(&self, model_name: &str, provider: Option<&str>) -> Option<&ModelInfo> {
        let normalized_name = self.normalize_model_name(model_name);

        if let Some(provider_name) = provider {
            if let Some(provider_models) = self.models.get(provider_name) {
                return provider_models.iter().find(|m| {
                    let model_normalized = self.normalize_model_name(&m.model_name);
                    model_normalized == normalized_name || m.model_name == model_name
                });
            }
        }

        for provider_models in self.models.values() {
            if let Some(model) = provider_models.iter().find(|m| {
                let model_normalized = self.normalize_model_name(&m.model_name);
                model_normalized == normalized_name || m.model_name == model_name
            }) {
                return Some(model);
            }
        }

        None
    }

    pub fn get_default_model(&self) -> &ModelInfo {
        self.models
            .get("anthropic")
            .and_then(|models| models.iter().find(|m| m.model_name.contains("sonnet-4")))
            .or_else(|| {
                self.models
                    .get("anthropic")
                    .and_then(|models| models.first())
            })
            .unwrap_or_else(|| {
                self.models
                    .values()
                    .next()
                    .and_then(|models| models.first())
                    .expect("No models configured")
            })
    }

    pub fn get_supported_models(&self) -> Vec<String> {
        self.models
            .values()
            .flat_map(|provider_models| provider_models.iter().map(|m| m.full_name()))
            .collect()
    }

    pub fn get_supported_providers(&self) -> Vec<String> {
        self.models.keys().cloned().collect()
    }

    pub fn get_models_for_provider(&self, provider: &str) -> Vec<String> {
        self.models
            .get(provider)
            .map(|models| models.iter().map(|m| m.model_name.clone()).collect())
            .unwrap_or_default()
    }

    fn normalize_model_name(&self, name: &str) -> String {
        let lower = name.to_lowercase();

        if lower.contains("opus-4-5") || lower.contains("opus-4.5") {
            return "claude-opus-4-5".to_string();
        }
        if lower.contains("sonnet-4") && !lower.contains("3-5") && !lower.contains("3.5") {
            return "claude-sonnet-4".to_string();
        }
        if lower.contains("3-5-sonnet") || lower.contains("3.5-sonnet") {
            return "claude-3-5-sonnet".to_string();
        }
        if lower.contains("gpt-4o-mini") {
            return "gpt-4o-mini".to_string();
        }
        if lower.contains("gpt-4o") {
            return "gpt-4o".to_string();
        }
        if lower.contains("gpt-4-turbo") {
            return "gpt-4-turbo".to_string();
        }
        if lower.contains("gemini-2") && lower.contains("flash") {
            return "gemini-2.0-flash".to_string();
        }
        if lower.contains("gemini-1.5-pro") {
            return "gemini-1.5-pro".to_string();
        }

        lower
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_calculator_new() {
        let calc = OpenCodeCostCalculator::new();
        assert!(!calc.models.is_empty());
    }

    #[test]
    fn test_calculate_cost_anthropic() {
        let calc = OpenCodeCostCalculator::new();

        let cost = calc
            .calculate_cost(10000, 5000, "claude-3-opus-20240229", Some("anthropic"))
            .unwrap();

        let expected = (10000.0 / 1000.0) * 0.015 + (5000.0 / 1000.0) * 0.075;
        assert!((cost - expected).abs() < 0.0001);
    }

    #[test]
    fn test_calculate_cost_openai() {
        let calc = OpenCodeCostCalculator::new();

        let cost = calc
            .calculate_cost(10000, 5000, "gpt-4o", Some("openai"))
            .unwrap();

        let expected = (10000.0 / 1000.0) * 0.005 + (5000.0 / 1000.0) * 0.015;
        assert!((cost - expected).abs() < 0.0001);
    }

    #[test]
    fn test_calculate_cost_without_provider() {
        let calc = OpenCodeCostCalculator::new();

        let cost = calc.calculate_cost(1000, 500, "gpt-4o", None).unwrap();

        assert!(cost > 0.0);
    }

    #[test]
    fn test_calculate_cost_unknown_model() {
        let calc = OpenCodeCostCalculator::new();

        let cost = calc
            .calculate_cost(1000, 500, "unknown-model", None)
            .unwrap();

        assert!(cost >= 0.0);
    }

    #[test]
    fn test_get_model() {
        let calc = OpenCodeCostCalculator::new();

        let model = calc.get_model("gpt-4o", Some("openai"));
        assert!(model.is_some());
        assert_eq!(model.unwrap().model_name, "gpt-4o");
    }

    #[test]
    fn test_get_supported_models() {
        let calc = OpenCodeCostCalculator::new();

        let models = calc.get_supported_models();
        assert!(!models.is_empty());
        assert!(models.iter().any(|m| m.contains("anthropic")));
        assert!(models.iter().any(|m| m.contains("openai")));
        assert!(models.iter().any(|m| m.contains("google")));
    }

    #[test]
    fn test_get_supported_providers() {
        let calc = OpenCodeCostCalculator::new();

        let providers = calc.get_supported_providers();
        assert!(providers.contains(&"anthropic".to_string()));
        assert!(providers.contains(&"openai".to_string()));
        assert!(providers.contains(&"google".to_string()));
        assert!(providers.contains(&"groq".to_string()));
    }

    #[test]
    fn test_get_models_for_provider() {
        let calc = OpenCodeCostCalculator::new();

        let openai_models = calc.get_models_for_provider("openai");
        assert!(!openai_models.is_empty());
        assert!(openai_models.contains(&"gpt-4o".to_string()));
    }

    #[test]
    fn test_ollama_zero_cost() {
        let calc = OpenCodeCostCalculator::new();

        let cost = calc
            .calculate_cost(100000, 50000, "local", Some("ollama"))
            .unwrap();

        assert_eq!(cost, 0.0);
    }

    #[test]
    fn test_normalize_model_name() {
        let calc = OpenCodeCostCalculator::new();

        assert_eq!(
            calc.normalize_model_name("claude-opus-4-5-20251101"),
            "claude-opus-4-5"
        );
        assert_eq!(calc.normalize_model_name("GPT-4O-MINI"), "gpt-4o-mini");
    }
}
