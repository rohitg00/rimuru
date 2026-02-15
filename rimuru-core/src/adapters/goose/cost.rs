use crate::error::RimuruResult;
use crate::models::ModelInfo;

pub struct GooseCostCalculator {
    models: Vec<ModelInfo>,
}

impl Default for GooseCostCalculator {
    fn default() -> Self {
        Self::new()
    }
}

impl GooseCostCalculator {
    pub fn new() -> Self {
        Self {
            models: vec![
                ModelInfo::new(
                    "anthropic".to_string(),
                    "claude-3-5-sonnet".to_string(),
                    0.003,
                    0.015,
                    200000,
                ),
                ModelInfo::new(
                    "anthropic".to_string(),
                    "claude-3-opus".to_string(),
                    0.015,
                    0.075,
                    200000,
                ),
                ModelInfo::new(
                    "anthropic".to_string(),
                    "claude-3-sonnet".to_string(),
                    0.003,
                    0.015,
                    200000,
                ),
                ModelInfo::new(
                    "anthropic".to_string(),
                    "claude-3-haiku".to_string(),
                    0.00025,
                    0.00125,
                    200000,
                ),
                ModelInfo::new(
                    "anthropic".to_string(),
                    "claude-3-5-haiku".to_string(),
                    0.0008,
                    0.004,
                    200000,
                ),
                ModelInfo::new(
                    "openai".to_string(),
                    "gpt-4o".to_string(),
                    0.0025,
                    0.010,
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
                ModelInfo::new("openai".to_string(), "o1".to_string(), 0.015, 0.060, 200000),
                ModelInfo::new(
                    "openai".to_string(),
                    "o1-mini".to_string(),
                    0.003,
                    0.012,
                    128000,
                ),
                ModelInfo::new("openai".to_string(), "o3".to_string(), 0.010, 0.040, 200000),
                ModelInfo::new(
                    "openai".to_string(),
                    "o4-mini".to_string(),
                    0.00110,
                    0.00440,
                    200000,
                ),
                ModelInfo::new(
                    "google".to_string(),
                    "gemini-2.0-flash".to_string(),
                    0.00010,
                    0.00040,
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
                    128000,
                ),
                ModelInfo::new(
                    "mistral".to_string(),
                    "codestral".to_string(),
                    0.0002,
                    0.0006,
                    256000,
                ),
                ModelInfo::new(
                    "groq".to_string(),
                    "llama-3.3-70b".to_string(),
                    0.00059,
                    0.00079,
                    131072,
                ),
                ModelInfo::new(
                    "groq".to_string(),
                    "llama-3.1-8b".to_string(),
                    0.00005,
                    0.00008,
                    131072,
                ),
                ModelInfo::new(
                    "groq".to_string(),
                    "mixtral-8x7b".to_string(),
                    0.00024,
                    0.00024,
                    32768,
                ),
                ModelInfo::new("ollama".to_string(), "local".to_string(), 0.0, 0.0, 128000),
                ModelInfo::new("ollama".to_string(), "llama3".to_string(), 0.0, 0.0, 128000),
                ModelInfo::new(
                    "ollama".to_string(),
                    "codellama".to_string(),
                    0.0,
                    0.0,
                    128000,
                ),
                ModelInfo::new(
                    "deepseek".to_string(),
                    "deepseek-chat".to_string(),
                    0.00014,
                    0.00028,
                    128000,
                ),
                ModelInfo::new(
                    "deepseek".to_string(),
                    "deepseek-coder".to_string(),
                    0.00014,
                    0.00028,
                    128000,
                ),
            ],
        }
    }

    pub fn calculate_cost(
        &self,
        input_tokens: i64,
        output_tokens: i64,
        model_name: &str,
    ) -> RimuruResult<f64> {
        if let Some(model) = self.get_model(model_name) {
            Ok(model.calculate_cost(input_tokens, output_tokens))
        } else {
            let default_model = self.get_default_model();
            Ok(default_model.calculate_cost(input_tokens, output_tokens))
        }
    }

    pub fn get_model(&self, model_name: &str) -> Option<&ModelInfo> {
        let (provider, model) = self.parse_model_name(model_name);

        self.models.iter().find(|m| {
            let model_match = m.model_name.to_lowercase() == model.to_lowercase()
                || m.model_name.to_lowercase().contains(&model.to_lowercase())
                || model.to_lowercase().contains(&m.model_name.to_lowercase());

            if let Some(ref p) = provider {
                model_match && m.provider.to_lowercase() == p.to_lowercase()
            } else {
                model_match
            }
        })
    }

    pub fn get_default_model(&self) -> &ModelInfo {
        self.models
            .iter()
            .find(|m| m.model_name == "claude-3-5-sonnet")
            .unwrap_or(&self.models[0])
    }

    pub fn get_supported_models(&self) -> Vec<String> {
        self.models
            .iter()
            .map(|m| format!("{}/{}", m.provider, m.model_name))
            .collect()
    }

    pub fn get_providers(&self) -> Vec<String> {
        let mut providers: Vec<String> = self.models.iter().map(|m| m.provider.clone()).collect();
        providers.sort();
        providers.dedup();
        providers
    }

    fn parse_model_name(&self, name: &str) -> (Option<String>, String) {
        if let Some(idx) = name.find('/') {
            let provider = name[..idx].to_string();
            let model = name[idx + 1..].to_string();
            (Some(provider), model)
        } else {
            (None, name.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_calculator_new() {
        let calc = GooseCostCalculator::new();
        assert!(!calc.models.is_empty());
    }

    #[test]
    fn test_calculate_cost_anthropic() {
        let calc = GooseCostCalculator::new();

        let cost = calc
            .calculate_cost(10000, 5000, "claude-3-5-sonnet")
            .unwrap();

        let expected = (10000.0 / 1000.0) * 0.003 + (5000.0 / 1000.0) * 0.015;
        assert!((cost - expected).abs() < 0.0001);
    }

    #[test]
    fn test_calculate_cost_openai() {
        let calc = GooseCostCalculator::new();

        let cost = calc.calculate_cost(10000, 5000, "gpt-4o").unwrap();

        let expected = (10000.0 / 1000.0) * 0.0025 + (5000.0 / 1000.0) * 0.010;
        assert!((cost - expected).abs() < 0.0001);
    }

    #[test]
    fn test_calculate_cost_with_provider_prefix() {
        let calc = GooseCostCalculator::new();

        let cost = calc
            .calculate_cost(10000, 5000, "anthropic/claude-3-5-sonnet")
            .unwrap();

        let expected = (10000.0 / 1000.0) * 0.003 + (5000.0 / 1000.0) * 0.015;
        assert!((cost - expected).abs() < 0.0001);
    }

    #[test]
    fn test_calculate_cost_ollama_free() {
        let calc = GooseCostCalculator::new();

        let cost = calc.calculate_cost(10000, 5000, "ollama/local").unwrap();
        assert_eq!(cost, 0.0);
    }

    #[test]
    fn test_calculate_cost_unknown_model() {
        let calc = GooseCostCalculator::new();

        let cost = calc.calculate_cost(1000, 500, "unknown-model").unwrap();

        assert!(cost > 0.0);
    }

    #[test]
    fn test_get_model() {
        let calc = GooseCostCalculator::new();

        let model = calc.get_model("gpt-4o");
        assert!(model.is_some());
        assert_eq!(model.unwrap().model_name, "gpt-4o");

        let model = calc.get_model("openai/gpt-4o");
        assert!(model.is_some());
        assert_eq!(model.unwrap().provider, "openai");
    }

    #[test]
    fn test_get_supported_models() {
        let calc = GooseCostCalculator::new();

        let models = calc.get_supported_models();
        assert!(!models.is_empty());
        assert!(models.iter().any(|m| m.contains("claude")));
        assert!(models.iter().any(|m| m.contains("gpt")));
        assert!(models.iter().any(|m| m.contains("gemini")));
    }

    #[test]
    fn test_get_providers() {
        let calc = GooseCostCalculator::new();

        let providers = calc.get_providers();
        assert!(!providers.is_empty());
        assert!(providers.contains(&"anthropic".to_string()));
        assert!(providers.contains(&"openai".to_string()));
        assert!(providers.contains(&"google".to_string()));
        assert!(providers.contains(&"ollama".to_string()));
    }

    #[test]
    fn test_parse_model_name() {
        let calc = GooseCostCalculator::new();

        let (provider, model) = calc.parse_model_name("anthropic/claude-3-5-sonnet");
        assert_eq!(provider, Some("anthropic".to_string()));
        assert_eq!(model, "claude-3-5-sonnet");

        let (provider, model) = calc.parse_model_name("gpt-4o");
        assert!(provider.is_none());
        assert_eq!(model, "gpt-4o");
    }
}
