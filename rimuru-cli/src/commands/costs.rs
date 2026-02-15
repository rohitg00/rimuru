use clap::Subcommand;
use colored::Colorize;
use comfy_table::{modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, Cell, Color, Table};
use rimuru_core::{AdapterManager, AdapterManagerConfig, AgentType, TimeRange};

#[derive(Subcommand)]
pub enum CostsCommand {
    #[command(about = "Show today's costs by agent")]
    Today {
        #[arg(
            short,
            long,
            default_value = "text",
            help = "Output format (text, json)"
        )]
        format: String,
    },

    #[command(about = "Show cost summary over a time period")]
    Summary {
        #[arg(
            short = 'r',
            long,
            default_value = "7d",
            help = "Time range (today, 7d, 30d, all)"
        )]
        range: String,

        #[arg(
            short,
            long,
            default_value = "text",
            help = "Output format (text, json)"
        )]
        format: String,
    },

    #[command(about = "Breakdown costs by model across all agents")]
    ByModel {
        #[arg(
            short = 'r',
            long,
            default_value = "7d",
            help = "Time range (today, 7d, 30d, all)"
        )]
        range: String,

        #[arg(
            short,
            long,
            default_value = "text",
            help = "Output format (text, json)"
        )]
        format: String,
    },

    #[command(about = "Show costs for a specific agent")]
    Agent {
        #[arg(help = "Agent name")]
        name: String,

        #[arg(
            short = 'r',
            long,
            default_value = "7d",
            help = "Time range (today, 7d, 30d, all)"
        )]
        range: String,

        #[arg(
            short,
            long,
            default_value = "text",
            help = "Output format (text, json)"
        )]
        format: String,
    },
}

pub async fn handle_costs_command(cmd: Option<CostsCommand>) -> anyhow::Result<()> {
    let manager = create_adapter_manager().await?;

    match cmd.unwrap_or(CostsCommand::Today {
        format: "text".to_string(),
    }) {
        CostsCommand::Today { format } => cmd_costs_today(&manager, &format).await,
        CostsCommand::Summary { range, format } => {
            cmd_costs_summary(&manager, &range, &format).await
        }
        CostsCommand::ByModel { range, format } => {
            cmd_costs_by_model(&manager, &range, &format).await
        }
        CostsCommand::Agent {
            name,
            range,
            format,
        } => cmd_costs_agent(&manager, &name, &range, &format).await,
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

async fn cmd_costs_today(manager: &AdapterManager, format: &str) -> anyhow::Result<()> {
    let aggregator = manager.cost_aggregator();
    let report = aggregator.get_cost_report(TimeRange::Today).await?;

    if format == "json" {
        println!("{}", serde_json::to_string_pretty(&report)?);
        return Ok(());
    }

    println!("{}", "Today's Costs".cyan().bold());
    println!(
        "{}",
        format!("Date: {}", chrono::Utc::now().format("%Y-%m-%d")).dimmed()
    );
    println!();

    if report.by_agent.is_empty() {
        println!("{}", "No cost data for today.".yellow());
        println!(
            "{}",
            "Start using your AI agents to see cost tracking.".dimmed()
        );
        return Ok(());
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec![
            Cell::new("Agent").fg(Color::White),
            Cell::new("Type").fg(Color::White),
            Cell::new("Requests").fg(Color::White),
            Cell::new("Input Tokens").fg(Color::White),
            Cell::new("Output Tokens").fg(Color::White),
            Cell::new("Cost").fg(Color::White),
        ]);

    for breakdown in &report.by_agent {
        table.add_row(vec![
            Cell::new(format!(
                "{} {}",
                get_agent_icon(&breakdown.agent_type),
                &breakdown.agent_name
            )),
            Cell::new(format_agent_type(&breakdown.agent_type)),
            Cell::new(breakdown.requests.to_string()),
            Cell::new(format_tokens(breakdown.input_tokens)),
            Cell::new(format_tokens(breakdown.output_tokens)),
            Cell::new(format!("${:.4}", breakdown.total_cost)).fg(Color::Yellow),
        ]);
    }

    println!("{table}");
    println!();

    println!("  {}", "Total".yellow().bold());
    println!("    Requests:       {}", report.total_requests);
    println!(
        "    Input Tokens:   {}",
        format_tokens(report.total_input_tokens)
    );
    println!(
        "    Output Tokens:  {}",
        format_tokens(report.total_output_tokens)
    );
    println!(
        "    {}         {}",
        "Total Cost:".bold(),
        format!("${:.4}", report.total_cost).yellow()
    );

    Ok(())
}

async fn cmd_costs_summary(
    manager: &AdapterManager,
    range: &str,
    format: &str,
) -> anyhow::Result<()> {
    let time_range = parse_time_range(range)?;
    let aggregator = manager.cost_aggregator();
    let report = aggregator.get_cost_report(time_range).await?;

    if format == "json" {
        println!("{}", serde_json::to_string_pretty(&report)?);
        return Ok(());
    }

    println!("{}", "Cost Summary".cyan().bold());
    println!(
        "{}",
        format!("Time range: {}", format_time_range(&time_range)).dimmed()
    );
    println!();

    if report.by_agent.is_empty() {
        println!("{}", "No cost data for this period.".yellow());
        return Ok(());
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS);

    table.add_row(vec![
        Cell::new("Metric").fg(Color::White),
        Cell::new("Value").fg(Color::White),
    ]);

    table.add_row(vec![
        Cell::new("Total Requests"),
        Cell::new(report.total_requests.to_string()),
    ]);

    table.add_row(vec![
        Cell::new("Total Input Tokens"),
        Cell::new(format_tokens(report.total_input_tokens)),
    ]);

    table.add_row(vec![
        Cell::new("Total Output Tokens"),
        Cell::new(format_tokens(report.total_output_tokens)),
    ]);

    table.add_row(vec![
        Cell::new("Total Tokens"),
        Cell::new(format_tokens(report.total_tokens)),
    ]);

    table.add_row(vec![
        Cell::new("Input Cost"),
        Cell::new(format!("${:.4}", report.total_input_cost)),
    ]);

    table.add_row(vec![
        Cell::new("Output Cost"),
        Cell::new(format!("${:.4}", report.total_output_cost)),
    ]);

    table.add_row(vec![
        Cell::new("Total Cost"),
        Cell::new(format!("${:.4}", report.total_cost)).fg(Color::Yellow),
    ]);

    table.add_row(vec![
        Cell::new("Avg Cost/Request"),
        Cell::new(format!("${:.6}", report.average_cost_per_request())),
    ]);

    table.add_row(vec![
        Cell::new("Avg Tokens/Request"),
        Cell::new(format!("{:.0}", report.average_tokens_per_request())),
    ]);

    println!("{table}");

    if !report.by_agent_type.is_empty() {
        println!();
        println!("{}", "By Agent Type".yellow().bold());
        println!();

        let mut type_table = Table::new();
        type_table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_header(vec![
                Cell::new("Type").fg(Color::White),
                Cell::new("Requests").fg(Color::White),
                Cell::new("Tokens").fg(Color::White),
                Cell::new("Cost").fg(Color::White),
            ]);

        for (agent_type, breakdown) in &report.by_agent_type {
            type_table.add_row(vec![
                Cell::new(format!(
                    "{} {}",
                    get_agent_icon(agent_type),
                    format_agent_type(agent_type)
                )),
                Cell::new(breakdown.requests.to_string()),
                Cell::new(format_tokens(breakdown.total_tokens)),
                Cell::new(format!("${:.4}", breakdown.total_cost)),
            ]);
        }

        println!("{type_table}");
    }

    Ok(())
}

async fn cmd_costs_by_model(
    manager: &AdapterManager,
    range: &str,
    format: &str,
) -> anyhow::Result<()> {
    let time_range = parse_time_range(range)?;
    let aggregator = manager.cost_aggregator();
    let report = aggregator.get_cost_report(time_range).await?;

    if format == "json" {
        println!("{}", serde_json::to_string_pretty(&report.by_model)?);
        return Ok(());
    }

    println!("{}", "Costs by Model".cyan().bold());
    println!(
        "{}",
        format!("Time range: {}", format_time_range(&time_range)).dimmed()
    );
    println!();

    if report.by_model.is_empty() {
        println!(
            "{}",
            "No model-specific cost data for this period.".yellow()
        );
        return Ok(());
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec![
            Cell::new("Model").fg(Color::White),
            Cell::new("Requests").fg(Color::White),
            Cell::new("Input Tokens").fg(Color::White),
            Cell::new("Output Tokens").fg(Color::White),
            Cell::new("Input Cost").fg(Color::White),
            Cell::new("Output Cost").fg(Color::White),
            Cell::new("Total Cost").fg(Color::White),
        ]);

    let mut sorted_models: Vec<_> = report.by_model.iter().collect();
    sorted_models.sort_by(|a, b| b.1.total_cost.partial_cmp(&a.1.total_cost).unwrap());

    for (model_name, breakdown) in sorted_models {
        table.add_row(vec![
            Cell::new(truncate_string(model_name, 25)),
            Cell::new(breakdown.requests.to_string()),
            Cell::new(format_tokens(breakdown.input_tokens)),
            Cell::new(format_tokens(breakdown.output_tokens)),
            Cell::new(format!("${:.4}", breakdown.input_cost)),
            Cell::new(format!("${:.4}", breakdown.output_cost)),
            Cell::new(format!("${:.4}", breakdown.total_cost)).fg(Color::Yellow),
        ]);
    }

    println!("{table}");
    println!();
    println!(
        "  {} {} models, {} total cost",
        "Summary:".bold(),
        report.by_model.len(),
        format!("${:.4}", report.total_cost).yellow()
    );

    Ok(())
}

async fn cmd_costs_agent(
    manager: &AdapterManager,
    name: &str,
    range: &str,
    format: &str,
) -> anyhow::Result<()> {
    let time_range = parse_time_range(range)?;
    let aggregator = manager.cost_aggregator();
    let report = aggregator.get_cost_report(time_range).await?;

    let agent_breakdown = report.by_agent.iter().find(|b| b.agent_name == name);

    if format == "json" {
        if let Some(breakdown) = agent_breakdown {
            println!("{}", serde_json::to_string_pretty(&breakdown)?);
        } else {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "error": "Agent not found",
                    "agent": name
                }))?
            );
        }
        return Ok(());
    }

    match agent_breakdown {
        Some(breakdown) => {
            println!("{}", "Agent Cost Details".cyan().bold());
            println!(
                "{}",
                format!("Time range: {}", format_time_range(&time_range)).dimmed()
            );
            println!();

            println!(
                "  {:<18} {} {}",
                "Agent:".bold(),
                get_agent_icon(&breakdown.agent_type),
                &breakdown.agent_name
            );
            println!(
                "  {:<18} {}",
                "Type:".bold(),
                format_agent_type(&breakdown.agent_type)
            );
            if let Some(ref model) = breakdown.model_name {
                println!("  {:<18} {}", "Model:".bold(), model);
            }

            println!();
            println!("  {}", "Usage".yellow().bold());
            println!("  {:<18} {}", "Requests:".bold(), breakdown.requests);
            println!(
                "  {:<18} {}",
                "Input Tokens:".bold(),
                format_tokens(breakdown.input_tokens)
            );
            println!(
                "  {:<18} {}",
                "Output Tokens:".bold(),
                format_tokens(breakdown.output_tokens)
            );
            println!(
                "  {:<18} {}",
                "Total Tokens:".bold(),
                format_tokens(breakdown.total_tokens)
            );

            println!();
            println!("  {}", "Cost Breakdown".yellow().bold());
            println!(
                "  {:<18} ${}",
                "Input Cost:".bold(),
                format!("{:.6}", breakdown.input_cost)
            );
            println!(
                "  {:<18} ${}",
                "Output Cost:".bold(),
                format!("{:.6}", breakdown.output_cost)
            );
            println!(
                "  {:<18} {}",
                "Total Cost:".bold(),
                format!("${:.4}", breakdown.total_cost).yellow()
            );
        }
        None => {
            println!(
                "{} Agent '{}' not found or has no cost data",
                "✗".red(),
                name
            );
            println!();
            println!("Available agents with cost data:");
            for b in &report.by_agent {
                println!(
                    "  {} {} ({})",
                    get_agent_icon(&b.agent_type),
                    &b.agent_name,
                    format_agent_type(&b.agent_type)
                );
            }
        }
    }

    Ok(())
}

fn parse_time_range(s: &str) -> anyhow::Result<TimeRange> {
    match s.to_lowercase().as_str() {
        "today" => Ok(TimeRange::Today),
        "yesterday" => Ok(TimeRange::Yesterday),
        "7d" | "week" | "last7days" => Ok(TimeRange::Last7Days),
        "30d" | "month" | "last30days" => Ok(TimeRange::Last30Days),
        "thismonth" => Ok(TimeRange::ThisMonth),
        "lastmonth" => Ok(TimeRange::LastMonth),
        "all" | "alltime" => Ok(TimeRange::AllTime),
        _ => Err(anyhow::anyhow!(
            "Unknown time range: {}. Valid ranges: today, yesterday, 7d, 30d, thismonth, lastmonth, all",
            s
        )),
    }
}

fn format_time_range(range: &TimeRange) -> String {
    match range {
        TimeRange::Today => "Today".to_string(),
        TimeRange::Yesterday => "Yesterday".to_string(),
        TimeRange::Last7Days => "Last 7 days".to_string(),
        TimeRange::Last30Days => "Last 30 days".to_string(),
        TimeRange::ThisMonth => "This month".to_string(),
        TimeRange::LastMonth => "Last month".to_string(),
        TimeRange::AllTime => "All time".to_string(),
        TimeRange::Custom => "Custom".to_string(),
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

fn format_tokens(tokens: i64) -> String {
    if tokens >= 1_000_000 {
        format!("{:.1}M", tokens as f64 / 1_000_000.0)
    } else if tokens >= 1_000 {
        format!("{:.1}K", tokens as f64 / 1_000.0)
    } else {
        tokens.to_string()
    }
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}…", &s[..max_len - 1])
    } else {
        s.to_string()
    }
}
