use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ModelInfo {
    pub id: Uuid,
    pub provider: String,
    pub model_name: String,
    pub input_price_per_1k: f64,
    pub output_price_per_1k: f64,
    pub context_window: i32,
    pub last_synced: DateTime<Utc>,
}

impl ModelInfo {
    pub fn new(
        provider: String,
        model_name: String,
        input_price_per_1k: f64,
        output_price_per_1k: f64,
        context_window: i32,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            provider,
            model_name,
            input_price_per_1k,
            output_price_per_1k,
            context_window,
            last_synced: Utc::now(),
        }
    }

    pub fn calculate_cost(&self, input_tokens: i64, output_tokens: i64) -> f64 {
        let input_cost = (input_tokens as f64 / 1000.0) * self.input_price_per_1k;
        let output_cost = (output_tokens as f64 / 1000.0) * self.output_price_per_1k;
        input_cost + output_cost
    }

    pub fn full_name(&self) -> String {
        format!("{}/{}", self.provider, self.model_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_info_new() {
        let model = ModelInfo::new(
            "anthropic".to_string(),
            "claude-3-opus".to_string(),
            0.015,
            0.075,
            200000,
        );

        assert_eq!(model.provider, "anthropic");
        assert_eq!(model.model_name, "claude-3-opus");
        assert_eq!(model.input_price_per_1k, 0.015);
        assert_eq!(model.output_price_per_1k, 0.075);
        assert_eq!(model.context_window, 200000);
    }

    #[test]
    fn test_calculate_cost() {
        let model = ModelInfo::new(
            "openai".to_string(),
            "gpt-4".to_string(),
            0.03,
            0.06,
            128000,
        );

        let cost = model.calculate_cost(10000, 5000);
        let expected = (10000.0 / 1000.0) * 0.03 + (5000.0 / 1000.0) * 0.06;

        assert!((cost - expected).abs() < 0.0001);
    }

    #[test]
    fn test_full_name() {
        let model = ModelInfo::new(
            "anthropic".to_string(),
            "claude-3-sonnet".to_string(),
            0.003,
            0.015,
            200000,
        );

        assert_eq!(model.full_name(), "anthropic/claude-3-sonnet");
    }
}
