use anyhow::Result;
use iii_sdk::III;
use serde_json::json;

use crate::output::{self, OutputFormat};

pub async fn list(iii: &III, format: &OutputFormat) -> Result<()> {
    let result = iii.trigger("rimuru.mcp.list", json!({})).await?;
    let servers = if let Some(arr) = result.get("servers").and_then(|v| v.as_array()) {
        arr.clone()
    } else {
        result.as_array().cloned().unwrap_or_default()
    };
    println!("{}", output::format_mcp_list(&servers, format));
    Ok(())
}
