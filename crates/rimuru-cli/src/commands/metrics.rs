use anyhow::Result;
use iii_sdk::{III, TriggerRequest};
use serde_json::json;

use crate::output::{self, OutputFormat};

pub async fn current(iii: &III, format: &OutputFormat) -> Result<()> {
    let result = iii
        .trigger(TriggerRequest {
            function_id: "rimuru.metrics.current".to_string(),
            payload: json!({}),
            action: None,
            timeout_ms: None,
        })
        .await?;
    let metrics = result.get("metrics").unwrap_or(&result);
    println!("{}", output::format_metrics(metrics, format));
    Ok(())
}

pub async fn history(iii: &III, format: &OutputFormat) -> Result<()> {
    let result = iii
        .trigger(TriggerRequest {
            function_id: "rimuru.metrics.history".to_string(),
            payload: json!({}),
            action: None,
            timeout_ms: None,
        })
        .await?;
    let entries = result
        .get("history")
        .and_then(|v| v.get("entries"))
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    println!("{}", output::format_metrics_history(&entries, format));
    Ok(())
}
