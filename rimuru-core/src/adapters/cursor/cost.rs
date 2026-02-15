use crate::error::RimuruResult;
use crate::models::ModelInfo;

use super::config::CursorTier;

pub struct CursorCostCalculator {
    models: Vec<ModelInfo>,
    tier: CursorTier,
}

impl Default for CursorCostCalculator {
    fn default() -> Self {
        Self::new()
    }
}

impl CursorCostCalculator {
    pub fn new() -> Self {
        Self::with_tier(CursorTier::Free)
    }

    pub fn with_tier(tier: CursorTier) -> Self {
        Self {
            models: Self::create_model_list(),
            tier,
        }
    }

    fn create_model_list() -> Vec<ModelInfo> {
        vec![
            ModelInfo::new(
                "cursor".to_string(),
                "gpt-4o".to_string(),
                0.0025,
                0.010,
                128000,
            ),
            ModelInfo::new(
                "cursor".to_string(),
                "gpt-4o-mini".to_string(),
                0.00015,
                0.0006,
                128000,
            ),
            ModelInfo::new(
                "cursor".to_string(),
                "gpt-4-turbo".to_string(),
                0.01,
                0.03,
                128000,
            ),
            ModelInfo::new("cursor".to_string(), "gpt-4".to_string(), 0.03, 0.06, 8192),
            ModelInfo::new(
                "cursor".to_string(),
                "gpt-3.5-turbo".to_string(),
                0.0005,
                0.0015,
                16385,
            ),
            ModelInfo::new("cursor".to_string(), "o1".to_string(), 0.015, 0.060, 200000),
            ModelInfo::new(
                "cursor".to_string(),
                "o1-mini".to_string(),
                0.003,
                0.012,
                128000,
            ),
            ModelInfo::new("cursor".to_string(), "o3".to_string(), 0.010, 0.040, 200000),
            ModelInfo::new(
                "cursor".to_string(),
                "o3-mini".to_string(),
                0.00110,
                0.00440,
                200000,
            ),
            ModelInfo::new(
                "cursor".to_string(),
                "claude-3-5-sonnet".to_string(),
                0.003,
                0.015,
                200000,
            ),
            ModelInfo::new(
                "cursor".to_string(),
                "claude-3-opus".to_string(),
                0.015,
                0.075,
                200000,
            ),
            ModelInfo::new(
                "cursor".to_string(),
                "claude-3-sonnet".to_string(),
                0.003,
                0.015,
                200000,
            ),
            ModelInfo::new(
                "cursor".to_string(),
                "claude-3-haiku".to_string(),
                0.00025,
                0.00125,
                200000,
            ),
            ModelInfo::new(
                "cursor".to_string(),
                "gemini-2.0-flash".to_string(),
                0.00010,
                0.00040,
                1000000,
            ),
            ModelInfo::new(
                "cursor".to_string(),
                "gemini-1.5-pro".to_string(),
                0.00125,
                0.005,
                2000000,
            ),
            ModelInfo::new(
                "cursor".to_string(),
                "cursor-small".to_string(),
                0.0001,
                0.0003,
                32000,
            ),
            ModelInfo::new(
                "cursor".to_string(),
                "cursor-fast".to_string(),
                0.00005,
                0.0002,
                16000,
            ),
        ]
    }

    pub fn set_tier(&mut self, tier: CursorTier) {
        self.tier = tier;
    }

    pub fn get_tier(&self) -> CursorTier {
        self.tier
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

    pub fn calculate_subscription_cost(&self, month_fraction: f64) -> f64 {
        self.tier.monthly_cost() * month_fraction
    }

    pub fn calculate_total_monthly_cost(
        &self,
        input_tokens: i64,
        output_tokens: i64,
        model_name: &str,
        requests: i64,
    ) -> RimuruResult<f64> {
        let subscription_cost = self.tier.monthly_cost();

        let api_cost = match self.tier {
            CursorTier::Free => self.calculate_cost(input_tokens, output_tokens, model_name)?,
            CursorTier::Pro | CursorTier::Business => {
                let included = self.tier.requests_included().unwrap_or(0);
                let overage = (requests - included).max(0);

                if overage > 0 {
                    let overage_cost =
                        self.calculate_cost(input_tokens, output_tokens, model_name)?;
                    overage_cost * (overage as f64 / requests.max(1) as f64)
                } else {
                    0.0
                }
            }
        };

        Ok(subscription_cost + api_cost)
    }

    pub fn is_premium_model(&self, model_name: &str) -> bool {
        let premium_models = [
            "gpt-4o",
            "gpt-4",
            "gpt-4-turbo",
            "o1",
            "o1-mini",
            "o3",
            "o3-mini",
            "claude-3-5-sonnet",
            "claude-3-opus",
            "claude-3-sonnet",
            "gemini-1.5-pro",
        ];

        let normalized = self.normalize_model_name(model_name);
        premium_models.iter().any(|m| normalized.contains(m))
    }

    fn normalize_model_name(&self, name: &str) -> String {
        let name = name.to_lowercase();
        if let Some(idx) = name.find('/') {
            name[idx + 1..].to_string()
        } else {
            name
        }
    }

    pub fn get_model(&self, model_name: &str) -> Option<&ModelInfo> {
        let normalized = self.normalize_model_name(model_name);

        self.models.iter().find(|m| {
            let model_normalized = m.model_name.to_lowercase();
            model_normalized == normalized
                || model_normalized.contains(&normalized)
                || normalized.contains(&model_normalized)
        })
    }

    pub fn get_default_model(&self) -> &ModelInfo {
        self.models
            .iter()
            .find(|m| m.model_name == "gpt-4o")
            .unwrap_or(&self.models[0])
    }

    pub fn get_supported_models(&self) -> Vec<String> {
        self.models
            .iter()
            .map(|m| format!("{}/{}", m.provider, m.model_name))
            .collect()
    }

    pub fn get_models_by_tier(&self, tier: CursorTier) -> Vec<String> {
        match tier {
            CursorTier::Free => {
                vec![
                    "cursor-small".to_string(),
                    "cursor-fast".to_string(),
                    "gpt-3.5-turbo".to_string(),
                ]
            }
            CursorTier::Pro | CursorTier::Business => {
                self.models.iter().map(|m| m.model_name.clone()).collect()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_calculator_new() {
        let calc = CursorCostCalculator::new();
        assert!(!calc.models.is_empty());
        assert_eq!(calc.tier, CursorTier::Free);
    }

    #[test]
    fn test_cost_calculator_with_tier() {
        let calc = CursorCostCalculator::with_tier(CursorTier::Pro);
        assert_eq!(calc.tier, CursorTier::Pro);
    }

    #[test]
    fn test_calculate_cost_gpt4o() {
        let calc = CursorCostCalculator::new();

        let cost = calc.calculate_cost(10000, 5000, "gpt-4o").unwrap();

        let expected = (10000.0 / 1000.0) * 0.0025 + (5000.0 / 1000.0) * 0.010;
        assert!((cost - expected).abs() < 0.0001);
    }

    #[test]
    fn test_calculate_cost_claude() {
        let calc = CursorCostCalculator::new();

        let cost = calc
            .calculate_cost(10000, 5000, "claude-3-5-sonnet")
            .unwrap();

        let expected = (10000.0 / 1000.0) * 0.003 + (5000.0 / 1000.0) * 0.015;
        assert!((cost - expected).abs() < 0.0001);
    }

    #[test]
    fn test_calculate_cost_unknown_model() {
        let calc = CursorCostCalculator::new();

        let cost = calc.calculate_cost(1000, 500, "unknown-model").unwrap();

        assert!(cost > 0.0);
    }

    #[test]
    fn test_subscription_cost() {
        let calc = CursorCostCalculator::with_tier(CursorTier::Pro);

        let full_month = calc.calculate_subscription_cost(1.0);
        assert_eq!(full_month, 20.0);

        let half_month = calc.calculate_subscription_cost(0.5);
        assert_eq!(half_month, 10.0);
    }

    #[test]
    fn test_is_premium_model() {
        let calc = CursorCostCalculator::new();

        assert!(calc.is_premium_model("gpt-4o"));
        assert!(calc.is_premium_model("cursor/gpt-4o"));
        assert!(calc.is_premium_model("claude-3-5-sonnet"));
        assert!(calc.is_premium_model("o1"));
        assert!(!calc.is_premium_model("cursor-small"));
        assert!(!calc.is_premium_model("cursor-fast"));
    }

    #[test]
    fn test_get_model() {
        let calc = CursorCostCalculator::new();

        let model = calc.get_model("gpt-4o");
        assert!(model.is_some());
        assert_eq!(model.unwrap().model_name, "gpt-4o");

        let model = calc.get_model("cursor/gpt-4o");
        assert!(model.is_some());
    }

    #[test]
    fn test_get_supported_models() {
        let calc = CursorCostCalculator::new();

        let models = calc.get_supported_models();
        assert!(!models.is_empty());
        assert!(models.iter().any(|m| m.contains("gpt-4o")));
        assert!(models.iter().any(|m| m.contains("claude")));
    }

    #[test]
    fn test_get_models_by_tier() {
        let calc = CursorCostCalculator::new();

        let free_models = calc.get_models_by_tier(CursorTier::Free);
        assert!(free_models.len() <= 5);
        assert!(free_models.contains(&"cursor-small".to_string()));

        let pro_models = calc.get_models_by_tier(CursorTier::Pro);
        assert!(pro_models.len() > free_models.len());
    }

    #[test]
    fn test_set_tier() {
        let mut calc = CursorCostCalculator::new();
        assert_eq!(calc.get_tier(), CursorTier::Free);

        calc.set_tier(CursorTier::Business);
        assert_eq!(calc.get_tier(), CursorTier::Business);
    }

    #[test]
    fn test_total_monthly_cost_free_tier() {
        let calc = CursorCostCalculator::with_tier(CursorTier::Free);

        let total = calc
            .calculate_total_monthly_cost(10000, 5000, "gpt-4o", 100)
            .unwrap();

        let api_cost = calc.calculate_cost(10000, 5000, "gpt-4o").unwrap();
        assert!((total - api_cost).abs() < 0.0001);
    }

    #[test]
    fn test_total_monthly_cost_pro_tier_within_limits() {
        let calc = CursorCostCalculator::with_tier(CursorTier::Pro);

        let total = calc
            .calculate_total_monthly_cost(10000, 5000, "gpt-4o", 100)
            .unwrap();

        assert_eq!(total, 20.0);
    }
}
