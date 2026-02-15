use clap::Subcommand;
use colored::Colorize;
use comfy_table::{modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, Cell, Color, Table};
use rimuru_core::{AdapterManager, AdapterManagerConfig, AgentType, TimeRange};

#[derive(Subcommand)]
pub enum SessionsCommand {
    #[command(about = "List sessions across agents")]
    List {
        #[arg(short = 't', long, help = "Filter by agent type")]
        agent_type: Option<String>,

        #[arg(short, long, help = "Filter by agent name")]
        agent: Option<String>,

        #[arg(
            short,
            long,
            default_value = "text",
            help = "Output format (text, json)"
        )]
        format: String,

        #[arg(long, help = "Show only active sessions across all agents")]
        active: bool,
    },

    #[command(about = "Show recent completed sessions")]
    History {
        #[arg(short, long, default_value = "10", help = "Number of sessions to show")]
        limit: usize,

        #[arg(
            short = 'r',
            long,
            default_value = "7d",
            help = "Time range (today, 7d, 30d, all)"
        )]
        range: String,

        #[arg(short = 't', long, help = "Filter by agent type")]
        agent_type: Option<String>,

        #[arg(
            short,
            long,
            default_value = "text",
            help = "Output format (text, json)"
        )]
        format: String,
    },

    #[command(about = "Show session statistics")]
    Stats {
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

    #[command(about = "Show details of a specific session")]
    Show {
        #[arg(help = "Session ID (UUID)")]
        session_id: String,
    },
}

pub async fn handle_sessions_command(cmd: Option<SessionsCommand>) -> anyhow::Result<()> {
    let manager = create_adapter_manager().await?;

    match cmd.unwrap_or(SessionsCommand::List {
        agent_type: None,
        agent: None,
        format: "text".to_string(),
        active: false,
    }) {
        SessionsCommand::List {
            agent_type,
            agent,
            format,
            active,
        } => {
            cmd_sessions_list(
                &manager,
                agent_type.as_deref(),
                agent.as_deref(),
                &format,
                active,
            )
            .await
        }
        SessionsCommand::History {
            limit,
            range,
            agent_type,
            format,
        } => cmd_sessions_history(&manager, limit, &range, agent_type.as_deref(), &format).await,
        SessionsCommand::Stats { range, format } => {
            cmd_sessions_stats(&manager, &range, &format).await
        }
        SessionsCommand::Show { session_id } => cmd_sessions_show(&manager, &session_id).await,
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

async fn cmd_sessions_list(
    manager: &AdapterManager,
    agent_type_filter: Option<&str>,
    agent_filter: Option<&str>,
    format: &str,
    active_only: bool,
) -> anyhow::Result<()> {
    let aggregator = manager.session_aggregator();
    let mut sessions = if active_only {
        aggregator.get_all_active_sessions().await?
    } else {
        let history = aggregator
            .get_session_history(Some(100), TimeRange::Last7Days)
            .await?;
        let active = aggregator.get_all_active_sessions().await?;
        let mut combined = active;
        combined.extend(history.into_iter().filter(|s| !s.is_active));
        combined
    };

    if let Some(type_str) = agent_type_filter {
        let agent_type = parse_agent_type(type_str)?;
        sessions.retain(|s| s.agent_type == agent_type);
    }

    if let Some(agent_name) = agent_filter {
        sessions.retain(|s| s.adapter_name == agent_name);
    }

    if format == "json" {
        println!("{}", serde_json::to_string_pretty(&sessions)?);
        return Ok(());
    }

    if sessions.is_empty() {
        if active_only {
            println!("{}", "No active sessions.".yellow());
        } else {
            println!("{}", "No sessions found.".yellow());
        }
        println!("{}", "Start a session with one of your AI agents.".dimmed());
        return Ok(());
    }

    let title = if active_only {
        "Active Sessions"
    } else {
        "Sessions"
    };
    println!("{}", title.cyan().bold());
    println!();

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec![
            Cell::new("Session ID").fg(Color::White),
            Cell::new("Agent").fg(Color::White),
            Cell::new("Model").fg(Color::White),
            Cell::new("Duration").fg(Color::White),
            Cell::new("Tokens").fg(Color::White),
            Cell::new("Project").fg(Color::White),
        ]);

    for session in &sessions {
        let duration = format_duration(session.duration_seconds.unwrap_or(0));
        let model = session.model_name.as_deref().unwrap_or("-").to_string();
        let project = session
            .project_path
            .as_deref()
            .map(|p| truncate_path(p, 25))
            .unwrap_or_else(|| "-".to_string());

        table.add_row(vec![
            Cell::new(truncate_uuid(&session.session_id.to_string())),
            Cell::new(format!(
                "{} {}",
                get_agent_icon(&session.agent_type),
                &session.adapter_name
            )),
            Cell::new(truncate_string(&model, 20)),
            Cell::new(&duration),
            Cell::new(format_tokens(session.total_tokens)),
            Cell::new(&project),
        ]);
    }

    println!("{table}");
    println!();
    let active_count = sessions.iter().filter(|s| s.is_active).count();
    if active_only {
        println!("  Total: {} active session(s)", sessions.len());
    } else {
        println!(
            "  Total: {} session(s) ({} active)",
            sessions.len(),
            active_count
        );
    }

    Ok(())
}

async fn cmd_sessions_history(
    manager: &AdapterManager,
    limit: usize,
    range: &str,
    agent_type_filter: Option<&str>,
    format: &str,
) -> anyhow::Result<()> {
    let time_range = parse_time_range(range)?;
    let aggregator = manager.session_aggregator();
    let mut sessions = aggregator
        .get_session_history(Some(limit * 2), time_range)
        .await?;

    sessions.retain(|s| !s.is_active);

    if let Some(type_str) = agent_type_filter {
        let agent_type = parse_agent_type(type_str)?;
        sessions.retain(|s| s.agent_type == agent_type);
    }

    sessions.truncate(limit);

    if format == "json" {
        println!("{}", serde_json::to_string_pretty(&sessions)?);
        return Ok(());
    }

    if sessions.is_empty() {
        println!("{}", "No completed sessions found.".yellow());
        return Ok(());
    }

    println!("{}", "Session History".cyan().bold());
    println!(
        "{}",
        format!("Time range: {}", format_time_range(&time_range)).dimmed()
    );
    println!();

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec![
            Cell::new("Session ID").fg(Color::White),
            Cell::new("Agent").fg(Color::White),
            Cell::new("Started").fg(Color::White),
            Cell::new("Duration").fg(Color::White),
            Cell::new("Tokens").fg(Color::White),
            Cell::new("Cost").fg(Color::White),
        ]);

    for session in &sessions {
        let duration = format_duration(session.duration_seconds.unwrap_or(0));
        let started = session.started_at.format("%m/%d %H:%M").to_string();
        let cost = session
            .cost_usd
            .map(|c| format!("${:.4}", c))
            .unwrap_or_else(|| "-".to_string());

        table.add_row(vec![
            Cell::new(truncate_uuid(&session.session_id.to_string())),
            Cell::new(format!(
                "{} {}",
                get_agent_icon(&session.agent_type),
                &session.adapter_name
            )),
            Cell::new(&started),
            Cell::new(&duration),
            Cell::new(format_tokens(session.total_tokens)),
            Cell::new(&cost),
        ]);
    }

    println!("{table}");
    println!();
    println!("  Showing {} of {} sessions", sessions.len(), limit);

    Ok(())
}

async fn cmd_sessions_stats(
    manager: &AdapterManager,
    range: &str,
    format: &str,
) -> anyhow::Result<()> {
    let time_range = parse_time_range(range)?;
    let aggregator = manager.session_aggregator();
    let stats = aggregator.get_stats(time_range).await?;

    if format == "json" {
        println!("{}", serde_json::to_string_pretty(&stats)?);
        return Ok(());
    }

    println!("{}", "Session Statistics".cyan().bold());
    println!(
        "{}",
        format!("Time range: {}", format_time_range(&time_range)).dimmed()
    );
    println!();

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS);

    table.add_row(vec![
        Cell::new("Metric").fg(Color::White),
        Cell::new("Value").fg(Color::White),
    ]);

    table.add_row(vec![
        Cell::new("Total Sessions"),
        Cell::new(stats.total_sessions.to_string()),
    ]);

    table.add_row(vec![
        Cell::new("Active Sessions"),
        Cell::new(stats.active_sessions.to_string()).fg(Color::Green),
    ]);

    table.add_row(vec![
        Cell::new("Completed Sessions"),
        Cell::new(stats.completed_sessions.to_string()),
    ]);

    table.add_row(vec![
        Cell::new("Total Duration"),
        Cell::new(format_duration(stats.total_duration_seconds)),
    ]);

    table.add_row(vec![
        Cell::new("Avg Duration"),
        Cell::new(format_duration(stats.average_duration_seconds as i64)),
    ]);

    table.add_row(vec![
        Cell::new("Total Tokens"),
        Cell::new(format_tokens(stats.total_tokens)),
    ]);

    table.add_row(vec![
        Cell::new("Avg Tokens/Session"),
        Cell::new(format!("{:.0}", stats.average_tokens_per_session)),
    ]);

    table.add_row(vec![
        Cell::new("Total Cost"),
        Cell::new(format!("${:.4}", stats.total_cost)).fg(Color::Yellow),
    ]);

    table.add_row(vec![
        Cell::new("Avg Cost/Session"),
        Cell::new(format!("${:.4}", stats.average_cost_per_session)),
    ]);

    println!("{table}");

    let stats_by_type = aggregator.get_stats_by_agent_type(time_range).await?;
    if !stats_by_type.is_empty() {
        println!();
        println!("{}", "By Agent Type".yellow().bold());
        println!();

        let mut type_table = Table::new();
        type_table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_header(vec![
                Cell::new("Type").fg(Color::White),
                Cell::new("Sessions").fg(Color::White),
                Cell::new("Tokens").fg(Color::White),
                Cell::new("Cost").fg(Color::White),
            ]);

        for (agent_type, type_stats) in &stats_by_type {
            type_table.add_row(vec![
                Cell::new(format!(
                    "{} {}",
                    get_agent_icon(agent_type),
                    format_agent_type(agent_type)
                )),
                Cell::new(type_stats.total_sessions.to_string()),
                Cell::new(format_tokens(type_stats.total_tokens)),
                Cell::new(format!("${:.4}", type_stats.total_cost)),
            ]);
        }

        println!("{type_table}");
    }

    Ok(())
}

async fn cmd_sessions_show(manager: &AdapterManager, session_id_str: &str) -> anyhow::Result<()> {
    let session_id = uuid::Uuid::parse_str(session_id_str)?;
    let aggregator = manager.session_aggregator();

    let session = aggregator.get_session_by_id(session_id).await?;

    match session {
        Some(s) => {
            println!("{}", "Session Details".cyan().bold());
            println!("{}", "═".repeat(50).dimmed());
            println!();

            println!("  {:<18} {}", "Session ID:".bold(), s.session_id);
            println!(
                "  {:<18} {} {}",
                "Agent:".bold(),
                get_agent_icon(&s.agent_type),
                &s.adapter_name
            );
            println!(
                "  {:<18} {}",
                "Type:".bold(),
                format_agent_type(&s.agent_type)
            );

            let status = if s.is_active {
                "Active".green().to_string()
            } else {
                "Completed".dimmed().to_string()
            };
            println!("  {:<18} {}", "Status:".bold(), status);

            println!();
            println!("  {}", "Timing".yellow().bold());
            println!(
                "  {:<18} {}",
                "Started:".bold(),
                s.started_at.format("%Y-%m-%d %H:%M:%S UTC")
            );
            if let Some(ended) = s.ended_at {
                println!(
                    "  {:<18} {}",
                    "Ended:".bold(),
                    ended.format("%Y-%m-%d %H:%M:%S UTC")
                );
            }
            if let Some(duration) = s.duration_seconds {
                println!("  {:<18} {}", "Duration:".bold(), format_duration(duration));
            }

            println!();
            println!("  {}", "Usage".yellow().bold());
            println!(
                "  {:<18} {}",
                "Input Tokens:".bold(),
                format_tokens(s.total_input_tokens)
            );
            println!(
                "  {:<18} {}",
                "Output Tokens:".bold(),
                format_tokens(s.total_output_tokens)
            );
            println!(
                "  {:<18} {}",
                "Total Tokens:".bold(),
                format_tokens(s.total_tokens)
            );

            if let Some(model) = &s.model_name {
                println!("  {:<18} {}", "Model:".bold(), model);
            }

            if let Some(cost) = s.cost_usd {
                println!();
                println!("  {}", "Cost".yellow().bold());
                println!("  {:<18} ${:.6}", "Total Cost:".bold(), cost);
            }

            if let Some(ref path) = s.project_path {
                println!();
                println!("  {}", "Project".yellow().bold());
                println!("  {:<18} {}", "Path:".bold(), path);
            }
        }
        None => {
            println!("{} Session '{}' not found", "✗".red(), session_id_str);
        }
    }

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

fn format_duration(seconds: i64) -> String {
    if seconds < 60 {
        format!("{}s", seconds)
    } else if seconds < 3600 {
        format!("{}m {}s", seconds / 60, seconds % 60)
    } else {
        let hours = seconds / 3600;
        let minutes = (seconds % 3600) / 60;
        format!("{}h {}m", hours, minutes)
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

fn truncate_uuid(uuid: &str) -> String {
    if uuid.len() > 8 {
        format!("{}…", &uuid[..8])
    } else {
        uuid.to_string()
    }
}

fn truncate_path(path: &str, max_len: usize) -> String {
    if path.len() <= max_len {
        return path.to_string();
    }

    if let Some(last_component) = path.rsplit('/').next() {
        if last_component.len() <= max_len - 3 {
            return format!("…/{}", last_component);
        }
    }

    format!("…{}", &path[path.len() - max_len + 1..])
}
