use anyhow::Result;
use iii_sdk::{III, TriggerRequest};
use serde_json::json;

use crate::output::{self, OutputFormat};

pub async fn list(iii: &III, format: &OutputFormat) -> Result<()> {
    let result = iii
        .trigger(TriggerRequest {
            function_id: "rimuru.agents.list".to_string(),
            payload: json!({}),
            action: None,
            timeout_ms: None,
        })
        .await?;
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
        .trigger(TriggerRequest {
            function_id: "rimuru.agents.get".to_string(),
            payload: json!({"agent_id": agent_id}),
            action: None,
            timeout_ms: None,
        })
        .await?;
    output::print_value(&result, format);
    Ok(())
}

pub async fn connect(iii: &III, agent_type: &str, format: &OutputFormat) -> Result<()> {
    let result = iii
        .trigger(TriggerRequest {
            function_id: "rimuru.agents.connect".to_string(),
            payload: json!({"agent_type": agent_type}),
            action: None,
            timeout_ms: None,
        })
        .await?;
    output::print_value(&result, format);
    Ok(())
}

pub async fn disconnect(iii: &III, agent_id: &str, format: &OutputFormat) -> Result<()> {
    let result = iii
        .trigger(TriggerRequest {
            function_id: "rimuru.agents.disconnect".to_string(),
            payload: json!({"agent_id": agent_id}),
            action: None,
            timeout_ms: None,
        })
        .await?;
    output::print_value(&result, format);
    Ok(())
}

pub async fn detect(iii: &III, format: &OutputFormat) -> Result<()> {
    let result = iii
        .trigger(TriggerRequest {
            function_id: "rimuru.agents.detect".to_string(),
            payload: json!({}),
            action: None,
            timeout_ms: None,
        })
        .await?;
    let agents = result
        .get("detected")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    println!("{}", output::format_detected_agents(&agents, format));
    Ok(())
}
