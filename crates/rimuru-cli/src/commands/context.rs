use anyhow::Result;
use comfy_table::{Table, presets::UTF8_FULL};
use iii_sdk::{III, TriggerRequest};
use serde_json::json;

use crate::output::{self, OutputFormat};

pub async fn breakdown(iii: &III, session_id: &str, format: &OutputFormat) -> Result<()> {
    let result = iii
        .trigger(TriggerRequest {
            function_id: "rimuru.context.breakdown".to_string(),
            payload: json!({"session_id": session_id}),
            action: None,
            timeout_ms: Some(30_000),
        })
        .await?;

    if let OutputFormat::Json = format {
        output::print_value(&result, format);
        return Ok(());
    }

    let total = result
        .get("total_tokens")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    if total == 0 {
        println!("No token data available for session {session_id}");
        return Ok(());
    }

    let pct = |val: u64| -> String {
        if total == 0 {
            "0.0%".to_string()
        } else {
            format!("{:.1}%", val as f64 / total as f64 * 100.0)
        }
    };

    let fields = [
        ("System Prompt", "system_prompt_tokens"),
        ("User Messages", "user_tokens"),
        ("Assistant Output", "assistant_tokens"),
        ("Conversation", "conversation_tokens"),
        ("Tool Schemas", "tool_schema_tokens"),
        ("Tool Results", "tool_result_tokens"),
        ("File Reads", "file_read_tokens"),
        ("Bash Output", "bash_output_tokens"),
        ("MCP", "mcp_tokens"),
        ("Cache Read", "cache_read_tokens"),
        ("Cache Write", "cache_write_tokens"),
    ];

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec!["Category", "Tokens", "% of Total"]);

    for (label, key) in &fields {
        let val = result.get(key).and_then(|v| v.as_u64()).unwrap_or(0);
        if val > 0 {
            table.add_row(vec![label.to_string(), format!("{val}"), pct(val)]);
        }
    }

    table.add_row(vec![
        "TOTAL".to_string(),
        format!("{total}"),
        "100%".to_string(),
    ]);

    println!("Context Breakdown for session {session_id}:");
    println!("{table}");

    let turns = result
        .get("turns")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    println!("{turns} turns recorded");

    Ok(())
}

pub async fn breakdowns(iii: &III, format: &OutputFormat) -> Result<()> {
    let result = iii
        .trigger(TriggerRequest {
            function_id: "rimuru.context.breakdown_by_session".to_string(),
            payload: json!({}),
            action: None,
            timeout_ms: None,
        })
        .await?;

    if let OutputFormat::Json = format {
        output::print_value(&result, format);
        return Ok(());
    }

    let total = result.get("total").and_then(|v| v.as_u64()).unwrap_or(0);
    println!("{total} cached breakdowns");

    if let Some(arr) = result.get("breakdowns").and_then(|v| v.as_array()) {
        let mut table = Table::new();
        table.load_preset(UTF8_FULL);
        table.set_header(vec!["Session", "Total Tokens", "Waste %"]);

        for b in arr {
            let sid = b.get("session_id").and_then(|v| v.as_str()).unwrap_or("?");
            let tokens = b.get("total_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
            let waste = b.get("waste_percent").and_then(|v| v.as_f64());
            let waste_str = waste
                .map(|w| format!("{w:.1}%"))
                .unwrap_or_else(|| "-".to_string());
            table.add_row(vec![
                sid[..8.min(sid.len())].to_string(),
                format!("{tokens}"),
                waste_str,
            ]);
        }
        println!("{table}");
    }

    Ok(())
}

pub async fn utilization(iii: &III, format: &OutputFormat) -> Result<()> {
    let result = iii
        .trigger(TriggerRequest {
            function_id: "rimuru.context.utilization".to_string(),
            payload: json!({}),
            action: None,
            timeout_ms: None,
        })
        .await?;

    if let OutputFormat::Json = format {
        output::print_value(&result, format);
        return Ok(());
    }

    let total = result
        .get("total_active")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    if total == 0 {
        println!("No active sessions.");
        return Ok(());
    }

    if let Some(arr) = result.get("utilizations").and_then(|v| v.as_array()) {
        let mut table = Table::new();
        table.load_preset(UTF8_FULL);
        table.set_header(vec!["Session", "Model", "Used", "Window", "Utilization"]);

        for u in arr {
            let sid = u.get("session_id").and_then(|v| v.as_str()).unwrap_or("?");
            let model = u.get("model").and_then(|v| v.as_str()).unwrap_or("?");
            let used = u.get("tokens_used").and_then(|v| v.as_u64()).unwrap_or(0);
            let window = u
                .get("context_window_size")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let pct = u
                .get("utilization_percent")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let near = u
                .get("is_near_limit")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let marker = if near { " !" } else { "" };

            table.add_row(vec![
                sid[..8.min(sid.len())].to_string(),
                model.to_string(),
                format!("{used}"),
                format!("{window}"),
                format!("{pct:.1}%{marker}"),
            ]);
        }
        println!("{table}");
    }

    Ok(())
}

pub async fn waste(iii: &III, format: &OutputFormat) -> Result<()> {
    let result = iii
        .trigger(TriggerRequest {
            function_id: "rimuru.context.waste".to_string(),
            payload: json!({}),
            action: None,
            timeout_ms: None,
        })
        .await?;

    if let OutputFormat::Json = format {
        output::print_value(&result, format);
        return Ok(());
    }

    let total_waste = result
        .get("total_waste_tokens")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let analyzed = result
        .get("total_sessions_analyzed")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    println!("{analyzed} sessions analyzed, {total_waste} tokens potentially wasted");

    if let Some(arr) = result.get("sessions").and_then(|v| v.as_array()) {
        if arr.is_empty() {
            println!(
                "No waste detected. Run `rimuru context breakdown <session-id>` to analyze a session first."
            );
            return Ok(());
        }

        let mut table = Table::new();
        table.load_preset(UTF8_FULL);
        table.set_header(vec![
            "Session", "Total", "Schemas", "Bash", "MCP", "Waste %", "Savings",
        ]);

        for r in arr {
            let sid = r.get("session_id").and_then(|v| v.as_str()).unwrap_or("?");
            let total = r.get("total_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
            let schemas = r
                .get("tool_schema_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let bash = r
                .get("bash_output_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            let mcp = r.get("mcp_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
            let waste_pct = r
                .get("waste_percent")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let savings = r
                .get("potential_savings")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);

            table.add_row(vec![
                sid[..8.min(sid.len())].to_string(),
                format!("{total}"),
                format!("{schemas}"),
                format!("{bash}"),
                format!("{mcp}"),
                format!("{waste_pct:.1}%"),
                format!("{savings}"),
            ]);
        }
        println!("{table}");
    }

    Ok(())
}
