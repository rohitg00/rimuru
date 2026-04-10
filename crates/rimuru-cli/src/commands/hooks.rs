use anyhow::Result;
use iii_sdk::{III, TriggerRequest};
use serde_json::json;

use crate::output::{self, OutputFormat};

pub async fn list(iii: &III, format: &OutputFormat) -> Result<()> {
    let result = iii
        .trigger(TriggerRequest {
            function_id: "rimuru.hooks.list".to_string(),
            payload: json!({}),
            action: None,
            timeout_ms: None,
        })
        .await?;
    let result = crate::output::unwrap_body(result);
    let hooks = if let Some(arr) = result.get("hooks").and_then(|v| v.as_array()) {
        arr.clone()
    } else {
        result.as_array().cloned().unwrap_or_default()
    };
    println!("{}", output::format_hooks_list(&hooks, format));
    Ok(())
}

pub async fn register(
    iii: &III,
    event_type: &str,
    function_id: &str,
    priority: i32,
    format: &OutputFormat,
) -> Result<()> {
    let result = iii
        .trigger(TriggerRequest {
            function_id: "rimuru.hooks.register".to_string(),
            payload: json!({
                "event_type": event_type,
                "function_id": function_id,
                "priority": priority,
            }),
            action: None,
            timeout_ms: None,
        })
        .await?;
    let result = crate::output::unwrap_body(result);

    let success = result
        .get("registered")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if success {
        println!("Hook registered: {event_type} -> {function_id} (priority {priority})");
    }

    output::print_value(&result, format);
    Ok(())
}

pub async fn dispatch(
    iii: &III,
    event_type: &str,
    payload: Option<&str>,
    format: &OutputFormat,
) -> Result<()> {
    let event_payload = if let Some(p) = payload {
        serde_json::from_str(p).unwrap_or_else(|_| json!({"data": p}))
    } else {
        json!({})
    };

    let result = iii
        .trigger(TriggerRequest {
            function_id: "rimuru.hooks.dispatch".to_string(),
            payload: json!({
                "event_type": event_type,
                "payload": event_payload,
            }),
            action: None,
            timeout_ms: None,
        })
        .await?;
    let result = crate::output::unwrap_body(result);

    let handler_count = result
        .get("handlers_called")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    println!("Dispatched '{event_type}' to {handler_count} handler(s).");
    output::print_value(&result, format);
    Ok(())
}
