use anyhow::Result;
use iii_sdk::III;
use serde_json::json;

use crate::output::{self, OutputFormat};

pub async fn check(iii: &III, format: &OutputFormat) -> Result<()> {
    let result = iii.trigger("rimuru.health.check", json!({})).await?;
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
