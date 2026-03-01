use anyhow::Result;
use iii_sdk::III;
use serde_json::{json, Value};

use crate::output::{self, OutputFormat};

pub async fn summary(iii: &III, format: &OutputFormat) -> Result<()> {
    let result = iii.trigger("rimuru.costs.summary", json!({})).await?;
    let summary = result.get("summary").unwrap_or(&result);
    println!("{}", output::format_costs_summary(summary, format));
    Ok(())
}

pub async fn daily(iii: &III, format: &OutputFormat) -> Result<()> {
    let result = iii.trigger("rimuru.costs.daily", json!({})).await?;
    let entries = result
        .get("daily")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    println!("{}", output::format_daily_costs(&entries, format));
    Ok(())
}

pub async fn agent(iii: &III, agent_id: Option<&str>, format: &OutputFormat) -> Result<()> {
    let input = if let Some(id) = agent_id {
        json!({"agent_id": id})
    } else {
        json!({})
    };
    let result = iii.trigger("rimuru.costs.by_agent", input).await?;
    let agents = match result {
        Value::Array(arr) => arr,
        other => vec![other],
    };
    println!("{}", output::format_daily_costs(&agents, format));
    Ok(())
}

pub async fn export(iii: &III, path: &str) -> Result<()> {
    let result = iii.trigger("rimuru.costs.summary", json!({})).await?;
    let content = serde_json::to_string_pretty(&result)?;
    std::fs::write(path, &content)?;
    println!("Exported costs to {path}");
    Ok(())
}
