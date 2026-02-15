use clap::Subcommand;
use colored::Colorize;
use comfy_table::{modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, Cell, Color, Table};
use rimuru_core::{Hook, HookContext, HookData, HookManager};
use std::sync::Arc;

#[derive(Subcommand)]
pub enum HooksCommand {
    #[command(about = "List all registered hooks and their handlers")]
    List {
        #[arg(short, long, help = "Filter by hook type")]
        hook_type: Option<String>,

        #[arg(
            short,
            long,
            default_value = "text",
            help = "Output format (text, json)"
        )]
        format: String,
    },

    #[command(about = "Manually trigger a hook (for testing)")]
    Trigger {
        #[arg(help = "Hook name (e.g., pre_session_start, on_cost_recorded)")]
        name: String,

        #[arg(short, long, help = "Optional JSON data to pass to handlers")]
        data: Option<String>,

        #[arg(short, long, help = "Source identifier for the trigger")]
        source: Option<String>,
    },

    #[command(about = "Show recent hook execution log")]
    Log {
        #[arg(
            short,
            long,
            default_value = "20",
            help = "Number of recent executions to show"
        )]
        limit: usize,

        #[arg(short = 't', long, help = "Filter by hook type")]
        hook_type: Option<String>,

        #[arg(
            short,
            long,
            default_value = "text",
            help = "Output format (text, json)"
        )]
        format: String,
    },

    #[command(about = "Show all standard hook types")]
    Types,
}

pub async fn handle_hooks_command(cmd: Option<HooksCommand>) -> anyhow::Result<()> {
    let manager = Arc::new(HookManager::new());

    match cmd.unwrap_or(HooksCommand::List {
        hook_type: None,
        format: "text".to_string(),
    }) {
        HooksCommand::List { hook_type, format } => {
            cmd_hooks_list(&manager, hook_type.as_deref(), &format).await
        }
        HooksCommand::Trigger { name, data, source } => {
            cmd_hooks_trigger(&manager, &name, data.as_deref(), source.as_deref()).await
        }
        HooksCommand::Log {
            limit,
            hook_type,
            format,
        } => cmd_hooks_log(&manager, limit, hook_type.as_deref(), &format).await,
        HooksCommand::Types => cmd_hooks_types().await,
    }
}

async fn cmd_hooks_list(
    manager: &HookManager,
    hook_type_filter: Option<&str>,
    format: &str,
) -> anyhow::Result<()> {
    let all_handlers = manager.get_all_handlers().await;

    let hook_filter = hook_type_filter.map(Hook::from_name);

    if format == "json" {
        let output: serde_json::Value = all_handlers
            .iter()
            .filter(|(hook, _)| {
                if let Some(ref filter) = hook_filter {
                    return hook == &filter;
                }
                true
            })
            .map(|(hook, handlers)| {
                (
                    hook.name().to_string(),
                    serde_json::Value::Array(
                        handlers
                            .iter()
                            .map(|h| {
                                serde_json::json!({
                                    "name": h.name,
                                    "priority": h.priority,
                                    "enabled": h.enabled,
                                    "plugin_id": h.plugin_id,
                                    "description": h.description
                                })
                            })
                            .collect::<Vec<_>>(),
                    ),
                )
            })
            .collect::<serde_json::Map<String, serde_json::Value>>()
            .into();

        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(());
    }

    println!("{}", "Registered Hooks".cyan().bold());
    println!();

    let standard_hooks = Hook::all_standard();
    let mut has_any_handlers = false;

    for hook in &standard_hooks {
        if let Some(ref filter) = hook_filter {
            if hook != filter {
                continue;
            }
        }

        let handlers = all_handlers.get(hook).cloned().unwrap_or_default();

        if handlers.is_empty() && hook_filter.is_none() {
            continue;
        }

        has_any_handlers = true;

        println!("  {} {}", "●".cyan(), hook.name().bold());

        if handlers.is_empty() {
            println!("    {}", "No handlers registered".dimmed());
        } else {
            let mut table = Table::new();
            table
                .load_preset(UTF8_FULL)
                .apply_modifier(UTF8_ROUND_CORNERS)
                .set_header(vec![
                    Cell::new("Handler").fg(Color::White),
                    Cell::new("Priority").fg(Color::White),
                    Cell::new("Enabled").fg(Color::White),
                    Cell::new("Plugin").fg(Color::White),
                    Cell::new("Description").fg(Color::White),
                ]);

            for handler in &handlers {
                let enabled_cell = if handler.enabled {
                    Cell::new("Yes").fg(Color::Green)
                } else {
                    Cell::new("No").fg(Color::Yellow)
                };

                table.add_row(vec![
                    Cell::new(&handler.name),
                    Cell::new(handler.priority.to_string()),
                    enabled_cell,
                    Cell::new(handler.plugin_id.as_deref().unwrap_or("-")),
                    Cell::new(handler.description.as_deref().unwrap_or("-")),
                ]);
            }

            println!("{table}");
        }
        println!();
    }

    if !has_any_handlers {
        println!("{}", "No hooks have registered handlers.".yellow());
        println!();
        println!(
            "{}",
            "Hooks are registered by plugins or the built-in handlers.".dimmed()
        );
        println!(
            "{}",
            "Run 'rimuru hooks types' to see available hook types.".dimmed()
        );
    }

    let total_handlers = manager.handler_count().await;
    println!("  Total handlers registered: {}", total_handlers);

    Ok(())
}

async fn cmd_hooks_trigger(
    manager: &HookManager,
    name: &str,
    data: Option<&str>,
    source: Option<&str>,
) -> anyhow::Result<()> {
    let hook = Hook::from_name(name);

    println!("{}", "Triggering Hook".cyan().bold());
    println!();

    println!("  {:<15} {}", "Hook:".bold(), hook.name());
    println!(
        "  {:<15} {}",
        "Source:".bold(),
        source.unwrap_or("cli-trigger")
    );

    let hook_data = if let Some(data_str) = data {
        match serde_json::from_str::<serde_json::Value>(data_str) {
            Ok(json) => {
                println!("  {:<15} {}", "Data:".bold(), data_str);
                HookData::Custom(json)
            }
            Err(e) => {
                println!("  {} Failed to parse JSON data: {}", "!".yellow(), e);
                println!("  {} Using empty data", "→".blue());
                HookData::None
            }
        }
    } else {
        println!("  {:<15} None", "Data:".bold());
        HookData::None
    };

    let ctx =
        HookContext::new(hook.clone(), hook_data).with_source(source.unwrap_or("cli-trigger"));

    println!();
    println!("  {} Executing hook handlers...", "→".blue());

    match manager.execute(ctx).await {
        Ok(result) => {
            println!();
            match result {
                rimuru_core::HookResult::Continue => {
                    println!("  {} Hook executed successfully (Continue)", "✓".green());
                }
                rimuru_core::HookResult::Skip => {
                    println!("  {} Hook skipped", "○".yellow());
                }
                rimuru_core::HookResult::Modified { message, .. } => {
                    println!("  {} Hook executed with modifications", "✓".green());
                    if let Some(msg) = message {
                        println!("    Message: {}", msg);
                    }
                }
                rimuru_core::HookResult::Abort { reason } => {
                    println!("  {} Hook aborted: {}", "✗".red(), reason);
                }
            }
        }
        Err(e) => {
            println!();
            println!("  {} Hook execution failed: {}", "✗".red(), e);
        }
    }

    Ok(())
}

async fn cmd_hooks_log(
    manager: &HookManager,
    limit: usize,
    hook_type_filter: Option<&str>,
    format: &str,
) -> anyhow::Result<()> {
    let executions = if let Some(hook_type) = hook_type_filter {
        let hook = Hook::from_name(hook_type);
        manager.get_executions_for_hook(&hook, limit).await
    } else {
        manager.get_recent_executions(limit).await
    };

    if format == "json" {
        let output: Vec<serde_json::Value> = executions
            .iter()
            .map(|e| {
                serde_json::json!({
                    "id": e.id.to_string(),
                    "hook": e.hook.name(),
                    "handler_name": e.handler_name,
                    "started_at": e.started_at.to_rfc3339(),
                    "completed_at": e.completed_at.map(|t| t.to_rfc3339()),
                    "duration_ms": e.duration_ms,
                    "result": e.result.as_ref().map(|r| format!("{:?}", r)),
                    "error": e.error,
                    "successful": e.is_successful()
                })
            })
            .collect();

        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(());
    }

    println!("{}", "Hook Execution Log".cyan().bold());
    println!();

    if executions.is_empty() {
        println!("{}", "No hook executions recorded.".yellow());
        println!();
        println!(
            "{}",
            "Hook executions are recorded when hooks are triggered.".dimmed()
        );
        println!(
            "{}",
            "Run 'rimuru hooks trigger <name>' to manually trigger a hook.".dimmed()
        );
        return Ok(());
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec![
            Cell::new("Time").fg(Color::White),
            Cell::new("Hook").fg(Color::White),
            Cell::new("Handler").fg(Color::White),
            Cell::new("Duration").fg(Color::White),
            Cell::new("Result").fg(Color::White),
        ]);

    for execution in &executions {
        let time = execution.started_at.format("%H:%M:%S").to_string();

        let duration = execution
            .duration_ms
            .map(|d| format!("{}ms", d))
            .unwrap_or_else(|| "-".to_string());

        let result_cell = if let Some(ref error) = execution.error {
            Cell::new(format!("Error: {}", truncate(error, 30))).fg(Color::Red)
        } else if let Some(ref result) = execution.result {
            match result {
                rimuru_core::HookResult::Continue => Cell::new("Continue").fg(Color::Green),
                rimuru_core::HookResult::Skip => Cell::new("Skip").fg(Color::Yellow),
                rimuru_core::HookResult::Modified { .. } => Cell::new("Modified").fg(Color::Cyan),
                rimuru_core::HookResult::Abort { reason } => {
                    Cell::new(format!("Abort: {}", truncate(reason, 20))).fg(Color::Red)
                }
            }
        } else {
            Cell::new("Pending").fg(Color::DarkGrey)
        };

        table.add_row(vec![
            Cell::new(time),
            Cell::new(execution.hook.name()),
            Cell::new(&execution.handler_name),
            Cell::new(duration),
            result_cell,
        ]);
    }

    println!("{table}");
    println!();
    println!("  Showing {} most recent executions", executions.len());

    Ok(())
}

async fn cmd_hooks_types() -> anyhow::Result<()> {
    println!("{}", "Available Hook Types".cyan().bold());
    println!();

    let hooks = Hook::all_standard();

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec![
            Cell::new("Hook Name").fg(Color::White),
            Cell::new("Description").fg(Color::White),
            Cell::new("Typical Data").fg(Color::White),
        ]);

    for hook in &hooks {
        let (description, data_type) = get_hook_info(hook);
        table.add_row(vec![
            Cell::new(hook.name()),
            Cell::new(description),
            Cell::new(data_type),
        ]);
    }

    println!("{table}");
    println!();
    println!(
        "  {}",
        "Custom hooks can also be created using Custom(name).".dimmed()
    );
    println!();
    println!("  {}", "Usage Examples:".yellow().bold());
    println!("    rimuru hooks trigger pre_session_start");
    println!("    rimuru hooks trigger on_cost_recorded --data '{{\"cost\": 0.05}}'");
    println!("    rimuru hooks list --hook-type on_error");

    Ok(())
}

fn get_hook_info(hook: &Hook) -> (&'static str, &'static str) {
    match hook {
        Hook::PreSessionStart => ("Fired before a session starts", "Session"),
        Hook::PostSessionEnd => ("Fired after a session ends", "Session"),
        Hook::OnCostRecorded => ("Fired when cost is recorded", "CostRecord"),
        Hook::OnMetricsCollected => ("Fired when metrics are collected", "MetricsSnapshot"),
        Hook::OnAgentConnect => ("Fired when agent connects", "Agent info"),
        Hook::OnAgentDisconnect => ("Fired when agent disconnects", "Agent info"),
        Hook::OnSyncComplete => ("Fired after sync completes", "Sync info"),
        Hook::OnPluginLoaded => ("Fired when plugin is loaded", "Plugin info"),
        Hook::OnPluginUnloaded => ("Fired when plugin is unloaded", "Plugin info"),
        Hook::OnConfigChanged => ("Fired when config changes", "Changed keys"),
        Hook::OnError => ("Fired on errors", "Error info"),
        Hook::Custom(_) => ("Custom user-defined hook", "Custom JSON"),
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}
