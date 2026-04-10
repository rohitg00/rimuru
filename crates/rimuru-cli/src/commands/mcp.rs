use anyhow::Result;
use comfy_table::{Table, presets::UTF8_FULL};
use iii_sdk::{III, TriggerRequest};
use serde_json::json;

use crate::output::{self, OutputFormat, unwrap_body};

pub async fn list(iii: &III, format: &OutputFormat) -> Result<()> {
    let result = iii
        .trigger(TriggerRequest {
            function_id: "rimuru.mcp.list".to_string(),
            payload: json!({}),
            action: None,
            timeout_ms: None,
        })
        .await?;
    let result = unwrap_body(result);
    let servers = if let Some(arr) = result.get("servers").and_then(|v| v.as_array()) {
        arr.clone()
    } else {
        result.as_array().cloned().unwrap_or_default()
    };
    println!("{}", output::format_mcp_list(&servers, format));
    Ok(())
}

pub async fn proxy_connect(
    iii: &III,
    name: &str,
    command: &str,
    args: &[String],
    format: &OutputFormat,
) -> Result<()> {
    let result = iii
        .trigger(TriggerRequest {
            function_id: "rimuru.mcp.proxy.connect".to_string(),
            payload: json!({"name": name, "command": command, "args": args}),
            action: None,
            timeout_ms: Some(30_000),
        })
        .await?;
    let result = unwrap_body(result);

    if matches!(format, OutputFormat::Json | OutputFormat::Yaml) {
        output::print_value(&result, format);
        return Ok(());
    }

    let tool_count = result
        .get("tool_count")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let schema_tokens = result
        .get("schema_tokens")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let server_name = result
        .get("server_name")
        .and_then(|v| v.as_str())
        .unwrap_or(name);

    println!(
        "Connected to '{}': {} tools, ~{} schema tokens",
        server_name, tool_count, schema_tokens
    );
    Ok(())
}

pub async fn proxy_disconnect(iii: &III, name: &str, format: &OutputFormat) -> Result<()> {
    let result = iii
        .trigger(TriggerRequest {
            function_id: "rimuru.mcp.proxy.disconnect".to_string(),
            payload: json!({"name": name}),
            action: None,
            timeout_ms: None,
        })
        .await?;
    let result = unwrap_body(result);

    if matches!(format, OutputFormat::Json | OutputFormat::Yaml) {
        output::print_value(&result, format);
        return Ok(());
    }

    println!("Disconnected '{}'", name);
    Ok(())
}

pub async fn proxy_tools(iii: &III, server: Option<&str>, format: &OutputFormat) -> Result<()> {
    let mut payload = json!({"progressive": true});
    if let Some(s) = server {
        payload["server"] = json!(s);
    }

    let result = iii
        .trigger(TriggerRequest {
            function_id: "rimuru.mcp.proxy.tools".to_string(),
            payload,
            action: None,
            timeout_ms: None,
        })
        .await?;
    let result = unwrap_body(result);

    if matches!(format, OutputFormat::Json | OutputFormat::Yaml) {
        output::print_value(&result, format);
        return Ok(());
    }

    let progressive = result
        .get("progressive_disclosure")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if let Some(tools) = result.get("tools").and_then(|v| v.as_array()) {
        let mut table = Table::new();
        table.load_preset(UTF8_FULL);

        if progressive {
            table.set_header(vec!["Tool", "Server", "Description", "Schema Tokens"]);
            println!(
                "Progressive disclosure active — use `rimuru mcp search <query>` for full schemas"
            );
        } else {
            table.set_header(vec!["Tool", "Server", "Description", "Schema Tokens"]);
        }

        for t in tools {
            let name = t.get("name").and_then(|v| v.as_str()).unwrap_or("?");
            let srv = t.get("server").and_then(|v| v.as_str()).unwrap_or("?");
            let desc = t.get("description").and_then(|v| v.as_str()).unwrap_or("-");
            let tokens = t.get("schema_tokens").and_then(|v| v.as_u64()).unwrap_or(0);

            table.add_row(vec![
                name.to_string(),
                srv.to_string(),
                desc[..desc.len().min(50)].to_string(),
                format!("{tokens}"),
            ]);
        }

        println!("{table}");
    }

    let total = result.get("total").and_then(|v| v.as_u64()).unwrap_or(0);
    let total_tokens = result
        .get("total_schema_tokens")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    println!("{total} tools, {total_tokens} total schema tokens");
    Ok(())
}

pub async fn proxy_search(iii: &III, query: &str, format: &OutputFormat) -> Result<()> {
    let result = iii
        .trigger(TriggerRequest {
            function_id: "rimuru.mcp.proxy.search".to_string(),
            payload: json!({"query": query, "limit": 10}),
            action: None,
            timeout_ms: None,
        })
        .await?;
    let result = unwrap_body(result);

    if matches!(format, OutputFormat::Json | OutputFormat::Yaml) {
        output::print_value(&result, format);
        return Ok(());
    }

    if let Some(tools) = result.get("tools").and_then(|v| v.as_array()) {
        if tools.is_empty() {
            println!("No tools matching '{query}'");
            return Ok(());
        }

        for t in tools {
            let name = t.get("name").and_then(|v| v.as_str()).unwrap_or("?");
            let srv = t.get("server").and_then(|v| v.as_str()).unwrap_or("?");
            let desc = t.get("description").and_then(|v| v.as_str()).unwrap_or("-");
            println!("  {name} ({srv}) — {desc}");
        }
    }
    Ok(())
}

pub async fn proxy_call(
    iii: &III,
    tool: &str,
    args_json: Option<&str>,
    format: &OutputFormat,
) -> Result<()> {
    let arguments = if let Some(raw) = args_json {
        serde_json::from_str(raw).unwrap_or(json!({}))
    } else {
        json!({})
    };

    let result = iii
        .trigger(TriggerRequest {
            function_id: "rimuru.mcp.proxy.call".to_string(),
            payload: json!({"tool": tool, "arguments": arguments}),
            action: None,
            timeout_ms: Some(60_000),
        })
        .await?;
    let result = unwrap_body(result);

    if matches!(format, OutputFormat::Json | OutputFormat::Yaml) {
        output::print_value(&result, format);
        return Ok(());
    }

    let cache_hit = result
        .get("cache_hit")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let input_tokens = result
        .get("input_tokens")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let output_tokens = result
        .get("output_tokens")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let latency = result
        .get("latency_ms")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);

    if let Some(r) = result.get("result") {
        println!("{}", serde_json::to_string_pretty(r)?);
    }

    println!(
        "\n--- {} tokens in, {} tokens out, {:.0}ms{}",
        input_tokens,
        output_tokens,
        latency,
        if cache_hit { " (cached)" } else { "" }
    );
    Ok(())
}

pub async fn proxy_stats(iii: &III, format: &OutputFormat) -> Result<()> {
    let result = iii
        .trigger(TriggerRequest {
            function_id: "rimuru.mcp.proxy.stats".to_string(),
            payload: json!({}),
            action: None,
            timeout_ms: None,
        })
        .await?;
    let result = unwrap_body(result);

    if matches!(format, OutputFormat::Json | OutputFormat::Yaml) {
        output::print_value(&result, format);
        return Ok(());
    }

    let total_calls = result
        .get("total_calls")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let total_input = result
        .get("total_input_tokens")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let total_output = result
        .get("total_output_tokens")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let cache_rate = result
        .get("cache_hit_rate")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);

    println!(
        "{} calls, {} input tokens, {} output tokens, {:.1}% cache hit rate",
        total_calls, total_input, total_output, cache_rate
    );

    if let Some(tools) = result.get("tools").and_then(|v| v.as_array())
        && !tools.is_empty()
    {
        let mut table = Table::new();
        table.load_preset(UTF8_FULL);
        table.set_header(vec![
            "Tool",
            "Calls",
            "Input",
            "Output",
            "Cache Hits",
            "Avg Latency",
        ]);

        for t in tools {
            let name = t.get("tool").and_then(|v| v.as_str()).unwrap_or("?");
            let calls = t.get("calls").and_then(|v| v.as_u64()).unwrap_or(0);
            let inp = t.get("input_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
            let out = t.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
            let hits = t.get("cache_hits").and_then(|v| v.as_u64()).unwrap_or(0);
            let lat = t
                .get("avg_latency_ms")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);

            table.add_row(vec![
                name.to_string(),
                format!("{calls}"),
                format!("{inp}"),
                format!("{out}"),
                format!("{hits}"),
                format!("{lat:.0}ms"),
            ]);
        }

        println!("{table}");
    }
    Ok(())
}
