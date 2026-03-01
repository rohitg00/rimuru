use anyhow::Result;
use iii_sdk::III;
use serde_json::{json, Value};

use crate::output::{self, OutputFormat};

pub async fn list(iii: &III, format: &OutputFormat) -> Result<()> {
    let result = iii.trigger("rimuru.sessions.list", json!({})).await?;
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
        .trigger("rimuru.sessions.get", json!({"session_id": session_id}))
        .await?;
    output::print_value(&result, format);
    Ok(())
}

pub async fn active(iii: &III, format: &OutputFormat) -> Result<()> {
    let result = iii.trigger("rimuru.sessions.active", json!({})).await?;
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
    let result = iii.trigger("rimuru.sessions.history", json!({})).await?;
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
