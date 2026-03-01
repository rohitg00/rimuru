use anyhow::Result;
use iii_sdk::III;
use serde_json::json;

use crate::output::{self, OutputFormat};

pub async fn list(iii: &III, format: &OutputFormat) -> Result<()> {
    let result = iii.trigger("rimuru.agents.list", json!({})).await?;
    let agents = result
        .get("agents")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    println!("{}", output::format_agents_list(&agents, format));
    Ok(())
}

pub async fn show(iii: &III, agent_id: &str, format: &OutputFormat) -> Result<()> {
    let result = iii
        .trigger("rimuru.agents.get", json!({"agent_id": agent_id}))
        .await?;
    output::print_value(&result, format);
    Ok(())
}

pub async fn connect(iii: &III, agent_type: &str, format: &OutputFormat) -> Result<()> {
    let result = iii
        .trigger("rimuru.agents.connect", json!({"agent_type": agent_type}))
        .await?;
    output::print_value(&result, format);
    Ok(())
}

pub async fn disconnect(iii: &III, agent_id: &str, format: &OutputFormat) -> Result<()> {
    let result = iii
        .trigger("rimuru.agents.disconnect", json!({"agent_id": agent_id}))
        .await?;
    output::print_value(&result, format);
    Ok(())
}

pub async fn detect(iii: &III, format: &OutputFormat) -> Result<()> {
    let result = iii.trigger("rimuru.agents.detect", json!({})).await?;
    let agents = result
        .get("detected")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    println!("{}", output::format_detected_agents(&agents, format));
    Ok(())
}
