use clap::Subcommand;
use colored::Colorize;
use comfy_table::{modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, Cell, Color, Table};
use rimuru_core::{AdapterManager, AdapterManagerConfig, AdapterStatus, AgentType};
use std::collections::HashMap;

#[derive(Subcommand)]
pub enum AgentsCommand {
    #[command(about = "List all configured agents with connection status")]
    List {
        #[arg(short = 't', long, help = "Filter by agent type")]
        agent_type: Option<String>,

        #[arg(
            short,
            long,
            default_value = "text",
            help = "Output format (text, json)"
        )]
        format: String,

        #[arg(short, long, help = "Show all agents including inactive/uninstalled")]
        all: bool,
    },

    #[command(about = "Register a new agent (interactive config)")]
    Add {
        #[arg(help = "Agent type (claude-code, opencode)")]
        agent_type: String,

        #[arg(short, long, help = "Custom name for the agent")]
        name: Option<String>,
    },

    #[command(about = "Show detailed status for a specific agent")]
    Status {
        #[arg(help = "Agent name (optional with --compare)")]
        name: Option<String>,

        #[arg(short, long, help = "Side-by-side comparison of all agents")]
        compare: bool,
    },

    #[command(about = "Scan system for installed agents and auto-register")]
    Scan,

    #[command(about = "Remove a registered agent")]
    Remove {
        #[arg(help = "Agent name to remove")]
        name: String,
    },

    #[command(about = "Reconnect a disconnected agent")]
    Reconnect {
        #[arg(help = "Agent name to reconnect")]
        name: String,
    },
}

pub async fn handle_agents_command(cmd: Option<AgentsCommand>) -> anyhow::Result<()> {
    let manager = create_adapter_manager().await?;

    match cmd.unwrap_or(AgentsCommand::List {
        agent_type: None,
        format: "text".to_string(),
        all: false,
    }) {
        AgentsCommand::List {
            agent_type,
            format,
            all,
        } => cmd_agents_list(&manager, agent_type.as_deref(), &format, all).await,
        AgentsCommand::Add { agent_type, name } => {
            cmd_agents_add(&manager, &agent_type, name.as_deref()).await
        }
        AgentsCommand::Status { name, compare } => {
            if compare {
                cmd_agents_compare(&manager).await
            } else if let Some(agent_name) = name {
                cmd_agents_status(&manager, &agent_name).await
            } else {
                cmd_agents_compare(&manager).await
            }
        }
        AgentsCommand::Scan => cmd_agents_scan(&manager).await,
        AgentsCommand::Remove { name } => cmd_agents_remove(&manager, &name).await,
        AgentsCommand::Reconnect { name } => cmd_agents_reconnect(&manager, &name).await,
    }
}

async fn create_adapter_manager() -> anyhow::Result<AdapterManager> {
    let config = AdapterManagerConfig {
        auto_discover: true,
        health_check_interval_secs: 60,
        reconnect_on_failure: true,
        max_reconnect_attempts: 3,
    };

    let manager = AdapterManager::new(config);
    manager.initialize().await?;

    Ok(manager)
}

async fn cmd_agents_list(
    manager: &AdapterManager,
    agent_type_filter: Option<&str>,
    format: &str,
    _show_all: bool,
) -> anyhow::Result<()> {
    let adapters = manager.list_adapters().await;
    let statuses = manager.get_all_statuses().await;
    let health = manager.get_health_status().await;
    let by_type = manager.list_adapters_by_type().await;

    let filtered_adapters: Vec<String> = if let Some(type_str) = agent_type_filter {
        let agent_type = parse_agent_type(type_str)?;
        adapters
            .into_iter()
            .filter(|name| {
                by_type
                    .get(&agent_type)
                    .map(|v| v.contains(name))
                    .unwrap_or(false)
            })
            .collect()
    } else {
        adapters
    };

    if format == "json" {
        let output: Vec<serde_json::Value> = filtered_adapters
            .iter()
            .map(|name| {
                let status = statuses
                    .get(name)
                    .cloned()
                    .unwrap_or(AdapterStatus::Unknown);
                let agent_type = get_agent_type_for_adapter(name, &by_type);
                let health_info = health.get(name);

                serde_json::json!({
                    "name": name,
                    "type": format_agent_type(&agent_type),
                    "status": status.to_string(),
                    "healthy": health_info.map(|h| h.healthy).unwrap_or(false),
                    "last_check": health_info.map(|h| h.last_check.to_rfc3339()),
                    "error": health_info.and_then(|h| h.error_message.clone())
                })
            })
            .collect();

        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(());
    }

    if filtered_adapters.is_empty() {
        println!("{}", "No agents registered.".yellow());
        println!(
            "{}",
            "Run 'rimuru agents scan' to auto-discover installed agents.".dimmed()
        );
        return Ok(());
    }

    println!("{}", "Registered Agents".cyan().bold());
    println!();

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec![
            Cell::new("Name").fg(Color::White),
            Cell::new("Type").fg(Color::White),
            Cell::new("Status").fg(Color::White),
            Cell::new("Health").fg(Color::White),
            Cell::new("Last Check").fg(Color::White),
        ]);

    for name in &filtered_adapters {
        let status = statuses
            .get(name)
            .cloned()
            .unwrap_or(AdapterStatus::Unknown);
        let agent_type = get_agent_type_for_adapter(name, &by_type);
        let health_info = health.get(name);

        let status_cell = match status {
            AdapterStatus::Connected => Cell::new("Connected").fg(Color::Green),
            AdapterStatus::Disconnected => Cell::new("Disconnected").fg(Color::Yellow),
            AdapterStatus::Error => Cell::new("Error").fg(Color::Red),
            AdapterStatus::Unknown => Cell::new("Unknown").fg(Color::DarkGrey),
        };

        let health_cell = if let Some(h) = health_info {
            if h.healthy {
                Cell::new("✓ Healthy").fg(Color::Green)
            } else {
                Cell::new("✗ Unhealthy").fg(Color::Red)
            }
        } else {
            Cell::new("-").fg(Color::DarkGrey)
        };

        let last_check = health_info
            .map(|h| h.last_check.format("%H:%M:%S").to_string())
            .unwrap_or_else(|| "-".to_string());

        table.add_row(vec![
            Cell::new(name),
            Cell::new(format!(
                "{} {}",
                get_agent_icon(&agent_type),
                format_agent_type(&agent_type)
            )),
            status_cell,
            health_cell,
            Cell::new(last_check),
        ]);
    }

    println!("{table}");
    println!();
    println!("  Total: {} agents", filtered_adapters.len());

    Ok(())
}

async fn cmd_agents_add(
    _manager: &AdapterManager,
    agent_type_str: &str,
    name: Option<&str>,
) -> anyhow::Result<()> {
    let agent_type = parse_agent_type(agent_type_str)?;
    let adapter_name = name.unwrap_or(agent_type_str);

    println!(
        "{} {} {}",
        "Adding".cyan().bold(),
        get_agent_icon(&agent_type),
        format_agent_type(&agent_type)
    );
    println!();

    println!(
        "  {} Checking for {} installation...",
        "→".blue(),
        format_agent_type(&agent_type)
    );

    let installed = match agent_type {
        AgentType::ClaudeCode => {
            let home = dirs::home_dir().unwrap_or_default();
            home.join(".claude").exists()
        }
        AgentType::OpenCode => {
            let home = dirs::home_dir().unwrap_or_default();
            home.join(".opencode").exists()
        }
        _ => {
            println!(
                "  {} Agent type not yet supported for auto-detection",
                "!".yellow()
            );
            false
        }
    };

    if installed {
        println!(
            "  {} Found {} installation",
            "✓".green(),
            format_agent_type(&agent_type)
        );
        println!("  {} Agent '{}' is ready to use", "✓".green(), adapter_name);
        println!();
        println!(
            "{}",
            "Agent registered! Run 'rimuru agents list' to see all agents.".green()
        );
    } else {
        println!(
            "  {} {} not found on this system",
            "✗".red(),
            format_agent_type(&agent_type)
        );
        println!();
        println!(
            "  Install {} first, then run this command again.",
            format_agent_type(&agent_type)
        );
    }

    Ok(())
}

async fn cmd_agents_status(manager: &AdapterManager, name: &str) -> anyhow::Result<()> {
    let adapters = manager.list_adapters().await;

    if !adapters.contains(&name.to_string()) {
        println!("{} Agent '{}' not found", "✗".red(), name);
        println!();
        println!("Available agents:");
        for adapter in &adapters {
            println!("  - {}", adapter);
        }
        return Ok(());
    }

    let status = manager.get_adapter_status(name).await?;
    let health = manager.get_adapter_health(name).await;
    let by_type = manager.list_adapters_by_type().await;
    let agent_type = get_agent_type_for_adapter(name, &by_type);

    println!("{}", "Agent Status".cyan().bold());
    println!("{}", "═".repeat(50).dimmed());
    println!();

    println!(
        "  {:<15} {} {}",
        "Name:".bold(),
        get_agent_icon(&agent_type),
        name
    );
    println!(
        "  {:<15} {}",
        "Type:".bold(),
        format_agent_type(&agent_type)
    );

    let status_str = match status {
        AdapterStatus::Connected => "Connected".green().to_string(),
        AdapterStatus::Disconnected => "Disconnected".yellow().to_string(),
        AdapterStatus::Error => "Error".red().to_string(),
        AdapterStatus::Unknown => "Unknown".dimmed().to_string(),
    };
    println!("  {:<15} {}", "Status:".bold(), status_str);

    if let Some(h) = &health {
        println!();
        println!("  {}", "Health Information".yellow().bold());
        let healthy_str = if h.healthy {
            "✓ Healthy".green().to_string()
        } else {
            "✗ Unhealthy".red().to_string()
        };
        println!("  {:<15} {}", "Health:".bold(), healthy_str);
        println!(
            "  {:<15} {}",
            "Last Check:".bold(),
            h.last_check.format("%Y-%m-%d %H:%M:%S UTC")
        );
        println!("  {:<15} {}", "Failures:".bold(), h.consecutive_failures);

        if let Some(ref error) = h.error_message {
            println!();
            println!("  {}", "Last Error:".red().bold());
            println!("    {}", error);
        }
    }

    let registry = manager.registry();
    if let Some(adapter) = registry.get(name).await {
        let adapter_guard = adapter.read().await;

        println!();
        println!("  {}", "Session Information".yellow().bold());

        if let Ok(sessions) = adapter_guard.get_sessions().await {
            println!("  {:<15} {}", "Total Sessions:".bold(), sessions.len());
        }

        if let Ok(Some(active)) = adapter_guard.get_active_session().await {
            println!();
            println!("  {}", "Active Session:".green().bold());
            println!("    Session ID: {}", active.session_id);
            println!(
                "    Started:    {}",
                active.started_at.format("%Y-%m-%d %H:%M:%S UTC")
            );
            println!("    Tokens:     {}", active.current_tokens);
            if let Some(ref model) = active.model_name {
                println!("    Model:      {}", model);
            }
            if let Some(ref path) = active.project_path {
                println!("    Project:    {}", path);
            }
        } else {
            println!("  {:<15} None", "Active Session:".bold());
        }
    }

    Ok(())
}

async fn cmd_agents_compare(manager: &AdapterManager) -> anyhow::Result<()> {
    let adapters = manager.list_adapters().await;
    let statuses = manager.get_all_statuses().await;
    let health = manager.get_health_status().await;
    let by_type = manager.list_adapters_by_type().await;

    if adapters.is_empty() {
        println!("{}", "No agents registered.".yellow());
        println!(
            "{}",
            "Run 'rimuru agents scan' to auto-discover installed agents.".dimmed()
        );
        return Ok(());
    }

    println!("{}", "Agent Comparison".cyan().bold());
    println!(
        "{}",
        format!("Comparing {} agents side-by-side", adapters.len()).dimmed()
    );
    println!();

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec![
            Cell::new("Agent").fg(Color::White),
            Cell::new("Type").fg(Color::White),
            Cell::new("Status").fg(Color::White),
            Cell::new("Health").fg(Color::White),
            Cell::new("Sessions").fg(Color::White),
            Cell::new("Active").fg(Color::White),
            Cell::new("Tokens").fg(Color::White),
        ]);

    for name in &adapters {
        let status = statuses
            .get(name)
            .cloned()
            .unwrap_or(AdapterStatus::Unknown);
        let agent_type = get_agent_type_for_adapter(name, &by_type);
        let health_info = health.get(name);

        let status_cell = match status {
            AdapterStatus::Connected => Cell::new("●").fg(Color::Green),
            AdapterStatus::Disconnected => Cell::new("○").fg(Color::Yellow),
            AdapterStatus::Error => Cell::new("✗").fg(Color::Red),
            AdapterStatus::Unknown => Cell::new("?").fg(Color::DarkGrey),
        };

        let health_cell = if let Some(h) = health_info {
            if h.healthy {
                Cell::new("✓").fg(Color::Green)
            } else {
                Cell::new("✗").fg(Color::Red)
            }
        } else {
            Cell::new("-").fg(Color::DarkGrey)
        };

        let mut session_count = 0;
        let mut active_session = false;
        let mut total_tokens = 0i64;

        let registry = manager.registry();
        if let Some(adapter) = registry.get(name).await {
            let adapter_guard = adapter.read().await;
            if let Ok(sessions) = adapter_guard.get_sessions().await {
                session_count = sessions.len();
            }
            if let Ok(Some(active)) = adapter_guard.get_active_session().await {
                active_session = true;
                total_tokens = active.current_tokens;
            }
        }

        let active_cell = if active_session {
            Cell::new("Yes").fg(Color::Green)
        } else {
            Cell::new("No").fg(Color::DarkGrey)
        };

        table.add_row(vec![
            Cell::new(format!("{} {}", get_agent_icon(&agent_type), name)),
            Cell::new(format_agent_type(&agent_type)),
            status_cell,
            health_cell,
            Cell::new(session_count.to_string()),
            active_cell,
            Cell::new(format_tokens(total_tokens)),
        ]);
    }

    println!("{table}");
    println!();

    let connected = statuses
        .values()
        .filter(|s| **s == AdapterStatus::Connected)
        .count();
    let healthy = health.values().filter(|h| h.healthy).count();

    println!("  {}", "Summary".yellow().bold());
    println!("    Total Agents:   {}", adapters.len());
    println!("    Connected:      {} / {}", connected, adapters.len());
    println!("    Healthy:        {} / {}", healthy, adapters.len());

    Ok(())
}

fn format_tokens(tokens: i64) -> String {
    if tokens >= 1_000_000 {
        format!("{:.1}M", tokens as f64 / 1_000_000.0)
    } else if tokens >= 1_000 {
        format!("{:.1}K", tokens as f64 / 1_000.0)
    } else {
        tokens.to_string()
    }
}

async fn cmd_agents_scan(manager: &AdapterManager) -> anyhow::Result<()> {
    println!("{}", "Scanning for installed agents...".cyan().bold());
    println!();

    let registered = manager.discover_and_register_adapters().await?;

    if registered.is_empty() {
        println!("  {} No new agents discovered", "!".yellow());
        println!();

        let existing = manager.list_adapters().await;
        if !existing.is_empty() {
            println!("  Already registered agents:");
            for name in &existing {
                println!("    - {}", name);
            }
        } else {
            println!("  No agents are installed on this system.");
            println!();
            println!("  Supported agents:");
            println!("    - Claude Code (install from https://claude.ai/code)");
            println!("    - OpenCode (install from https://opencode.ai)");
        }
    } else {
        println!(
            "  {} Discovered {} agent(s):",
            "✓".green(),
            registered.len()
        );
        for name in &registered {
            let by_type = manager.list_adapters_by_type().await;
            let agent_type = get_agent_type_for_adapter(name, &by_type);
            println!(
                "    {} {} ({})",
                get_agent_icon(&agent_type),
                name,
                format_agent_type(&agent_type)
            );
        }
    }

    println!();
    println!(
        "{}",
        "Run 'rimuru agents list' to see all registered agents.".dimmed()
    );

    Ok(())
}

async fn cmd_agents_remove(manager: &AdapterManager, name: &str) -> anyhow::Result<()> {
    let adapters = manager.list_adapters().await;

    if !adapters.contains(&name.to_string()) {
        println!("{} Agent '{}' not found", "✗".red(), name);
        return Ok(());
    }

    println!("  {} Removing agent '{}'...", "→".blue(), name);

    manager.unregister_adapter(name).await?;

    println!("  {} Agent '{}' has been removed", "✓".green(), name);

    Ok(())
}

async fn cmd_agents_reconnect(manager: &AdapterManager, name: &str) -> anyhow::Result<()> {
    let adapters = manager.list_adapters().await;

    if !adapters.contains(&name.to_string()) {
        println!("{} Agent '{}' not found", "✗".red(), name);
        return Ok(());
    }

    println!("  {} Reconnecting agent '{}'...", "→".blue(), name);

    manager.reconnect_adapter(name).await?;

    println!(
        "  {} Agent '{}' reconnected successfully",
        "✓".green(),
        name
    );

    Ok(())
}

fn parse_agent_type(s: &str) -> anyhow::Result<AgentType> {
    match s.to_lowercase().as_str() {
        "claude_code" | "claudecode" | "claude-code" => Ok(AgentType::ClaudeCode),
        "codex" => Ok(AgentType::Codex),
        "copilot" | "github-copilot" => Ok(AgentType::Copilot),
        "goose" => Ok(AgentType::Goose),
        "open_code" | "opencode" | "open-code" => Ok(AgentType::OpenCode),
        "cursor" => Ok(AgentType::Cursor),
        _ => Err(anyhow::anyhow!(
            "Unknown agent type: {}. Valid types: claude-code, codex, copilot, goose, opencode, cursor",
            s
        )),
    }
}

fn format_agent_type(agent_type: &AgentType) -> String {
    match agent_type {
        AgentType::ClaudeCode => "Claude Code".to_string(),
        AgentType::Codex => "Codex".to_string(),
        AgentType::Copilot => "Copilot".to_string(),
        AgentType::Goose => "Goose".to_string(),
        AgentType::OpenCode => "OpenCode".to_string(),
        AgentType::Cursor => "Cursor".to_string(),
    }
}

fn get_agent_icon(agent_type: &AgentType) -> &'static str {
    match agent_type {
        AgentType::ClaudeCode => "⟁",
        AgentType::Codex => "◎",
        AgentType::Copilot => "◈",
        AgentType::Goose => "⬡",
        AgentType::OpenCode => "◇",
        AgentType::Cursor => "◫",
    }
}

fn get_agent_type_for_adapter(name: &str, by_type: &HashMap<AgentType, Vec<String>>) -> AgentType {
    for (agent_type, names) in by_type {
        if names.contains(&name.to_string()) {
            return *agent_type;
        }
    }
    AgentType::ClaudeCode
}
