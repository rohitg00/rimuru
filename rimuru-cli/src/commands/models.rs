use anyhow::Result;
use clap::Subcommand;
use colored::Colorize;
use comfy_table::{presets::UTF8_FULL, Cell, ContentArrangement, Table};
use rimuru_core::{Database, DatabaseConfig, ModelRepository, Repository};

use crate::config::CliConfig;

#[derive(Subcommand)]
pub enum ModelsCommand {
    #[command(about = "List all known models with pricing")]
    List {
        #[arg(short, long, help = "Filter by provider name")]
        provider: Option<String>,

        #[arg(
            short,
            long,
            default_value = "text",
            help = "Output format (text, json)"
        )]
        format: String,
    },

    #[command(about = "Search models by name")]
    Search {
        #[arg(help = "Search query")]
        query: String,
    },

    #[command(about = "Show detailed pricing for a specific model")]
    Pricing {
        #[arg(help = "Model name (e.g., gpt-4o, claude-3-opus)")]
        model: String,

        #[arg(short, long, help = "Provider name (optional if model is unique)")]
        provider: Option<String>,
    },
}

pub async fn handle_models_command(action: Option<ModelsCommand>) -> Result<()> {
    match action {
        Some(ModelsCommand::List { provider, format }) => cmd_models_list(provider, &format).await,
        Some(ModelsCommand::Search { query }) => cmd_models_search(&query).await,
        Some(ModelsCommand::Pricing { model, provider }) => {
            cmd_models_pricing(&model, provider).await
        }
        None => cmd_models_list(None, "text").await,
    }
}

async fn get_db() -> Result<Database> {
    let config = CliConfig::load()?;
    let db_config = DatabaseConfig {
        url: config.database_url,
        ..Default::default()
    };

    let db = Database::connect(&db_config).await?;
    Ok(db)
}

async fn cmd_models_list(provider: Option<String>, format: &str) -> Result<()> {
    let db = get_db().await?;
    let repo = ModelRepository::new(db.pool().clone());

    let models = match &provider {
        Some(p) => repo.get_by_provider(p).await?,
        None => repo.get_all().await?,
    };

    if format == "json" {
        let json = serde_json::to_string_pretty(&models)?;
        println!("{}", json);
        db.close().await;
        return Ok(());
    }

    if models.is_empty() {
        println!("{}", "No models found.".yellow());
        println!();
        println!(
            "Run {} to populate model data.",
            "rimuru sync run".cyan().bold()
        );
        db.close().await;
        return Ok(());
    }

    println!("{}", "Model Pricing".cyan().bold());
    if let Some(p) = &provider {
        println!("Provider: {}", p.yellow());
    }
    println!("{}", "═".repeat(80).dimmed());
    println!();

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Provider").fg(comfy_table::Color::Cyan),
            Cell::new("Model").fg(comfy_table::Color::Cyan),
            Cell::new("Input $/1K").fg(comfy_table::Color::Cyan),
            Cell::new("Output $/1K").fg(comfy_table::Color::Cyan),
            Cell::new("Context").fg(comfy_table::Color::Cyan),
            Cell::new("Last Synced").fg(comfy_table::Color::Cyan),
        ]);

    for model in &models {
        let context = if model.context_window >= 1_000_000 {
            format!("{}M", model.context_window / 1_000_000)
        } else if model.context_window >= 1_000 {
            format!("{}K", model.context_window / 1_000)
        } else {
            model.context_window.to_string()
        };

        table.add_row(vec![
            Cell::new(&model.provider),
            Cell::new(&model.model_name),
            Cell::new(format!("${:.6}", model.input_price_per_1k)),
            Cell::new(format!("${:.6}", model.output_price_per_1k)),
            Cell::new(context),
            Cell::new(model.last_synced.format("%Y-%m-%d").to_string()),
        ]);
    }

    println!("{}", table);
    println!();
    println!("  Total: {} models", models.len());

    db.close().await;
    Ok(())
}

async fn cmd_models_search(query: &str) -> Result<()> {
    let db = get_db().await?;
    let repo = ModelRepository::new(db.pool().clone());

    let models = repo.search_by_name(query).await?;

    if models.is_empty() {
        println!("No models found matching '{}'", query.yellow());
        db.close().await;
        return Ok(());
    }

    println!("{} {}", "Search Results for:".cyan().bold(), query.yellow());
    println!("{}", "═".repeat(60).dimmed());
    println!();

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Provider").fg(comfy_table::Color::Cyan),
            Cell::new("Model").fg(comfy_table::Color::Cyan),
            Cell::new("Input $/1K").fg(comfy_table::Color::Cyan),
            Cell::new("Output $/1K").fg(comfy_table::Color::Cyan),
            Cell::new("Context").fg(comfy_table::Color::Cyan),
        ]);

    for model in &models {
        let context = format_context_window(model.context_window);

        table.add_row(vec![
            Cell::new(&model.provider),
            Cell::new(&model.model_name),
            Cell::new(format!("${:.6}", model.input_price_per_1k)),
            Cell::new(format!("${:.6}", model.output_price_per_1k)),
            Cell::new(context),
        ]);
    }

    println!("{}", table);
    println!();
    println!("  Found: {} models", models.len());

    db.close().await;
    Ok(())
}

async fn cmd_models_pricing(model: &str, provider: Option<String>) -> Result<()> {
    let db = get_db().await?;
    let repo = ModelRepository::new(db.pool().clone());

    let model_info = if let Some(p) = &provider {
        repo.get_by_name(p, model).await?
    } else {
        let results = repo.search_by_name(model).await?;
        results
            .into_iter()
            .find(|m| m.model_name.to_lowercase() == model.to_lowercase())
    };

    match model_info {
        Some(info) => {
            println!(
                "{} {}",
                "Model Pricing:".cyan().bold(),
                info.full_name().yellow()
            );
            println!("{}", "═".repeat(50).dimmed());
            println!();

            println!("  {:<20} {}", "Provider:".bold(), info.provider);
            println!("  {:<20} {}", "Model:".bold(), info.model_name);
            println!();
            println!("  {}", "Pricing per 1K tokens:".yellow().bold());
            println!("    {:<18} ${:.6}", "Input:", info.input_price_per_1k);
            println!("    {:<18} ${:.6}", "Output:", info.output_price_per_1k);
            println!();
            println!(
                "  {:<20} {}",
                "Context Window:".bold(),
                format_context_window(info.context_window)
            );
            println!(
                "  {:<20} {}",
                "Last Synced:".bold(),
                info.last_synced.format("%Y-%m-%d %H:%M:%S UTC")
            );

            println!();
            println!("  {}", "Cost Examples:".yellow().bold());

            let examples = [
                ("1K input + 1K output", 1000i64, 1000i64),
                ("10K input + 2K output", 10000, 2000),
                ("100K input + 10K output", 100000, 10000),
            ];

            for (desc, input, output) in examples {
                let cost = info.calculate_cost(input, output);
                println!("    {:<25} ${:.6}", desc, cost);
            }
        }
        None => {
            println!("{} Model '{}' not found", "✗".red().bold(), model.yellow());

            let suggestions = repo.search_by_name(model).await?;
            if !suggestions.is_empty() {
                println!();
                println!("  Did you mean one of these?");
                for s in suggestions.iter().take(5) {
                    println!("    - {}", s.full_name().dimmed());
                }
            }
        }
    }

    db.close().await;
    Ok(())
}

fn format_context_window(tokens: i32) -> String {
    if tokens >= 1_000_000 {
        format!("{:.1}M tokens", tokens as f64 / 1_000_000.0)
    } else if tokens >= 1_000 {
        format!("{}K tokens", tokens / 1_000)
    } else {
        format!("{} tokens", tokens)
    }
}
