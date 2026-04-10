use anyhow::Result;
use iii_sdk::{III, TriggerRequest};
use serde_json::{Value, json};

use crate::output::{self, OutputFormat};

pub async fn list(iii: &III, format: &OutputFormat) -> Result<()> {
    let result = iii
        .trigger(TriggerRequest {
            function_id: "rimuru.sessions.list".to_string(),
            payload: json!({}),
            action: None,
            timeout_ms: None,
        })
        .await?;
    let result = crate::output::unwrap_body(result);
    let sessions = if let Some(arr) = result.get("sessions").and_then(|v| v.as_array()) {
        arr.clone()
    } else {
        result.as_array().cloned().unwrap_or_default()
    };
    println!("{}", output::format_sessions_list(&sessions, format));
    Ok(())
}

pub async fn show(iii: &III, session_id: &str, format: &OutputFormat) -> Result<()> {
    let result = iii
        .trigger(TriggerRequest {
            function_id: "rimuru.sessions.get".to_string(),
            payload: json!({"session_id": session_id}),
            action: None,
            timeout_ms: None,
        })
        .await?;
    let result = crate::output::unwrap_body(result);
    output::print_value(&result, format);
    Ok(())
}

pub async fn active(iii: &III, format: &OutputFormat) -> Result<()> {
    let result = iii
        .trigger(TriggerRequest {
            function_id: "rimuru.sessions.active".to_string(),
            payload: json!({}),
            action: None,
            timeout_ms: None,
        })
        .await?;
    let result = crate::output::unwrap_body(result);
    let sessions = result
        .get("active_sessions")
        .or_else(|| result.get("sessions"))
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    if sessions.is_empty() {
        println!("No active sessions.");
    } else {
        println!("{}", output::format_sessions_list(&sessions, format));
    }
    Ok(())
}

pub async fn history(iii: &III, format: &OutputFormat) -> Result<()> {
    let result = iii
        .trigger(TriggerRequest {
            function_id: "rimuru.sessions.history".to_string(),
            payload: json!({}),
            action: None,
            timeout_ms: None,
        })
        .await?;
    let result = crate::output::unwrap_body(result);
    let sessions = match &result {
        Value::Array(arr) => arr.clone(),
        Value::Object(map) => map
            .get("sessions")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default(),
        _ => vec![result],
    };
    println!("{}", output::format_sessions_list(&sessions, format));
    Ok(())
}
