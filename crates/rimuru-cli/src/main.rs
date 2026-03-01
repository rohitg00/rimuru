mod commands;
mod output;

use anyhow::Result;
use clap::{Parser, Subcommand};
use iii_sdk::III;
use output::OutputFormat;

#[derive(Parser)]
#[command(name = "rimuru", version, about = "AI agent orchestration & cost tracking")]
struct Cli {
    #[arg(long, default_value = "ws://127.0.0.1:49134", env = "RIMURU_ENGINE_URL")]
    engine_url: String,

    #[arg(long, default_value = "table", value_enum)]
    format: OutputFormat,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Manage AI agents")]
    Agents {
        #[command(subcommand)]
        action: AgentsAction,
    },

    #[command(about = "Manage sessions")]
    Sessions {
        #[command(subcommand)]
        action: SessionsAction,
    },

    #[command(about = "Cost tracking and reporting")]
    Costs {
        #[command(subcommand)]
        action: CostsAction,
    },

    #[command(about = "Model pricing data")]
    Models {
        #[command(subcommand)]
        action: ModelsAction,
    },

    #[command(about = "System metrics")]
    Metrics {
        #[command(subcommand)]
        action: MetricsAction,
    },

    #[command(about = "Plugin management")]
    Plugins {
        #[command(subcommand)]
        action: PluginsAction,
    },

    #[command(about = "Hook management")]
    Hooks {
        #[command(subcommand)]
        action: HooksAction,
    },

    #[command(about = "MCP servers")]
    Mcp {
        #[command(subcommand)]
        action: McpAction,
    },

    #[command(about = "Configuration")]
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    #[command(about = "Health check")]
    Health,

    #[command(about = "Open the web UI")]
    Ui {
        #[arg(long, default_value = "3100")]
        port: u16,
    },
}

#[derive(Subcommand)]
enum AgentsAction {
    #[command(about = "List all agents")]
    List,
    #[command(about = "Show agent details")]
    Show { agent_id: String },
    #[command(about = "Connect an agent")]
    Connect { agent_type: String },
    #[command(about = "Disconnect an agent")]
    Disconnect { agent_id: String },
    #[command(about = "Detect installed agents")]
    Detect,
}

#[derive(Subcommand)]
enum SessionsAction {
    #[command(about = "List sessions")]
    List,
    #[command(about = "Show session details")]
    Show { session_id: String },
    #[command(about = "Show active sessions")]
    Active,
    #[command(about = "Show session history")]
    History,
}

#[derive(Subcommand)]
enum CostsAction {
    #[command(about = "Cost summary")]
    Summary,
    #[command(about = "Daily cost breakdown")]
    Daily,
    #[command(about = "Cost breakdown by agent")]
    Agent {
        #[arg(long)]
        agent_id: Option<String>,
    },
    #[command(about = "Export cost data")]
    Export { path: String },
}

#[derive(Subcommand)]
enum ModelsAction {
    #[command(about = "List models and pricing")]
    List,
    #[command(about = "Sync model pricing data")]
    Sync,
    #[command(about = "Get model details")]
    Get { model_id: String },
}

#[derive(Subcommand)]
enum MetricsAction {
    #[command(about = "Current system metrics")]
    Current,
    #[command(about = "Metrics history")]
    History,
}

#[derive(Subcommand)]
enum PluginsAction {
    #[command(about = "List installed plugins")]
    List,
    #[command(about = "Install a plugin")]
    Install { path: String },
    #[command(about = "Uninstall a plugin")]
    Uninstall { plugin_id: String },
}

#[derive(Subcommand)]
enum HooksAction {
    #[command(about = "List registered hooks")]
    List,
    #[command(about = "Register a hook")]
    Register {
        event_type: String,
        function_id: String,
        #[arg(long, default_value = "0")]
        priority: i32,
    },
    #[command(about = "Dispatch an event to hooks")]
    Dispatch {
        event_type: String,
        #[arg(long)]
        payload: Option<String>,
    },
}

#[derive(Subcommand)]
enum McpAction {
    #[command(about = "List MCP servers")]
    List,
}

#[derive(Subcommand)]
enum ConfigAction {
    #[command(about = "Get config value")]
    Get { key: Option<String> },
    #[command(about = "Set config value")]
    Set { key: String, value: String },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let format = &cli.format;

    if let Commands::Ui { port } = &cli.command {
        return open_ui(*port);
    }

    let iii = III::new(&cli.engine_url);
    iii.connect().await?;
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    let result = match cli.command {
        Commands::Agents { action } => match action {
            AgentsAction::List => commands::agents::list(&iii, format).await,
            AgentsAction::Show { agent_id } => {
                commands::agents::show(&iii, &agent_id, format).await
            }
            AgentsAction::Connect { agent_type } => {
                commands::agents::connect(&iii, &agent_type, format).await
            }
            AgentsAction::Disconnect { agent_id } => {
                commands::agents::disconnect(&iii, &agent_id, format).await
            }
            AgentsAction::Detect => commands::agents::detect(&iii, format).await,
        },

        Commands::Sessions { action } => match action {
            SessionsAction::List => commands::sessions::list(&iii, format).await,
            SessionsAction::Show { session_id } => {
                commands::sessions::show(&iii, &session_id, format).await
            }
            SessionsAction::Active => commands::sessions::active(&iii, format).await,
            SessionsAction::History => commands::sessions::history(&iii, format).await,
        },

        Commands::Costs { action } => match action {
            CostsAction::Summary => commands::costs::summary(&iii, format).await,
            CostsAction::Daily => commands::costs::daily(&iii, format).await,
            CostsAction::Agent { agent_id } => {
                commands::costs::agent(&iii, agent_id.as_deref(), format).await
            }
            CostsAction::Export { path } => commands::costs::export(&iii, &path).await,
        },

        Commands::Models { action } => match action {
            ModelsAction::List => commands::models::list(&iii, format).await,
            ModelsAction::Sync => commands::models::sync(&iii, format).await,
            ModelsAction::Get { model_id } => {
                commands::models::get(&iii, &model_id, format).await
            }
        },

        Commands::Metrics { action } => match action {
            MetricsAction::Current => commands::metrics::current(&iii, format).await,
            MetricsAction::History => commands::metrics::history(&iii, format).await,
        },

        Commands::Plugins { action } => match action {
            PluginsAction::List => commands::plugins::list(&iii, format).await,
            PluginsAction::Install { path } => {
                commands::plugins::install(&iii, &path, format).await
            }
            PluginsAction::Uninstall { plugin_id } => {
                commands::plugins::uninstall(&iii, &plugin_id, format).await
            }
        },

        Commands::Hooks { action } => match action {
            HooksAction::List => commands::hooks::list(&iii, format).await,
            HooksAction::Register {
                event_type,
                function_id,
                priority,
            } => {
                commands::hooks::register(&iii, &event_type, &function_id, priority, format)
                    .await
            }
            HooksAction::Dispatch {
                event_type,
                payload,
            } => {
                commands::hooks::dispatch(&iii, &event_type, payload.as_deref(), format)
                    .await
            }
        },

        Commands::Mcp { action } => match action {
            McpAction::List => commands::mcp::list(&iii, format).await,
        },

        Commands::Config { action } => match action {
            ConfigAction::Get { key } => {
                commands::config::get(&iii, key.as_deref(), format).await
            }
            ConfigAction::Set { key, value } => {
                commands::config::set(&iii, &key, &value, format).await
            }
        },

        Commands::Health => commands::health::check(&iii, format).await,

        Commands::Ui { .. } => unreachable!(),
    };

    iii.shutdown_async().await;
    result
}

fn open_ui(port: u16) -> Result<()> {
    let url = format!("http://localhost:{port}");
    println!("Opening Rimuru UI at {url}");

    #[cfg(target_os = "macos")]
    std::process::Command::new("open").arg(&url).spawn()?;
    #[cfg(target_os = "linux")]
    std::process::Command::new("xdg-open").arg(&url).spawn()?;

    Ok(())
}
