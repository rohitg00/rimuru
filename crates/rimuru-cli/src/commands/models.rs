use anyhow::Result;
use iii_sdk::III;
use serde_json::json;

use crate::output::{self, OutputFormat};

pub async fn list(iii: &III, format: &OutputFormat) -> Result<()> {
    let result = iii.trigger("rimuru.models.list", json!({})).await?;
    let models = if let Some(arr) = result.get("models").and_then(|v| v.as_array()) {
        arr.clone()
    } else {
        result.as_array().cloned().unwrap_or_default()
    };
    println!("{}", output::format_models_list(&models, format));
    Ok(())
}

pub async fn sync(iii: &III, format: &OutputFormat) -> Result<()> {
    println!("Syncing model pricing data...");
    let result = iii.trigger("rimuru.models.sync", json!({})).await?;
    output::print_value(&result, format);
    Ok(())
}

pub async fn get(iii: &III, model_id: &str, format: &OutputFormat) -> Result<()> {
    let result = iii
        .trigger("rimuru.models.get", json!({"model_id": model_id}))
        .await?;

    if result.is_null() || result.get("error").and_then(|v| v.as_bool()).unwrap_or(false) {
        anyhow::bail!("Model '{}' not found", model_id);
    }

    output::print_value(&result, format);
    Ok(())
}
