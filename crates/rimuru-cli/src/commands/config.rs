use anyhow::Result;
use iii_sdk::{III, TriggerRequest};
use serde_json::{Value, json};

use crate::output::{self, OutputFormat};

// ---------- cross-agent config sync (#26 / #31) ----------

/// Read every installed agent's native config and emit one canonical
/// JSON document on stdout. Pipe it into `rimuru config sync import -`
/// on another machine to copy state across hosts.
pub async fn sync_export(iii: &III) -> Result<()> {
    let result = iii
        .trigger(TriggerRequest {
            function_id: "rimuru.sync.export".to_string(),
            payload: json!({}),
            action: None,
            timeout_ms: Some(15_000),
        })
        .await?;
    let body = crate::output::unwrap_body(result);
    let canonical = body.get("canonical").cloned().unwrap_or(Value::Null);
    println!("{}", serde_json::to_string_pretty(&canonical)?);

    if let Some(errors) = body.get("errors").and_then(|v| v.as_object())
        && !errors.is_empty()
    {
        eprintln!();
        eprintln!("Read errors (these agents were skipped):");
        for (agent, err) in errors {
            eprintln!("  {}: {}", agent, err.as_str().unwrap_or(""));
        }
    }
    Ok(())
}

/// Read a canonical JSON file (or `-` for stdin) and either print the
/// per-agent diff (default, dry-run) or apply it to every installed
/// agent's native config (`--apply`). Backups are taken before each
/// write.
pub async fn sync_import(iii: &III, path: &str, apply: bool) -> Result<()> {
    let raw = if path == "-" {
        use std::io::Read;
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf)?;
        buf
    } else {
        std::fs::read_to_string(path)?
    };
    let canonical: Value = serde_json::from_str(&raw)
        .map_err(|e| anyhow::anyhow!("invalid canonical JSON in {}: {}", path, e))?;

    let result = iii
        .trigger(TriggerRequest {
            function_id: "rimuru.sync.import".to_string(),
            payload: json!({
                "canonical": canonical,
                "apply": apply,
            }),
            action: None,
            timeout_ms: Some(30_000),
        })
        .await?;
    let body = crate::output::unwrap_body(result);

    let mut failed_agents: Vec<String> = Vec::new();

    if let Some(results) = body.get("results").and_then(|v| v.as_object()) {
        for (agent, entry) in results {
            let applied = entry
                .get("applied")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let reason = entry.get("reason").and_then(|v| v.as_str()).unwrap_or("");
            let backup = entry
                .get("backup_file")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let error = entry.get("error").and_then(|v| v.as_str());

            println!("== {} ==", agent);
            if applied {
                println!("  applied");
                if !backup.is_empty() {
                    println!("  backup: {}", backup);
                }
            } else if let Some(err) = error {
                println!("  error: {}", err);
                // A per-agent error in --apply mode is a real failure.
                // Dry-run errors (e.g. read parse failures) surface via
                // the read_error field instead; the `error` key only
                // appears when the write path actually ran.
                if apply {
                    failed_agents.push(agent.clone());
                }
            } else if !reason.is_empty() {
                println!("  skipped ({})", reason);
            }

            if let Some(diff) = entry.get("diff") {
                print_sync_diff(diff);
            }
            println!();
        }
    }

    if !apply {
        eprintln!("Dry run — pass --apply to write changes.");
    } else if !failed_agents.is_empty() {
        anyhow::bail!(
            "sync failed for {} agent(s): {}",
            failed_agents.len(),
            failed_agents.join(", ")
        );
    }
    Ok(())
}

/// Show drift between every installed agent and the merged canonical
/// state (or a specific agent when --agent is set).
pub async fn sync_diff(iii: &III, agent_filter: Option<&str>) -> Result<()> {
    let mut payload = serde_json::Map::new();
    if let Some(name) = agent_filter {
        payload.insert("agent".into(), Value::String(name.to_string()));
    }
    let result = iii
        .trigger(TriggerRequest {
            function_id: "rimuru.sync.diff".to_string(),
            payload: Value::Object(payload),
            action: None,
            timeout_ms: Some(15_000),
        })
        .await?;
    let body = crate::output::unwrap_body(result);

    if let Some(diffs) = body.get("diffs").and_then(|v| v.as_object()) {
        if diffs.is_empty() {
            println!("No agents found.");
            return Ok(());
        }
        for (agent, diff) in diffs {
            println!("== {} ==", agent);
            print_sync_diff(diff);
            println!();
        }
    }
    Ok(())
}

fn print_sync_diff(diff: &Value) {
    let print_list = |label: &str, key1: &str, key2: &str| {
        if let Some(section) = diff.get(key1).and_then(|v| v.as_object())
            && let Some(added) = section.get(key2).and_then(|v| v.as_array())
            && !added.is_empty()
        {
            let names: Vec<&str> = added.iter().filter_map(|v| v.as_str()).collect();
            println!("  {}: {}", label, names.join(", "));
        }
    };

    print_list("mcp servers added", "mcp_servers", "added");
    print_list("mcp servers removed", "mcp_servers", "removed");
    print_list("mcp servers changed", "mcp_servers", "changed");
    print_list("allowed tools added", "allowed_tools", "added");
    print_list("allowed tools removed", "allowed_tools", "removed");
    print_list("denied tools added", "denied_tools", "added");
    print_list("denied tools removed", "denied_tools", "removed");
    print_list("model added", "model_preferences", "added");
    print_list("model changed", "model_preferences", "changed");
    print_list("model removed", "model_preferences", "removed");

    if diff
        .get("custom_instructions_changed")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        println!("  custom instructions changed");
    }
}

pub async fn get(iii: &III, key: Option<&str>, format: &OutputFormat) -> Result<()> {
    let input = if let Some(k) = key {
        json!({"key": k})
    } else {
        json!({})
    };
    let result = iii
        .trigger(TriggerRequest {
            function_id: "rimuru.config.get".to_string(),
            payload: input,
            action: None,
            timeout_ms: None,
        })
        .await?;
    let result = crate::output::unwrap_body(result);

    if let Some(k) = key {
        if let Some(val) = result.get("value") {
            let source = result
                .get("source")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            println!("{k} = {} ({})", val, source);
        } else {
            anyhow::bail!("Config key '{k}' not found");
        }
    } else {
        let config_obj = result.get("config").unwrap_or(&result);
        println!("{}", output::format_config(config_obj, format));
    }
    Ok(())
}

pub async fn set(iii: &III, key: &str, value: &str, format: &OutputFormat) -> Result<()> {
    let typed_value = if value == "true" {
        Value::Bool(true)
    } else if value == "false" {
        Value::Bool(false)
    } else if let Ok(n) = value.parse::<i64>() {
        Value::Number(n.into())
    } else if let Ok(n) = value.parse::<f64>() {
        serde_json::Number::from_f64(n)
            .map(Value::Number)
            .unwrap_or_else(|| Value::String(value.to_string()))
    } else {
        Value::String(value.to_string())
    };

    let result = iii
        .trigger(TriggerRequest {
            function_id: "rimuru.config.set".to_string(),
            payload: json!({
                "key": key,
                "value": typed_value,
            }),
            action: None,
            timeout_ms: None,
        })
        .await?;
    let result = crate::output::unwrap_body(result);

    println!("Set {key} = {value}");
    output::print_value(&result, format);
    Ok(())
}
