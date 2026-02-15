use crate::error::RimuruResult;
use crate::models::ModelInfo;

pub struct CodexCostCalculator {
    models: Vec<ModelInfo>,
}

impl Default for CodexCostCalculator {
    fn default() -> Self {
        Self::new()
    }
}

impl CodexCostCalculator {
    pub fn new() -> Self {
        Self {
            models: vec![
                ModelInfo::new("openai".to_string(), "o3".to_string(), 0.010, 0.040, 200000),
                ModelInfo::new(
                    "openai".to_string(),
                    "o4-mini".to_string(),
                    0.00110,
                    0.00440,
                    200000,
                ),
                ModelInfo::new(
                    "openai".to_string(),
                    "gpt-4.1".to_string(),
                    0.002,
                    0.008,
                    1000000,
                ),
                ModelInfo::new(
                    "openai".to_string(),
                    "gpt-4.1-mini".to_string(),
                    0.0004,
                    0.0016,
                    1000000,
                ),
                ModelInfo::new(
                    "openai".to_string(),
                    "gpt-4.1-nano".to_string(),
                    0.0001,
                    0.0004,
                    1000000,
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
                ModelInfo::new(
                    "openai".to_string(),
                    "o1-preview".to_string(),
                    0.015,
                    0.060,
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
        let normalized_name = self.normalize_model_name(model_name);
        self.models.iter().find(|m| {
            let model_normalized = self.normalize_model_name(&m.model_name);
            model_normalized == normalized_name || m.model_name == model_name
        })
    }

    pub fn get_default_model(&self) -> &ModelInfo {
        self.models
            .iter()
            .find(|m| m.model_name == "o4-mini")
            .unwrap_or(&self.models[0])
    }

    pub fn get_supported_models(&self) -> Vec<String> {
        self.models.iter().map(|m| m.model_name.clone()).collect()
    }

    fn normalize_model_name(&self, name: &str) -> String {
        let lower = name.to_lowercase();

        if lower.contains("o3") && !lower.contains("gpt") {
            return "o3".to_string();
        }
        if lower.contains("o4-mini") {
            return "o4-mini".to_string();
        }
        if lower.contains("gpt-4.1-mini") {
            return "gpt-4.1-mini".to_string();
        }
        if lower.contains("gpt-4.1-nano") {
            return "gpt-4.1-nano".to_string();
        }
        if lower.contains("gpt-4.1") && !lower.contains("mini") && !lower.contains("nano") {
            return "gpt-4.1".to_string();
        }
        if lower.contains("gpt-4o-mini") {
            return "gpt-4o-mini".to_string();
        }
        if lower.contains("gpt-4o") && !lower.contains("mini") {
            return "gpt-4o".to_string();
        }
        if lower.contains("gpt-4-turbo") {
            return "gpt-4-turbo".to_string();
        }
        if lower.contains("gpt-4") && !lower.contains("turbo") && !lower.contains("o") {
            return "gpt-4".to_string();
        }
        if lower.contains("gpt-3.5") {
            return "gpt-3.5-turbo".to_string();
        }
        if lower.contains("o1-mini") {
            return "o1-mini".to_string();
        }
        if lower.contains("o1-preview") {
            return "o1-preview".to_string();
        }
        if lower.contains("o1") && !lower.contains("mini") && !lower.contains("preview") {
            return "o1".to_string();
        }

        lower
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_calculator_new() {
        let calc = CodexCostCalculator::new();
        assert!(!calc.models.is_empty());
    }

    #[test]
    fn test_calculate_cost_gpt4o() {
        let calc = CodexCostCalculator::new();

        let cost = calc.calculate_cost(10000, 5000, "gpt-4o").unwrap();

        let expected = (10000.0 / 1000.0) * 0.0025 + (5000.0 / 1000.0) * 0.010;
        assert!((cost - expected).abs() < 0.0001);
    }

    #[test]
    fn test_calculate_cost_o4_mini() {
        let calc = CodexCostCalculator::new();

        let cost = calc.calculate_cost(10000, 5000, "o4-mini").unwrap();

        let expected = (10000.0 / 1000.0) * 0.00110 + (5000.0 / 1000.0) * 0.00440;
        assert!((cost - expected).abs() < 0.0001);
    }

    #[test]
    fn test_calculate_cost_unknown_model() {
        let calc = CodexCostCalculator::new();

        let cost = calc.calculate_cost(1000, 500, "unknown-model").unwrap();

        assert!(cost > 0.0);
    }

    #[test]
    fn test_get_model() {
        let calc = CodexCostCalculator::new();

        let model = calc.get_model("gpt-4o");
        assert!(model.is_some());
        assert_eq!(model.unwrap().model_name, "gpt-4o");
    }

    #[test]
    fn test_get_supported_models() {
        let calc = CodexCostCalculator::new();

        let models = calc.get_supported_models();
        assert!(!models.is_empty());
        assert!(models.iter().any(|m| m.contains("gpt-4o")));
        assert!(models.iter().any(|m| m.contains("o1")));
    }

    #[test]
    fn test_normalize_model_name() {
        let calc = CodexCostCalculator::new();

        assert_eq!(calc.normalize_model_name("GPT-4o"), "gpt-4o");
        assert_eq!(calc.normalize_model_name("gpt-4o-mini"), "gpt-4o-mini");
        assert_eq!(calc.normalize_model_name("o1-mini"), "o1-mini");
    }
}
