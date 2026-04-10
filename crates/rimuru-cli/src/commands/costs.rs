use anyhow::Result;
use iii_sdk::{III, TriggerRequest};
use serde_json::{Value, json};

use crate::output::{self, OutputFormat};

pub async fn summary(iii: &III, format: &OutputFormat) -> Result<()> {
    let result = iii
        .trigger(TriggerRequest {
            function_id: "rimuru.costs.summary".to_string(),
            payload: json!({}),
            action: None,
            timeout_ms: None,
        })
        .await?;
    let result = crate::output::unwrap_body(result);
    let summary = result.get("summary").unwrap_or(&result);
    println!("{}", output::format_costs_summary(summary, format));
    Ok(())
}

pub async fn daily(iii: &III, format: &OutputFormat) -> Result<()> {
    let result = iii
        .trigger(TriggerRequest {
            function_id: "rimuru.costs.daily".to_string(),
            payload: json!({}),
            action: None,
            timeout_ms: None,
        })
        .await?;
    let result = crate::output::unwrap_body(result);
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
    let result = iii
        .trigger(TriggerRequest {
            function_id: "rimuru.costs.by_agent".to_string(),
            payload: input,
            action: None,
            timeout_ms: None,
        })
        .await?;
    let result = crate::output::unwrap_body(result);
    let agents = match result {
        Value::Array(arr) => arr,
        other => vec![other],
    };
    println!("{}", output::format_daily_costs(&agents, format));
    Ok(())
}

pub async fn export(iii: &III, path: &str) -> Result<()> {
    let result = iii
        .trigger(TriggerRequest {
            function_id: "rimuru.costs.summary".to_string(),
            payload: json!({}),
            action: None,
            timeout_ms: None,
        })
        .await?;
    let result = crate::output::unwrap_body(result);
    let content = serde_json::to_string_pretty(&result)?;
    std::fs::write(path, &content)?;
    println!("Exported costs to {path}");
    Ok(())
}
