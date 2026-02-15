use clap::Subcommand;
use colored::Colorize;
use comfy_table::{modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, Cell, Color, Table};
use rimuru_core::{
    list_builtin_plugins, PluginCapability, PluginConfig, PluginLoader, PluginState, PluginStatus,
};

#[derive(Subcommand)]
pub enum PluginsCommand {
    #[command(about = "List all plugins (installed, enabled, available)")]
    List {
        #[arg(short, long, help = "Show all including disabled plugins")]
        all: bool,

        #[arg(
            short,
            long,
            default_value = "text",
            help = "Output format (text, json)"
        )]
        format: String,

        #[arg(
            short,
            long,
            help = "Filter by capability (agent, exporter, notifier, view, hook)"
        )]
        capability: Option<String>,
    },

    #[command(about = "Install a plugin from path/URL/registry")]
    Install {
        #[arg(help = "Plugin source (local path, URL, or registry name)")]
        source: String,

        #[arg(short, long, help = "Enable plugin after installation")]
        enable: bool,
    },

    #[command(about = "Enable a plugin")]
    Enable {
        #[arg(help = "Plugin name or ID")]
        name: String,
    },

    #[command(about = "Disable a plugin")]
    Disable {
        #[arg(help = "Plugin name or ID")]
        name: String,
    },

    #[command(about = "Uninstall a plugin")]
    Uninstall {
        #[arg(help = "Plugin name or ID")]
        name: String,

        #[arg(short, long, help = "Force uninstall without confirmation")]
        force: bool,
    },

    #[command(about = "Show detailed plugin information")]
    Info {
        #[arg(help = "Plugin name or ID")]
        name: String,
    },

    #[command(about = "Configure plugin settings")]
    Config {
        #[arg(help = "Plugin name or ID")]
        name: String,

        #[arg(short, long, help = "Setting key to configure")]
        key: Option<String>,

        #[arg(short, long, help = "Value to set")]
        value: Option<String>,

        #[arg(long, help = "Show current configuration")]
        show: bool,
    },

    #[command(about = "Show available built-in plugins")]
    Builtin,
}

pub async fn handle_plugins_command(cmd: Option<PluginsCommand>) -> anyhow::Result<()> {
    let loader = PluginLoader::with_default_dir()?;

    match cmd.unwrap_or(PluginsCommand::List {
        all: false,
        format: "text".to_string(),
        capability: None,
    }) {
        PluginsCommand::List {
            all,
            format,
            capability,
        } => cmd_plugins_list(&loader, all, &format, capability.as_deref()).await,
        PluginsCommand::Install { source, enable } => {
            cmd_plugins_install(&loader, &source, enable).await
        }
        PluginsCommand::Enable { name } => cmd_plugins_enable(&loader, &name).await,
        PluginsCommand::Disable { name } => cmd_plugins_disable(&loader, &name).await,
        PluginsCommand::Uninstall { name, force } => {
            cmd_plugins_uninstall(&loader, &name, force).await
        }
        PluginsCommand::Info { name } => cmd_plugins_info(&loader, &name).await,
        PluginsCommand::Config {
            name,
            key,
            value,
            show,
        } => cmd_plugins_config(&loader, &name, key.as_deref(), value.as_deref(), show).await,
        PluginsCommand::Builtin => cmd_plugins_builtin().await,
    }
}

async fn cmd_plugins_list(
    loader: &PluginLoader,
    show_all: bool,
    format: &str,
    capability_filter: Option<&str>,
) -> anyhow::Result<()> {
    loader.ensure_plugins_dir().await?;
    let _ = loader.load_all_plugins().await;
    let plugins = loader.get_all_plugins().await;

    let capability_filter = capability_filter.and_then(parse_capability);

    let filtered_plugins: Vec<&PluginState> = plugins
        .iter()
        .filter(|p| {
            if !show_all && p.status == PluginStatus::Disabled {
                return false;
            }
            if let Some(ref cap) = capability_filter {
                return p.info.capabilities.contains(cap);
            }
            true
        })
        .collect();

    if format == "json" {
        let output: Vec<serde_json::Value> = filtered_plugins
            .iter()
            .map(|p| {
                serde_json::json!({
                    "name": p.info.name,
                    "version": p.info.version,
                    "author": p.info.author,
                    "description": p.info.description,
                    "status": format!("{:?}", p.status),
                    "enabled": p.config.enabled,
                    "capabilities": p.info.capabilities.iter().map(|c| c.to_string()).collect::<Vec<_>>(),
                    "loaded_at": p.loaded_at.map(|t| t.to_rfc3339()),
                    "error": p.error
                })
            })
            .collect();

        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(());
    }

    println!("{}", "Installed Plugins".cyan().bold());
    println!();

    if filtered_plugins.is_empty() {
        println!("{}", "No plugins installed.".yellow());
        println!();
        println!(
            "{}",
            "Run 'rimuru plugins builtin' to see available built-in plugins.".dimmed()
        );
        println!(
            "{}",
            "Run 'rimuru plugins install <path>' to install a plugin.".dimmed()
        );
        return Ok(());
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec![
            Cell::new("Name").fg(Color::White),
            Cell::new("Version").fg(Color::White),
            Cell::new("Status").fg(Color::White),
            Cell::new("Capabilities").fg(Color::White),
            Cell::new("Author").fg(Color::White),
        ]);

    for plugin in &filtered_plugins {
        let status_cell = match plugin.status {
            PluginStatus::Enabled => Cell::new("Enabled").fg(Color::Green),
            PluginStatus::Disabled => Cell::new("Disabled").fg(Color::Yellow),
            PluginStatus::Loaded => Cell::new("Loaded").fg(Color::Cyan),
            PluginStatus::Error => Cell::new("Error").fg(Color::Red),
            PluginStatus::Unloaded => Cell::new("Unloaded").fg(Color::DarkGrey),
        };

        let capabilities = plugin
            .info
            .capabilities
            .iter()
            .map(format_capability)
            .collect::<Vec<_>>()
            .join(", ");

        table.add_row(vec![
            Cell::new(&plugin.info.name),
            Cell::new(&plugin.info.version),
            status_cell,
            Cell::new(if capabilities.is_empty() {
                "-".to_string()
            } else {
                capabilities
            }),
            Cell::new(if plugin.info.author.is_empty() {
                "-".to_string()
            } else {
                plugin.info.author.clone()
            }),
        ]);
    }

    println!("{table}");
    println!();

    let enabled_count = filtered_plugins
        .iter()
        .filter(|p| p.status == PluginStatus::Enabled)
        .count();
    println!(
        "  Total: {} plugins ({} enabled)",
        filtered_plugins.len(),
        enabled_count
    );

    Ok(())
}

async fn cmd_plugins_install(
    loader: &PluginLoader,
    source: &str,
    enable: bool,
) -> anyhow::Result<()> {
    println!("{}", "Installing plugin...".cyan().bold());
    println!();

    println!("  {} Source: {}", "→".blue(), source);

    match loader.install_plugin(source).await {
        Ok(plugin_id) => {
            println!("  {} Plugin installed: {}", "✓".green(), plugin_id);

            if enable {
                println!("  {} Enabling plugin...", "→".blue());
                if let Err(e) = loader.enable_plugin(&plugin_id).await {
                    println!("  {} Failed to enable: {}", "!".yellow(), e);
                } else {
                    println!("  {} Plugin enabled", "✓".green());
                }
            }

            println!();
            println!(
                "{}",
                format!("Plugin '{}' installed successfully!", plugin_id).green()
            );
        }
        Err(e) => {
            println!("  {} Installation failed: {}", "✗".red(), e);
            return Err(e.into());
        }
    }

    Ok(())
}

async fn cmd_plugins_enable(loader: &PluginLoader, name: &str) -> anyhow::Result<()> {
    let _ = loader.load_all_plugins().await;

    let plugin_id = find_plugin_id(loader, name).await?;

    println!("  {} Enabling plugin '{}'...", "→".blue(), plugin_id);

    loader.enable_plugin(&plugin_id).await?;

    println!("  {} Plugin '{}' enabled", "✓".green(), plugin_id);

    Ok(())
}

async fn cmd_plugins_disable(loader: &PluginLoader, name: &str) -> anyhow::Result<()> {
    let _ = loader.load_all_plugins().await;

    let plugin_id = find_plugin_id(loader, name).await?;

    println!("  {} Disabling plugin '{}'...", "→".blue(), plugin_id);

    loader.disable_plugin(&plugin_id).await?;

    println!("  {} Plugin '{}' disabled", "✓".green(), plugin_id);

    Ok(())
}

async fn cmd_plugins_uninstall(
    loader: &PluginLoader,
    name: &str,
    force: bool,
) -> anyhow::Result<()> {
    let _ = loader.load_all_plugins().await;

    let plugin_id = find_plugin_id(loader, name).await?;

    if !force {
        println!("{} About to uninstall plugin '{}'", "!".yellow(), plugin_id);
        println!("  This will remove all plugin files and configuration.");
        println!("  Use --force to skip this confirmation.");
        println!();
        println!("  Proceeding with uninstall...");
    }

    println!("  {} Uninstalling plugin '{}'...", "→".blue(), plugin_id);

    loader.uninstall_plugin(&plugin_id).await?;

    println!("  {} Plugin '{}' uninstalled", "✓".green(), plugin_id);

    Ok(())
}

async fn cmd_plugins_info(loader: &PluginLoader, name: &str) -> anyhow::Result<()> {
    let _ = loader.load_all_plugins().await;

    let plugin_id = find_plugin_id(loader, name).await?;
    let state = loader.get_plugin_state(&plugin_id).await?;
    let manifest = loader.get_plugin_manifest(&plugin_id).await?;
    let plugin_dir = loader.get_plugin_dir(&plugin_id).await?;

    println!("{}", "Plugin Information".cyan().bold());
    println!("{}", "═".repeat(50).dimmed());
    println!();

    println!("  {:<15} {}", "Name:".bold(), state.info.name);
    println!("  {:<15} {}", "Version:".bold(), state.info.version);
    println!(
        "  {:<15} {}",
        "Author:".bold(),
        if state.info.author.is_empty() {
            "-".to_string()
        } else {
            state.info.author.clone()
        }
    );
    println!("  {:<15} {}", "Description:".bold(), state.info.description);

    println!();
    println!("  {}", "Status".yellow().bold());
    let status_str = match state.status {
        PluginStatus::Enabled => "Enabled".green().to_string(),
        PluginStatus::Disabled => "Disabled".yellow().to_string(),
        PluginStatus::Loaded => "Loaded".cyan().to_string(),
        PluginStatus::Error => "Error".red().to_string(),
        PluginStatus::Unloaded => "Unloaded".dimmed().to_string(),
    };
    println!("  {:<15} {}", "Status:".bold(), status_str);

    if let Some(loaded_at) = state.loaded_at {
        println!(
            "  {:<15} {}",
            "Loaded At:".bold(),
            loaded_at.format("%Y-%m-%d %H:%M:%S UTC")
        );
    }

    if let Some(ref error) = state.error {
        println!();
        println!("  {}", "Error:".red().bold());
        println!("    {}", error);
    }

    println!();
    println!("  {}", "Capabilities".yellow().bold());
    if state.info.capabilities.is_empty() {
        println!("    None");
    } else {
        for cap in &state.info.capabilities {
            println!("    {} {}", "•".blue(), format_capability(cap));
        }
    }

    println!();
    println!("  {}", "Metadata".yellow().bold());
    if let Some(ref homepage) = state.info.homepage {
        println!("  {:<15} {}", "Homepage:".bold(), homepage);
    }
    if let Some(ref repository) = state.info.repository {
        println!("  {:<15} {}", "Repository:".bold(), repository);
    }
    if let Some(ref license) = state.info.license {
        println!("  {:<15} {}", "License:".bold(), license);
    }

    println!();
    println!("  {}", "Files".yellow().bold());
    println!("  {:<15} {:?}", "Plugin Dir:".bold(), plugin_dir);
    println!("  {:<15} {}", "Manifest:".bold(), manifest.plugin.name);

    if !manifest.dependencies.is_empty() {
        println!();
        println!("  {}", "Dependencies".yellow().bold());
        for dep in &manifest.dependencies {
            let optional = if dep.optional { " (optional)" } else { "" };
            println!(
                "    {} {} @ {}{}",
                "•".blue(),
                dep.name,
                dep.version_requirement,
                optional
            );
        }
    }

    Ok(())
}

async fn cmd_plugins_config(
    loader: &PluginLoader,
    name: &str,
    key: Option<&str>,
    value: Option<&str>,
    show: bool,
) -> anyhow::Result<()> {
    let _ = loader.load_all_plugins().await;

    let plugin_id = find_plugin_id(loader, name).await?;
    let state = loader.get_plugin_state(&plugin_id).await?;

    if show || (key.is_none() && value.is_none()) {
        println!("{}", "Plugin Configuration".cyan().bold());
        println!("{}", "═".repeat(50).dimmed());
        println!();

        println!("  {:<15} {}", "Plugin:".bold(), plugin_id);
        println!("  {:<15} {}", "Enabled:".bold(), state.config.enabled);
        println!("  {:<15} {}", "Priority:".bold(), state.config.priority);
        println!();

        println!("  {}", "Settings".yellow().bold());
        if let serde_json::Value::Object(ref map) = state.config.settings {
            if map.is_empty() {
                println!("    No custom settings configured.");
            } else {
                for (k, v) in map {
                    println!("    {}: {}", k.bold(), v);
                }
            }
        }

        return Ok(());
    }

    if let (Some(key), Some(value)) = (key, value) {
        println!("  {} Setting '{}' = '{}'...", "→".blue(), key, value);

        let parsed_value: serde_json::Value = serde_json::from_str(value)
            .unwrap_or_else(|_| serde_json::Value::String(value.to_string()));

        let new_config = PluginConfig::new().with_setting(key, parsed_value);

        let merged_config = PluginConfig {
            enabled: state.config.enabled,
            priority: state.config.priority,
            settings: {
                let mut settings = state.config.settings.clone();
                if let (
                    serde_json::Value::Object(ref mut existing),
                    serde_json::Value::Object(ref new),
                ) = (&mut settings, &new_config.settings)
                {
                    for (k, v) in new {
                        existing.insert(k.clone(), v.clone());
                    }
                }
                settings
            },
        };

        loader.configure_plugin(&plugin_id, merged_config).await?;

        println!(
            "  {} Configuration updated for '{}'",
            "✓".green(),
            plugin_id
        );
    } else {
        println!(
            "{}",
            "Specify both --key and --value to set a configuration value.".yellow()
        );
        println!(
            "{}",
            "Use --show to display current configuration.".dimmed()
        );
    }

    Ok(())
}

async fn cmd_plugins_builtin() -> anyhow::Result<()> {
    let builtin = list_builtin_plugins();

    println!("{}", "Built-in Plugins".cyan().bold());
    println!();

    if builtin.is_empty() {
        println!("{}", "No built-in plugins available.".yellow());
        return Ok(());
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec![
            Cell::new("Name").fg(Color::White),
            Cell::new("Type").fg(Color::White),
            Cell::new("Description").fg(Color::White),
        ]);

    for info in &builtin {
        let plugin_type = info
            .capabilities
            .first()
            .map(format_capability)
            .unwrap_or_else(|| "Unknown".to_string());

        table.add_row(vec![
            Cell::new(&info.name),
            Cell::new(plugin_type),
            Cell::new(&info.description),
        ]);
    }

    println!("{table}");
    println!();
    println!(
        "  {}",
        "Built-in plugins are included with Rimuru and don't need installation.".dimmed()
    );
    println!(
        "  {}",
        "Configure them using the hooks system or plugin configuration.".dimmed()
    );

    Ok(())
}

async fn find_plugin_id(loader: &PluginLoader, name: &str) -> anyhow::Result<String> {
    let ids = loader.get_loaded_plugin_ids().await;

    if ids.contains(&name.to_string()) {
        return Ok(name.to_string());
    }

    for id in &ids {
        if id.starts_with(&format!("{}@", name)) || id.contains(name) {
            return Ok(id.clone());
        }
    }

    Err(anyhow::anyhow!(
        "Plugin '{}' not found. Available plugins: {}",
        name,
        if ids.is_empty() {
            "none".to_string()
        } else {
            ids.join(", ")
        }
    ))
}

fn parse_capability(s: &str) -> Option<PluginCapability> {
    match s.to_lowercase().as_str() {
        "agent" => Some(PluginCapability::Agent),
        "exporter" => Some(PluginCapability::Exporter),
        "notifier" => Some(PluginCapability::Notifier),
        "view" => Some(PluginCapability::View),
        "hook" => Some(PluginCapability::Hook),
        "custom" => Some(PluginCapability::Custom),
        _ => None,
    }
}

fn format_capability(cap: &PluginCapability) -> String {
    match cap {
        PluginCapability::Agent => "Agent".to_string(),
        PluginCapability::Exporter => "Exporter".to_string(),
        PluginCapability::Notifier => "Notifier".to_string(),
        PluginCapability::View => "View".to_string(),
        PluginCapability::Hook => "Hook".to_string(),
        PluginCapability::Custom => "Custom".to_string(),
    }
}
