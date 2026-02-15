use crate::error::RimuruResult;
use crate::models::ModelInfo;

use super::config::CopilotProduct;

pub struct CopilotCostCalculator {
    product: CopilotProduct,
    models: Vec<ModelInfo>,
}

impl Default for CopilotCostCalculator {
    fn default() -> Self {
        Self::new(CopilotProduct::Individual)
    }
}

impl CopilotCostCalculator {
    pub fn new(product: CopilotProduct) -> Self {
        Self {
            product,
            models: vec![
                ModelInfo::new(
                    "github".to_string(),
                    "copilot-gpt-4".to_string(),
                    0.0,
                    0.0,
                    8192,
                ),
                ModelInfo::new(
                    "github".to_string(),
                    "copilot-gpt-3.5-turbo".to_string(),
                    0.0,
                    0.0,
                    4096,
                ),
                ModelInfo::new(
                    "github".to_string(),
                    "copilot-claude-3.5-sonnet".to_string(),
                    0.0,
                    0.0,
                    200000,
                ),
                ModelInfo::new(
                    "github".to_string(),
                    "copilot-gemini-1.5-pro".to_string(),
                    0.0,
                    0.0,
                    1000000,
                ),
            ],
        }
    }

    pub fn with_product(mut self, product: CopilotProduct) -> Self {
        self.product = product;
        self
    }

    pub fn monthly_subscription_cost(&self) -> f64 {
        match self.product {
            CopilotProduct::Individual => 10.0,
            CopilotProduct::Business => 19.0,
            CopilotProduct::Enterprise => 39.0,
        }
    }

    pub fn daily_rate(&self) -> f64 {
        self.monthly_subscription_cost() / 30.0
    }

    pub fn calculate_cost(
        &self,
        _input_tokens: i64,
        _output_tokens: i64,
        _model_name: &str,
    ) -> RimuruResult<f64> {
        Ok(0.0)
    }

    pub fn estimate_usage_cost(&self, days: u32) -> f64 {
        self.daily_rate() * days as f64
    }

    pub fn calculate_prorated_cost(&self, days_used: u32, days_in_month: u32) -> f64 {
        let monthly_cost = self.monthly_subscription_cost();
        (monthly_cost / days_in_month as f64) * days_used as f64
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
            .find(|m| m.model_name == "copilot-gpt-4")
            .unwrap_or(&self.models[0])
    }

    pub fn get_supported_models(&self) -> Vec<String> {
        self.models.iter().map(|m| m.model_name.clone()).collect()
    }

    fn normalize_model_name(&self, name: &str) -> String {
        let lower = name.to_lowercase();

        if lower.contains("copilot") && lower.contains("claude") {
            return "copilot-claude-3.5-sonnet".to_string();
        }
        if lower.contains("copilot") && lower.contains("gemini") {
            return "copilot-gemini-1.5-pro".to_string();
        }
        if lower.contains("copilot") && lower.contains("gpt-4") {
            return "copilot-gpt-4".to_string();
        }
        if lower.contains("copilot") && lower.contains("3.5") {
            return "copilot-gpt-3.5-turbo".to_string();
        }
        if lower.contains("gpt-4") {
            return "copilot-gpt-4".to_string();
        }
        if lower.contains("gpt-3.5") {
            return "copilot-gpt-3.5-turbo".to_string();
        }

        lower
    }

    pub fn get_product(&self) -> CopilotProduct {
        self.product
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_calculator_new() {
        let calc = CopilotCostCalculator::new(CopilotProduct::Individual);
        assert!(!calc.models.is_empty());
        assert_eq!(calc.product, CopilotProduct::Individual);
    }

    #[test]
    fn test_monthly_subscription_cost() {
        let individual = CopilotCostCalculator::new(CopilotProduct::Individual);
        assert_eq!(individual.monthly_subscription_cost(), 10.0);

        let business = CopilotCostCalculator::new(CopilotProduct::Business);
        assert_eq!(business.monthly_subscription_cost(), 19.0);

        let enterprise = CopilotCostCalculator::new(CopilotProduct::Enterprise);
        assert_eq!(enterprise.monthly_subscription_cost(), 39.0);
    }

    #[test]
    fn test_daily_rate() {
        let calc = CopilotCostCalculator::new(CopilotProduct::Individual);
        let daily = calc.daily_rate();

        assert!((daily - (10.0 / 30.0)).abs() < 0.0001);
    }

    #[test]
    fn test_calculate_cost_subscription_based() {
        let calc = CopilotCostCalculator::new(CopilotProduct::Individual);

        let cost = calc.calculate_cost(10000, 5000, "copilot-gpt-4").unwrap();
        assert_eq!(cost, 0.0);
    }

    #[test]
    fn test_estimate_usage_cost() {
        let calc = CopilotCostCalculator::new(CopilotProduct::Individual);

        let cost_30_days = calc.estimate_usage_cost(30);
        assert!((cost_30_days - 10.0).abs() < 0.0001);

        let cost_15_days = calc.estimate_usage_cost(15);
        assert!((cost_15_days - 5.0).abs() < 0.0001);
    }

    #[test]
    fn test_calculate_prorated_cost() {
        let calc = CopilotCostCalculator::new(CopilotProduct::Individual);

        let prorated = calc.calculate_prorated_cost(15, 30);
        assert!((prorated - 5.0).abs() < 0.0001);
    }

    #[test]
    fn test_get_model() {
        let calc = CopilotCostCalculator::new(CopilotProduct::Individual);

        let model = calc.get_model("copilot-gpt-4");
        assert!(model.is_some());
        assert_eq!(model.unwrap().model_name, "copilot-gpt-4");
    }

    #[test]
    fn test_get_supported_models() {
        let calc = CopilotCostCalculator::new(CopilotProduct::Individual);

        let models = calc.get_supported_models();
        assert!(!models.is_empty());
        assert!(models.iter().any(|m| m.contains("copilot-gpt-4")));
    }

    #[test]
    fn test_normalize_model_name() {
        let calc = CopilotCostCalculator::new(CopilotProduct::Individual);

        assert_eq!(calc.normalize_model_name("GPT-4"), "copilot-gpt-4");
        assert_eq!(
            calc.normalize_model_name("copilot-gpt-3.5-turbo"),
            "copilot-gpt-3.5-turbo"
        );
    }

    #[test]
    fn test_with_product() {
        let calc = CopilotCostCalculator::default().with_product(CopilotProduct::Enterprise);
        assert_eq!(calc.get_product(), CopilotProduct::Enterprise);
    }
}
