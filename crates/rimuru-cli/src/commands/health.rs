use anyhow::Result;
use iii_sdk::{III, TriggerRequest};
use serde_json::json;

use crate::output::{self, OutputFormat};

pub async fn check(iii: &III, format: &OutputFormat) -> Result<()> {
    let result = iii
        .trigger(TriggerRequest {
            function_id: "rimuru.health.check".to_string(),
            payload: json!({}),
            action: None,
            timeout_ms: Some(5_000),
        })
        .await?;
    println!("{}", output::format_health(&result, format));

    let status = result
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    if status == "unhealthy" {
        std::process::exit(1);
    }

    Ok(())
}
