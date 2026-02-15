use std::collections::HashMap;
use tracing::{debug, warn};

use crate::models::ModelInfo;

use super::types::ModelSource;

pub struct ModelAggregator {
    conflict_resolution: ConflictResolution,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConflictResolution {
    #[default]
    OfficialFirst,
    MostRecent,
    LowestPrice,
    HighestContextWindow,
}

impl ModelAggregator {
    pub fn new() -> Self {
        Self {
            conflict_resolution: ConflictResolution::default(),
        }
    }

    pub fn with_resolution(resolution: ConflictResolution) -> Self {
        Self {
            conflict_resolution: resolution,
        }
    }

    pub fn merge_models(&self, sources: Vec<(ModelSource, Vec<ModelInfo>)>) -> Vec<ModelInfo> {
        let mut model_map: HashMap<String, (ModelInfo, ModelSource)> = HashMap::new();

        for (source, models) in sources {
            for model in models {
                let key = format!(
                    "{}/{}",
                    model.provider.to_lowercase(),
                    model.model_name.to_lowercase()
                );

                if let Some((existing_model, existing_source)) = model_map.get(&key) {
                    let should_replace =
                        self.should_replace(existing_model, *existing_source, &model, source);

                    if should_replace {
                        debug!(
                            "Replacing model {} from {:?} with {:?}",
                            key, existing_source, source
                        );
                        model_map.insert(key, (model, source));
                    }
                } else {
                    model_map.insert(key, (model, source));
                }
            }
        }

        model_map.into_values().map(|(model, _)| model).collect()
    }

    fn should_replace(
        &self,
        existing: &ModelInfo,
        existing_source: ModelSource,
        new: &ModelInfo,
        new_source: ModelSource,
    ) -> bool {
        match self.conflict_resolution {
            ConflictResolution::OfficialFirst => new_source.priority() < existing_source.priority(),
            ConflictResolution::MostRecent => new.last_synced > existing.last_synced,
            ConflictResolution::LowestPrice => {
                let new_total = new.input_price_per_1k + new.output_price_per_1k;
                let existing_total = existing.input_price_per_1k + existing.output_price_per_1k;
                new_total < existing_total
            }
            ConflictResolution::HighestContextWindow => {
                new.context_window > existing.context_window
            }
        }
    }

    pub fn deduplicate_models(&self, models: Vec<ModelInfo>) -> Vec<ModelInfo> {
        let mut seen: HashMap<String, ModelInfo> = HashMap::new();

        for model in models {
            let key = format!(
                "{}/{}",
                model.provider.to_lowercase(),
                model.model_name.to_lowercase()
            );

            if let Some(existing) = seen.get(&key) {
                if model.last_synced > existing.last_synced {
                    seen.insert(key, model);
                }
            } else {
                seen.insert(key, model);
            }
        }

        seen.into_values().collect()
    }

    pub fn normalize_provider_name(provider: &str) -> String {
        match provider.to_lowercase().as_str() {
            "anthropic" | "claude" => "anthropic".to_string(),
            "openai" | "gpt" => "openai".to_string(),
            "google" | "gemini" | "vertex" => "google".to_string(),
            "meta" | "llama" => "meta".to_string(),
            "mistral" | "mistralai" => "mistral".to_string(),
            "cohere" => "cohere".to_string(),
            other => other.to_lowercase(),
        }
    }

    pub fn normalize_model_name(model_name: &str) -> String {
        model_name.to_lowercase().replace([' ', '_'], "-")
    }

    pub fn validate_pricing(model: &ModelInfo) -> bool {
        if model.input_price_per_1k < 0.0 {
            warn!(
                "Invalid negative input price for {}/{}: {}",
                model.provider, model.model_name, model.input_price_per_1k
            );
            return false;
        }

        if model.output_price_per_1k < 0.0 {
            warn!(
                "Invalid negative output price for {}/{}: {}",
                model.provider, model.model_name, model.output_price_per_1k
            );
            return false;
        }

        if model.context_window <= 0 {
            warn!(
                "Invalid context window for {}/{}: {}",
                model.provider, model.model_name, model.context_window
            );
            return false;
        }

        if model.input_price_per_1k > 1000.0 || model.output_price_per_1k > 1000.0 {
            warn!(
                "Suspiciously high pricing for {}/{}: input={}, output={}",
                model.provider,
                model.model_name,
                model.input_price_per_1k,
                model.output_price_per_1k
            );
        }

        true
    }

    pub fn filter_valid_models(&self, models: Vec<ModelInfo>) -> Vec<ModelInfo> {
        models.into_iter().filter(Self::validate_pricing).collect()
    }
}

impl Default for ModelAggregator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_model(
        provider: &str,
        name: &str,
        input_price: f64,
        output_price: f64,
    ) -> ModelInfo {
        ModelInfo::new(
            provider.to_string(),
            name.to_string(),
            input_price,
            output_price,
            128000,
        )
    }

    #[test]
    fn test_merge_models_official_first() {
        let aggregator = ModelAggregator::new();

        let official_models = vec![create_test_model(
            "anthropic",
            "claude-3-opus",
            0.015,
            0.075,
        )];

        let community_models = vec![create_test_model(
            "anthropic",
            "claude-3-opus",
            0.020,
            0.080,
        )];

        let merged = aggregator.merge_models(vec![
            (ModelSource::OfficialApi, official_models),
            (ModelSource::OpenRouter, community_models),
        ]);

        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].input_price_per_1k, 0.015);
    }

    #[test]
    fn test_merge_models_community_wins_when_no_official() {
        let aggregator = ModelAggregator::new();

        let community_models = vec![create_test_model("openai", "gpt-4", 0.03, 0.06)];

        let merged = aggregator.merge_models(vec![(ModelSource::OpenRouter, community_models)]);

        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].provider, "openai");
    }

    #[test]
    fn test_deduplicate_models() {
        let aggregator = ModelAggregator::new();

        let mut model1 = create_test_model("anthropic", "claude-3-opus", 0.015, 0.075);
        let mut model2 = create_test_model("anthropic", "claude-3-opus", 0.016, 0.076);

        model2.last_synced = model1.last_synced + chrono::Duration::hours(1);

        let deduped = aggregator.deduplicate_models(vec![model1, model2]);

        assert_eq!(deduped.len(), 1);
        assert_eq!(deduped[0].input_price_per_1k, 0.016);
    }

    #[test]
    fn test_normalize_provider_name() {
        assert_eq!(
            ModelAggregator::normalize_provider_name("Anthropic"),
            "anthropic"
        );
        assert_eq!(ModelAggregator::normalize_provider_name("OPENAI"), "openai");
        assert_eq!(ModelAggregator::normalize_provider_name("Google"), "google");
        assert_eq!(ModelAggregator::normalize_provider_name("gemini"), "google");
        assert_eq!(ModelAggregator::normalize_provider_name("vertex"), "google");
        assert_eq!(
            ModelAggregator::normalize_provider_name("Claude"),
            "anthropic"
        );
        assert_eq!(ModelAggregator::normalize_provider_name("GPT"), "openai");
        assert_eq!(
            ModelAggregator::normalize_provider_name("Unknown"),
            "unknown"
        );
    }

    #[test]
    fn test_normalize_model_name() {
        assert_eq!(
            ModelAggregator::normalize_model_name("Claude 3 Opus"),
            "claude-3-opus"
        );
        assert_eq!(
            ModelAggregator::normalize_model_name("GPT_4_Turbo"),
            "gpt-4-turbo"
        );
        assert_eq!(ModelAggregator::normalize_model_name("gpt-4o"), "gpt-4o");
    }

    #[test]
    fn test_validate_pricing_valid() {
        let model = create_test_model("anthropic", "claude-3-opus", 0.015, 0.075);
        assert!(ModelAggregator::validate_pricing(&model));
    }

    #[test]
    fn test_validate_pricing_negative_input() {
        let model = create_test_model("test", "model", -0.01, 0.03);
        assert!(!ModelAggregator::validate_pricing(&model));
    }

    #[test]
    fn test_validate_pricing_negative_output() {
        let model = create_test_model("test", "model", 0.01, -0.03);
        assert!(!ModelAggregator::validate_pricing(&model));
    }

    #[test]
    fn test_validate_pricing_zero_context() {
        let mut model = create_test_model("test", "model", 0.01, 0.03);
        model.context_window = 0;
        assert!(!ModelAggregator::validate_pricing(&model));
    }

    #[test]
    fn test_filter_valid_models() {
        let aggregator = ModelAggregator::new();

        let valid_model = create_test_model("anthropic", "claude", 0.01, 0.03);
        let mut invalid_model = create_test_model("test", "bad", -0.01, 0.03);

        let filtered = aggregator.filter_valid_models(vec![valid_model.clone(), invalid_model]);

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].provider, "anthropic");
    }

    #[test]
    fn test_conflict_resolution_most_recent() {
        let aggregator = ModelAggregator::with_resolution(ConflictResolution::MostRecent);

        let mut older = create_test_model("anthropic", "claude-3-opus", 0.015, 0.075);
        let mut newer = create_test_model("anthropic", "claude-3-opus", 0.020, 0.080);

        newer.last_synced = older.last_synced + chrono::Duration::hours(1);

        let merged = aggregator.merge_models(vec![
            (ModelSource::OfficialApi, vec![older]),
            (ModelSource::OpenRouter, vec![newer]),
        ]);

        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].input_price_per_1k, 0.020);
    }

    #[test]
    fn test_conflict_resolution_lowest_price() {
        let aggregator = ModelAggregator::with_resolution(ConflictResolution::LowestPrice);

        let expensive = create_test_model("anthropic", "claude-3-opus", 0.020, 0.080);
        let cheaper = create_test_model("anthropic", "claude-3-opus", 0.015, 0.075);

        let merged = aggregator.merge_models(vec![
            (ModelSource::OfficialApi, vec![expensive]),
            (ModelSource::OpenRouter, vec![cheaper]),
        ]);

        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].input_price_per_1k, 0.015);
    }
}
