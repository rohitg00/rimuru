use crate::error::RimuruResult;
use crate::models::ModelInfo;

pub struct ClaudeCodeCostCalculator {
    models: Vec<ModelInfo>,
}

impl Default for ClaudeCodeCostCalculator {
    fn default() -> Self {
        Self::new()
    }
}

impl ClaudeCodeCostCalculator {
    pub fn new() -> Self {
        Self {
            models: vec![
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
                ModelInfo::new(
                    "anthropic".to_string(),
                    "claude-3-5-haiku-20241022".to_string(),
                    0.001,
                    0.005,
                    200000,
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
            .find(|m| m.model_name.contains("sonnet-4"))
            .unwrap_or(&self.models[0])
    }

    pub fn get_supported_models(&self) -> Vec<String> {
        self.models.iter().map(|m| m.model_name.clone()).collect()
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
        if lower.contains("3-opus") || lower.contains("3.opus") {
            return "claude-3-opus".to_string();
        }
        if lower.contains("3-sonnet") || lower.contains("3.sonnet") {
            return "claude-3-sonnet".to_string();
        }
        if lower.contains("3-5-haiku") || lower.contains("3.5-haiku") {
            return "claude-3-5-haiku".to_string();
        }
        if lower.contains("3-haiku") || lower.contains("3.haiku") {
            return "claude-3-haiku".to_string();
        }

        lower
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_calculator_new() {
        let calc = ClaudeCodeCostCalculator::new();
        assert!(!calc.models.is_empty());
    }

    #[test]
    fn test_calculate_cost() {
        let calc = ClaudeCodeCostCalculator::new();

        let cost = calc
            .calculate_cost(10000, 5000, "claude-3-opus-20240229")
            .unwrap();

        let expected = (10000.0 / 1000.0) * 0.015 + (5000.0 / 1000.0) * 0.075;
        assert!((cost - expected).abs() < 0.0001);
    }

    #[test]
    fn test_calculate_cost_unknown_model() {
        let calc = ClaudeCodeCostCalculator::new();

        let cost = calc.calculate_cost(1000, 500, "unknown-model").unwrap();

        assert!(cost > 0.0);
    }

    #[test]
    fn test_get_model() {
        let calc = ClaudeCodeCostCalculator::new();

        let model = calc.get_model("claude-3-opus-20240229");
        assert!(model.is_some());
        assert_eq!(model.unwrap().model_name, "claude-3-opus-20240229");
    }

    #[test]
    fn test_get_supported_models() {
        let calc = ClaudeCodeCostCalculator::new();

        let models = calc.get_supported_models();
        assert!(!models.is_empty());
        assert!(models.iter().any(|m| m.contains("opus")));
        assert!(models.iter().any(|m| m.contains("sonnet")));
        assert!(models.iter().any(|m| m.contains("haiku")));
    }

    #[test]
    fn test_normalize_model_name() {
        let calc = ClaudeCodeCostCalculator::new();

        assert_eq!(
            calc.normalize_model_name("claude-opus-4-5-20251101"),
            "claude-opus-4-5"
        );
        assert_eq!(
            calc.normalize_model_name("claude-sonnet-4-20251101"),
            "claude-sonnet-4"
        );
        assert_eq!(
            calc.normalize_model_name("claude-3-5-sonnet-20241022"),
            "claude-3-5-sonnet"
        );
    }
}
