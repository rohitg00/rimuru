use std::time::Duration;

use anyhow::Result;
use comfy_table::{Cell, Color, ContentArrangement, Table};
use iii_sdk::{III, TriggerRequest};
use serde_json::json;

use crate::output::{self, OutputFormat};

pub async fn start(
    iii: &III,
    limit: f64,
    action: &str,
    command: &[String],
    _format: &OutputFormat,
) -> Result<()> {
    if command.is_empty() {
        anyhow::bail!("No command specified");
    }

    let guard_id = uuid::Uuid::new_v4().to_string();
    let command_str = command.join(" ");
    let started_at = chrono::Utc::now().to_rfc3339();

    iii.trigger(TriggerRequest {
        function_id: "rimuru.guard.register".to_string(),
        payload: json!({
            "id": guard_id,
            "command": command_str,
            "limit": limit,
            "action": action,
            "started_at": started_at
        }),
        action: None,
        timeout_ms: None,
    })
    .await?;

    println!("Guard started: {}", &guard_id[..8]);
    println!("  Limit: ${:.2} | Action: {}", limit, action);
    println!("  Running: {}", command_str);
    println!();

    let mut child = tokio::process::Command::new(&command[0])
        .args(&command[1..])
        .spawn()?;

    let mut action_taken = "none".to_string();
    let mut warned = false;
    let mut total_cost = 0.0_f64;

    loop {
        tokio::select! {
            exit_status = child.wait() => {
                match exit_status {
                    Ok(status) => {
                        println!("\nProcess exited with status: {}", status);
                    }
                    Err(e) => {
                        println!("\nProcess error: {}", e);
                    }
                }
                break;
            }
            _ = tokio::time::sleep(Duration::from_secs(5)) => {
                let cost_result = iii.trigger(TriggerRequest {
                    function_id: "rimuru.costs.summary".to_string(),
                    payload: json!({}),
                    action: None,
                    timeout_ms: Some(10_000),
                }).await;

                if let Ok(result) = cost_result {
                    let body = output::unwrap_body(result);
                    total_cost = body
                        .get("summary")
                        .and_then(|s| s.get("total_cost"))
                        .and_then(|v| v.as_f64())
                        .unwrap_or(total_cost);
                }

                if total_cost >= limit {
                    if action == "kill" {
                        println!("\n[GUARD] Cost ${:.2} exceeded limit ${:.2} — killing process", total_cost, limit);
                        let _ = child.kill().await;
                        action_taken = "killed".to_string();
                        break;
                    } else if !warned {
                        println!("\n[GUARD] Warning: cost ${:.2} exceeded limit ${:.2}", total_cost, limit);
                        action_taken = "warned".to_string();
                        warned = true;
                    }
                }
            }
        }
    }

    let ended_at = chrono::Utc::now().to_rfc3339();

    let _ = iii
        .trigger(TriggerRequest {
            function_id: "rimuru.guard.complete".to_string(),
            payload: json!({
                "id": guard_id,
                "final_cost": total_cost,
                "action_taken": action_taken,
                "ended_at": ended_at
            }),
            action: None,
            timeout_ms: None,
        })
        .await;

    println!();
    println!("Guard summary:");
    println!("  ID: {}", &guard_id[..8]);
    println!("  Final cost: ${:.2}", total_cost);
    println!("  Limit: ${:.2}", limit);
    println!("  Action taken: {}", action_taken);

    Ok(())
}

pub async fn status(iii: &III, format: &OutputFormat) -> Result<()> {
    let result = iii
        .trigger(TriggerRequest {
            function_id: "rimuru.guard.list".to_string(),
            payload: json!({}),
            action: None,
            timeout_ms: None,
        })
        .await?;

    let body = output::unwrap_body(result);
    let guards = body
        .get("guards")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&body)?);
        }
        OutputFormat::Yaml => {
            println!("{}", serde_yaml::to_string(&body)?);
        }
        OutputFormat::Table => {
            if guards.is_empty() {
                println!("No active guards");
                return Ok(());
            }

            let mut table = Table::new();
            table.set_content_arrangement(ContentArrangement::Dynamic);
            table.set_header(vec![
                Cell::new("ID").fg(Color::Cyan),
                Cell::new("Command").fg(Color::Cyan),
                Cell::new("Limit").fg(Color::Cyan),
                Cell::new("Current").fg(Color::Cyan),
                Cell::new("Action").fg(Color::Cyan),
                Cell::new("Started").fg(Color::Cyan),
            ]);

            for guard in &guards {
                let id = guard.get("id").and_then(|v| v.as_str()).unwrap_or("");
                let short_id = if id.len() > 8 { &id[..8] } else { id };
                table.add_row(vec![
                    Cell::new(short_id),
                    Cell::new(
                        guard
                            .get("command")
                            .and_then(|v| v.as_str())
                            .unwrap_or(""),
                    ),
                    Cell::new(format!(
                        "${:.2}",
                        guard.get("limit").and_then(|v| v.as_f64()).unwrap_or(0.0)
                    )),
                    Cell::new(format!(
                        "${:.2}",
                        guard
                            .get("current_cost")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.0)
                    )),
                    Cell::new(
                        guard
                            .get("action")
                            .and_then(|v| v.as_str())
                            .unwrap_or("warn"),
                    ),
                    Cell::new(
                        guard
                            .get("started_at")
                            .and_then(|v| v.as_str())
                            .unwrap_or(""),
                    ),
                ]);
            }

            println!("{table}");
        }
    }

    Ok(())
}

pub async fn history(iii: &III, format: &OutputFormat) -> Result<()> {
    let result = iii
        .trigger(TriggerRequest {
            function_id: "rimuru.guard.history".to_string(),
            payload: json!({}),
            action: None,
            timeout_ms: None,
        })
        .await?;

    let body = output::unwrap_body(result);
    let history = body
        .get("history")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&body)?);
        }
        OutputFormat::Yaml => {
            println!("{}", serde_yaml::to_string(&body)?);
        }
        OutputFormat::Table => {
            if history.is_empty() {
                println!("No guard history");
                return Ok(());
            }

            let mut table = Table::new();
            table.set_content_arrangement(ContentArrangement::Dynamic);
            table.set_header(vec![
                Cell::new("ID").fg(Color::Cyan),
                Cell::new("Command").fg(Color::Cyan),
                Cell::new("Limit").fg(Color::Cyan),
                Cell::new("Final Cost").fg(Color::Cyan),
                Cell::new("Action Taken").fg(Color::Cyan),
                Cell::new("Started").fg(Color::Cyan),
                Cell::new("Ended").fg(Color::Cyan),
            ]);

            for record in &history {
                let id = record.get("id").and_then(|v| v.as_str()).unwrap_or("");
                let short_id = if id.len() > 8 { &id[..8] } else { id };
                table.add_row(vec![
                    Cell::new(short_id),
                    Cell::new(
                        record
                            .get("command")
                            .and_then(|v| v.as_str())
                            .unwrap_or(""),
                    ),
                    Cell::new(format!(
                        "${:.2}",
                        record.get("limit").and_then(|v| v.as_f64()).unwrap_or(0.0)
                    )),
                    Cell::new(format!(
                        "${:.2}",
                        record
                            .get("final_cost")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.0)
                    )),
                    Cell::new(
                        record
                            .get("action_taken")
                            .and_then(|v| v.as_str())
                            .unwrap_or("none"),
                    ),
                    Cell::new(
                        record
                            .get("started_at")
                            .and_then(|v| v.as_str())
                            .unwrap_or(""),
                    ),
                    Cell::new(
                        record
                            .get("ended_at")
                            .and_then(|v| v.as_str())
                            .unwrap_or(""),
                    ),
                ]);
            }

            println!("{table}");
        }
    }

    Ok(())
}
