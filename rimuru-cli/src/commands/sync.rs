use anyhow::Result;
use chrono::{DateTime, Utc};
use clap::Subcommand;
use colored::Colorize;
use comfy_table::{presets::UTF8_FULL, Cell, ContentArrangement, Table};
use rimuru_core::{
    AnthropicSyncProvider, BackgroundSyncScheduler, Database, DatabaseConfig, GoogleSyncProvider,
    LiteLLMSyncProvider, ModelRepository, OpenAISyncProvider, OpenRouterSyncProvider,
    SyncModuleConfig, SyncScheduler,
};
use std::sync::Arc;

use crate::config::CliConfig;

#[derive(Subcommand)]
pub enum SyncCommand {
    #[command(about = "Manually trigger a full model sync")]
    Run {
        #[arg(short, long, help = "Sync only a specific provider")]
        provider: Option<String>,
    },

    #[command(about = "Show sync status and last sync times")]
    Status,

    #[command(about = "Show sync history with success/failure counts")]
    History {
        #[arg(short, long, default_value = "10", help = "Number of entries to show")]
        limit: usize,
    },
}

pub async fn handle_sync_command(action: Option<SyncCommand>) -> Result<()> {
    match action {
        Some(SyncCommand::Run { provider }) => cmd_sync_run(provider).await,
        Some(SyncCommand::Status) => cmd_sync_status().await,
        Some(SyncCommand::History { limit }) => cmd_sync_history(limit).await,
        None => cmd_sync_status().await,
    }
}

async fn create_scheduler() -> Result<(BackgroundSyncScheduler, Database)> {
    let config = CliConfig::load()?;
    let db_config = DatabaseConfig {
        url: config.database_url,
        ..Default::default()
    };

    let db = Database::connect(&db_config).await?;
    let model_repo = ModelRepository::new(db.pool().clone());
    let sync_config = SyncModuleConfig::default();

    let scheduler = BackgroundSyncScheduler::new(sync_config, model_repo);

    scheduler
        .register_provider(Arc::new(AnthropicSyncProvider::new()))
        .await;
    scheduler
        .register_provider(Arc::new(OpenAISyncProvider::new()))
        .await;
    scheduler
        .register_provider(Arc::new(GoogleSyncProvider::new()))
        .await;
    scheduler
        .register_provider(Arc::new(OpenRouterSyncProvider::new()))
        .await;
    scheduler
        .register_provider(Arc::new(LiteLLMSyncProvider::new()))
        .await;

    Ok((scheduler, db))
}

async fn cmd_sync_run(provider: Option<String>) -> Result<()> {
    let (scheduler, db) = create_scheduler().await?;

    match provider {
        Some(provider_name) => {
            println!(
                "{} {} {}",
                "→".blue(),
                "Syncing provider:".cyan(),
                provider_name.yellow()
            );

            match scheduler.trigger_provider_sync(&provider_name).await {
                Ok(result) => {
                    if result.success {
                        println!(
                            "{} Sync completed successfully for {}",
                            "✓".green().bold(),
                            provider_name.green()
                        );
                        println!(
                            "  Models added: {}, updated: {}, unchanged: {}",
                            result.models_added, result.models_updated, result.models_unchanged
                        );
                        println!("  Duration: {}ms", result.duration_ms);
                    } else {
                        println!(
                            "{} Sync failed for {}",
                            "✗".red().bold(),
                            provider_name.red()
                        );
                        for error in &result.errors {
                            println!("  {} {}: {}", "Error".red(), error.code, error.message);
                        }
                    }
                }
                Err(e) => {
                    println!(
                        "{} Failed to sync {}: {}",
                        "✗".red().bold(),
                        provider_name.red(),
                        e
                    );
                }
            }
        }
        None => {
            println!("{} {}", "→".blue(), "Running full sync...".cyan());

            match scheduler.trigger_sync().await {
                Ok(result) => {
                    if result.success {
                        println!("{} Full sync completed successfully", "✓".green().bold());
                    } else {
                        println!(
                            "{} Full sync completed with {} errors",
                            "!".yellow().bold(),
                            result.errors.len()
                        );
                    }
                    println!();
                    println!(
                        "  Total models: {} (added: {}, updated: {}, unchanged: {})",
                        result.total_models(),
                        result.models_added,
                        result.models_updated,
                        result.models_unchanged
                    );
                    println!("  Duration: {}ms", result.duration_ms);

                    if !result.errors.is_empty() {
                        println!();
                        println!("  {}", "Errors:".red().bold());
                        for error in &result.errors {
                            println!(
                                "    {} {}: {}",
                                if error.recoverable { "⚠" } else { "✗" },
                                error.code,
                                error.message
                            );
                        }
                    }
                }
                Err(e) => {
                    println!("{} Full sync failed: {}", "✗".red().bold(), e);
                }
            }
        }
    }

    db.close().await;
    Ok(())
}

async fn cmd_sync_status() -> Result<()> {
    let (scheduler, db) = create_scheduler().await?;

    let status = scheduler.get_status().await;

    println!("{}", "Sync Status".cyan().bold());
    println!("{}", "═".repeat(50).dimmed());
    println!();

    println!(
        "  {:<20} {}",
        "Running:".bold(),
        if status.is_running {
            "Yes".green()
        } else {
            "No".yellow()
        }
    );

    if let Some(last_sync) = status.last_full_sync {
        println!(
            "  {:<20} {}",
            "Last Full Sync:".bold(),
            format_datetime(&last_sync)
        );
    } else {
        println!("  {:<20} {}", "Last Full Sync:".bold(), "Never".dimmed());
    }

    if let Some(next_sync) = status.next_scheduled_sync {
        println!(
            "  {:<20} {}",
            "Next Scheduled:".bold(),
            format_datetime(&next_sync)
        );
    }

    println!();
    println!("  {}", "Provider Status:".yellow().bold());

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Provider").fg(comfy_table::Color::Cyan),
            Cell::new("Enabled").fg(comfy_table::Color::Cyan),
            Cell::new("Last Sync").fg(comfy_table::Color::Cyan),
            Cell::new("Status").fg(comfy_table::Color::Cyan),
            Cell::new("Models").fg(comfy_table::Color::Cyan),
        ]);

    for (name, provider_status) in &status.provider_status {
        let last_sync = provider_status
            .last_sync
            .map(|t| format_datetime(&t))
            .unwrap_or_else(|| "Never".to_string());

        let status_str = if provider_status.last_success {
            "OK".green().to_string()
        } else {
            format!("Failed ({})", provider_status.consecutive_failures)
                .red()
                .to_string()
        };

        table.add_row(vec![
            Cell::new(name),
            Cell::new(if provider_status.enabled {
                "✓"
            } else {
                "✗"
            }),
            Cell::new(last_sync),
            Cell::new(status_str),
            Cell::new(provider_status.models_count.to_string()),
        ]);
    }

    println!("{}", table);

    db.close().await;
    Ok(())
}

async fn cmd_sync_history(limit: usize) -> Result<()> {
    let (scheduler, db) = create_scheduler().await?;

    let history = scheduler.get_history(limit).await;

    println!("{}", "Sync History".cyan().bold());
    println!("{}", "═".repeat(50).dimmed());
    println!();

    if history.is_empty() {
        println!("  No sync history available");
        db.close().await;
        return Ok(());
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Time").fg(comfy_table::Color::Cyan),
            Cell::new("Provider").fg(comfy_table::Color::Cyan),
            Cell::new("Status").fg(comfy_table::Color::Cyan),
            Cell::new("Models").fg(comfy_table::Color::Cyan),
            Cell::new("Duration").fg(comfy_table::Color::Cyan),
        ]);

    for entry in history.iter().rev() {
        let status = if entry.success {
            "✓ Success".green().to_string()
        } else {
            format!(
                "✗ {}",
                entry
                    .error_message
                    .as_deref()
                    .unwrap_or("Failed")
                    .chars()
                    .take(30)
                    .collect::<String>()
            )
            .red()
            .to_string()
        };

        table.add_row(vec![
            Cell::new(format_datetime(&entry.timestamp)),
            Cell::new(&entry.provider),
            Cell::new(status),
            Cell::new(entry.models_synced.to_string()),
            Cell::new(format!("{}ms", entry.duration_ms)),
        ]);
    }

    println!("{}", table);

    let success_count = history.iter().filter(|e| e.success).count();
    let failure_count = history.len() - success_count;

    println!();
    println!(
        "  Summary: {} successful, {} failed",
        success_count.to_string().green(),
        failure_count.to_string().red()
    );

    db.close().await;
    Ok(())
}

fn format_datetime(dt: &DateTime<Utc>) -> String {
    dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}
