#![allow(
    dead_code,
    unused_imports,
    unused_variables,
    unused_mut,
    clippy::too_many_arguments,
    clippy::needless_borrows_for_generic_args,
    clippy::manual_clamp,
    clippy::useless_format,
    clippy::len_zero,
    clippy::field_reassign_with_default,
    clippy::map_entry,
    clippy::format_in_format_args
)]

use clap::{Parser, Subcommand};
use colored::Colorize;
use rimuru_core::{AgentRepository, Database, DatabaseConfig, SessionRepository, SystemCollector};
use std::process::ExitCode;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod commands;
mod config;

use commands::{
    handle_agents_command, handle_costs_command, handle_hooks_command, handle_models_command,
    handle_plugins_command, handle_sessions_command, handle_skills_command, handle_sync_command,
    AgentsCommand, CostsCommand, HooksCommand, ModelsCommand, PluginsCommand, SessionsCommand,
    SkillsCommand, SyncCommand,
};
use config::CliConfig;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const NAME: &str = env!("CARGO_PKG_NAME");

#[derive(Parser)]
#[command(name = "rimuru")]
#[command(author = "Rohit Ghumare <ghumare64@gmail.com>")]
#[command(version = VERSION)]
#[command(about = "Rimuru - Unified AI Agent Orchestration and Cost Tracking Platform")]
#[command(long_about = r#"
Rimuru is a unified platform for managing AI coding agents, tracking costs,
and monitoring system resources. It supports multiple agent types including
Claude Code, Codex, Copilot, Goose, OpenCode, and Cursor.

Use 'rimuru init' to initialize the database, then 'rimuru status' to view
system metrics and 'rimuru agents' to list registered agents.
"#)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Initialize the database and run migrations")]
    Init {
        #[arg(short, long)]
        force: bool,
    },

    #[command(about = "Show current system metrics (CPU, RAM, active sessions)")]
    Status {
        #[arg(short, long)]
        detailed: bool,

        #[arg(short, long, default_value = "text")]
        format: String,
    },

    #[command(about = "Manage registered AI coding agents")]
    Agents {
        #[command(subcommand)]
        action: Option<AgentsCommand>,
    },

    #[command(about = "Manage and view sessions across all agents")]
    Sessions {
        #[command(subcommand)]
        action: Option<SessionsCommand>,
    },

    #[command(about = "Track and analyze costs across all agents")]
    Costs {
        #[command(subcommand)]
        action: Option<CostsCommand>,
    },

    #[command(about = "Sync model pricing from providers")]
    Sync {
        #[command(subcommand)]
        action: Option<SyncCommand>,
    },

    #[command(about = "List and search models with pricing")]
    Models {
        #[command(subcommand)]
        action: Option<ModelsCommand>,
    },

    #[command(about = "Manage skills from SkillKit marketplace")]
    Skills {
        #[command(subcommand)]
        action: Option<SkillsCommand>,
    },

    #[command(about = "Manage plugins (install, enable, disable, configure)")]
    Plugins {
        #[command(subcommand)]
        action: Option<PluginsCommand>,
    },

    #[command(about = "Manage hooks and view execution history")]
    Hooks {
        #[command(subcommand)]
        action: Option<HooksCommand>,
    },

    #[command(about = "Show version information")]
    Version {
        #[arg(short, long)]
        detailed: bool,
    },
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();

    init_logging(cli.verbose);

    match run(cli).await {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{}: {}", "Error".red().bold(), e);
            ExitCode::FAILURE
        }
    }
}

fn init_logging(verbose: bool) {
    let filter = if verbose {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug"))
    } else {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn"))
    };

    tracing_subscriber::registry()
        .with(fmt::layer().with_target(false))
        .with(filter)
        .init();
}

async fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Commands::Init { force } => cmd_init(force).await,
        Commands::Status { detailed, format } => cmd_status(detailed, &format).await,
        Commands::Agents { action } => handle_agents_command(action).await,
        Commands::Sessions { action } => handle_sessions_command(action).await,
        Commands::Costs { action } => handle_costs_command(action).await,
        Commands::Sync { action } => handle_sync_command(action).await,
        Commands::Models { action } => handle_models_command(action).await,
        Commands::Skills { action } => handle_skills_command(action).await,
        Commands::Plugins { action } => handle_plugins_command(action).await,
        Commands::Hooks { action } => handle_hooks_command(action).await,
        Commands::Version { detailed } => cmd_version(detailed),
    }
}

async fn cmd_init(force: bool) -> anyhow::Result<()> {
    println!("{}", "Initializing Rimuru...".cyan().bold());
    println!();

    let config = CliConfig::load()?;
    println!(
        "  {} Database URL: {}",
        "→".blue(),
        mask_password(&config.database_url)
    );

    println!("  {} Connecting to database...", "→".blue());
    let db_config = DatabaseConfig {
        url: config.database_url,
        ..Default::default()
    };

    let db = Database::connect(&db_config).await?;

    println!("  {} Running migrations...", "→".blue());
    if force {
        println!("    {} Force mode enabled", "!".yellow());
    }

    db.run_migrations().await?;

    println!("  {} Verifying connection...", "→".blue());
    db.health_check().await?;

    db.close().await;

    println!();
    println!(
        "{} {}",
        "✓".green().bold(),
        "Database initialized successfully!".green()
    );

    Ok(())
}

async fn cmd_status(detailed: bool, format: &str) -> anyhow::Result<()> {
    let config = CliConfig::load()?;

    let db_config = DatabaseConfig {
        url: config.database_url,
        ..Default::default()
    };

    let db = Database::connect(&db_config).await?;
    db.health_check().await?;

    let collector = SystemCollector::new();
    let session_repo = SessionRepository::new(db.pool().clone());
    let active_sessions = session_repo.get_active_count().await.unwrap_or(0) as i32;
    let metrics = collector.collect(active_sessions).await;

    let agent_repo = AgentRepository::new(db.pool().clone());
    let total_agents = agent_repo.count().await.unwrap_or(0);

    if format == "json" {
        let output = serde_json::json!({
            "timestamp": metrics.timestamp.to_rfc3339(),
            "cpu_percent": metrics.cpu_percent,
            "memory": {
                "used_mb": metrics.memory_used_mb,
                "total_mb": metrics.memory_total_mb,
                "percent": metrics.memory_percent()
            },
            "active_sessions": metrics.active_sessions,
            "total_agents": total_agents,
            "database": "connected"
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("{}", "Rimuru System Status".cyan().bold());
        println!("{}", "═".repeat(40).dimmed());
        println!();

        println!("  {} {}", "Database:".bold(), "Connected".green());
        println!();

        println!("  {}", "System Metrics".yellow().bold());
        println!("    CPU Usage:     {:>6.1}%", metrics.cpu_percent);

        let mem_percent = metrics.memory_percent();
        println!(
            "    Memory:        {:>6} MB / {} MB ({:.1}%)",
            metrics.memory_used_mb, metrics.memory_total_mb, mem_percent
        );

        if detailed {
            println!();
            println!("  {}", "Details".yellow().bold());
            println!("    CPU Cores:     {:>6}", collector.get_cpu_count().await);
            if let Some(name) = collector.get_system_name().await {
                println!("    System:        {:>6}", name);
            }
            if let Some(os) = collector.get_os_version().await {
                println!("    OS Version:    {:>6}", os);
            }
            if let Some(host) = collector.get_host_name().await {
                println!("    Hostname:      {:>6}", host);
            }
        }

        println!();
        println!("  {}", "Agent Activity".yellow().bold());
        println!("    Active Sessions: {:>4}", metrics.active_sessions);
        println!("    Total Agents:    {:>4}", total_agents);

        println!();
        println!(
            "  {} {}",
            "Timestamp:".dimmed(),
            metrics.timestamp.format("%Y-%m-%d %H:%M:%S UTC")
        );
    }

    db.close().await;
    Ok(())
}

fn cmd_version(detailed: bool) -> anyhow::Result<()> {
    if detailed {
        println!("{}", "Rimuru Version Information".cyan().bold());
        println!("{}", "═".repeat(40).dimmed());
        println!("  {:<15} {}", "Version:".bold(), VERSION);
        println!("  {:<15} {}", "Name:".bold(), NAME);
        println!("  {:<15} Apache-2.0", "License:".bold());
        println!(
            "  {:<15} https://github.com/rohitg00/rimuru",
            "Repository:".bold()
        );
        println!();
        println!("  {}", "Supported Agents:".bold());
        println!("    ⟁ Claude Code");
        println!("    ◎ Codex (OpenAI)");
        println!("    ◈ GitHub Copilot");
        println!("    ⬡ Goose");
        println!("    ◇ OpenCode");
        println!("    ◫ Cursor");
        println!();
        println!("  {}", "Build Information:".bold());
        println!("    Rust Edition: 2021");
        #[cfg(debug_assertions)]
        println!("    Build:        Debug");
        #[cfg(not(debug_assertions))]
        println!("    Build:        Release");
    } else {
        println!("rimuru {}", VERSION);
    }

    Ok(())
}

fn mask_password(url: &str) -> String {
    if let Some(at_pos) = url.find('@') {
        if let Some(proto_end) = url.find("://") {
            let before_auth = &url[..proto_end + 3];
            let after_at = &url[at_pos..];
            if let Some(colon_pos) = url[proto_end + 3..at_pos].find(':') {
                let user = &url[proto_end + 3..proto_end + 3 + colon_pos];
                return format!("{}{}:****{}", before_auth, user, after_at);
            }
        }
    }
    url.to_string()
}
