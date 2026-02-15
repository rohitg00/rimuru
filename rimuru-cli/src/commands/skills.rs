use clap::Subcommand;
use colored::Colorize;
use comfy_table::{modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, Cell, Color, Table};
use rimuru_core::{
    InstalledSkill, PublishResult, SearchFilters, Skill, SkillKitAgent, SkillKitBridge,
    SkillRecommendation, TranslationResult,
};

#[derive(Subcommand)]
pub enum SkillsCommand {
    #[command(about = "Search marketplace for skills")]
    Search {
        #[arg(help = "Search query")]
        query: String,

        #[arg(short, long, help = "Filter by agent type")]
        agent: Option<String>,

        #[arg(short, long, help = "Filter by tags (comma-separated)")]
        tags: Option<String>,

        #[arg(
            short,
            long,
            default_value = "text",
            help = "Output format (text, json)"
        )]
        format: String,

        #[arg(short, long, default_value = "20", help = "Number of results to show")]
        limit: usize,
    },

    #[command(about = "Install a skill for agents")]
    Install {
        #[arg(help = "Skill name to install")]
        name: String,

        #[arg(short, long, help = "Target agent type")]
        agent: Option<String>,

        #[arg(long, help = "Install for all supported agents")]
        all: bool,

        #[arg(
            short,
            long,
            default_value = "text",
            help = "Output format (text, json)"
        )]
        format: String,
    },

    #[command(about = "Uninstall a skill")]
    Uninstall {
        #[arg(help = "Skill name to uninstall")]
        name: String,
    },

    #[command(about = "Translate a skill between agents")]
    Translate {
        #[arg(help = "Skill name to translate")]
        name: String,

        #[arg(long, help = "Source agent type")]
        from: String,

        #[arg(long, help = "Target agent type")]
        to: String,

        #[arg(
            short,
            long,
            default_value = "text",
            help = "Output format (text, json)"
        )]
        format: String,
    },

    #[command(about = "List installed skills")]
    List {
        #[arg(short, long, help = "Filter by agent type")]
        agent: Option<String>,

        #[arg(
            short,
            long,
            default_value = "text",
            help = "Output format (text, json)"
        )]
        format: String,

        #[arg(long, help = "Show only enabled skills")]
        enabled: bool,

        #[arg(long, help = "Show only disabled skills")]
        disabled: bool,
    },

    #[command(about = "Get AI-powered skill recommendations")]
    Recommend {
        #[arg(
            short,
            long,
            help = "Workflow description for context-aware recommendations"
        )]
        workflow: Option<String>,

        #[arg(
            short,
            long,
            default_value = "text",
            help = "Output format (text, json)"
        )]
        format: String,

        #[arg(short, long, default_value = "10", help = "Number of recommendations")]
        limit: usize,
    },

    #[command(about = "Publish a skill to the marketplace")]
    Publish {
        #[arg(help = "Path to skill file (SKILL.md)")]
        path: String,

        #[arg(
            short,
            long,
            default_value = "text",
            help = "Output format (text, json)"
        )]
        format: String,

        #[arg(long, help = "Skip validation before publishing")]
        skip_validation: bool,
    },

    #[command(about = "Show details for a specific skill")]
    Show {
        #[arg(help = "Skill name to show")]
        name: String,

        #[arg(
            short,
            long,
            default_value = "text",
            help = "Output format (text, json)"
        )]
        format: String,
    },

    #[command(about = "Sync local skills with marketplace")]
    Sync,

    #[command(about = "Show marketplace statistics")]
    Stats {
        #[arg(
            short,
            long,
            default_value = "text",
            help = "Output format (text, json)"
        )]
        format: String,
    },

    #[command(about = "List all supported agents")]
    Agents {
        #[arg(
            short,
            long,
            default_value = "text",
            help = "Output format (text, json)"
        )]
        format: String,
    },
}

pub async fn handle_skills_command(cmd: Option<SkillsCommand>) -> anyhow::Result<()> {
    let bridge = SkillKitBridge::new().await?;

    if !bridge.is_available() {
        println!("{} SkillKit is not installed", "✗".red());
        println!();
        println!("Install SkillKit with:");
        println!("  {}", "npm i -g skillkit".cyan());
        println!();
        println!("Or use via npx:");
        println!("  {}", "npx skillkit --help".cyan());
        return Ok(());
    }

    match cmd.unwrap_or(SkillsCommand::List {
        agent: None,
        format: "text".to_string(),
        enabled: false,
        disabled: false,
    }) {
        SkillsCommand::Search {
            query,
            agent,
            tags,
            format,
            limit,
        } => {
            cmd_skills_search(
                &bridge,
                &query,
                agent.as_deref(),
                tags.as_deref(),
                &format,
                limit,
            )
            .await
        }
        SkillsCommand::Install {
            name,
            agent,
            all,
            format,
        } => cmd_skills_install(&bridge, &name, agent.as_deref(), all, &format).await,
        SkillsCommand::Uninstall { name } => cmd_skills_uninstall(&bridge, &name).await,
        SkillsCommand::Translate {
            name,
            from,
            to,
            format,
        } => cmd_skills_translate(&bridge, &name, &from, &to, &format).await,
        SkillsCommand::List {
            agent,
            format,
            enabled,
            disabled,
        } => cmd_skills_list(&bridge, agent.as_deref(), &format, enabled, disabled).await,
        SkillsCommand::Recommend {
            workflow,
            format,
            limit,
        } => cmd_skills_recommend(&bridge, workflow.as_deref(), &format, limit).await,
        SkillsCommand::Publish {
            path,
            format,
            skip_validation,
        } => cmd_skills_publish(&bridge, &path, &format, skip_validation).await,
        SkillsCommand::Show { name, format } => cmd_skills_show(&bridge, &name, &format).await,
        SkillsCommand::Sync => cmd_skills_sync(&bridge).await,
        SkillsCommand::Stats { format } => cmd_skills_stats(&bridge, &format).await,
        SkillsCommand::Agents { format } => cmd_skills_agents(&format).await,
    }
}

async fn cmd_skills_search(
    bridge: &SkillKitBridge,
    query: &str,
    agent: Option<&str>,
    tags: Option<&str>,
    format: &str,
    limit: usize,
) -> anyhow::Result<()> {
    println!(
        "{} {}",
        "Searching marketplace for:".cyan().bold(),
        query.yellow()
    );
    println!();

    let mut filters = SearchFilters::default();
    if let Some(agent_str) = agent {
        if let Some(parsed_agent) = SkillKitAgent::parse(agent_str) {
            filters.agent = Some(parsed_agent);
        } else {
            println!("{} Unknown agent type: {}", "⚠".yellow(), agent_str);
        }
    }
    if let Some(tags_str) = tags {
        filters.tags = tags_str.split(',').map(|s| s.trim().to_string()).collect();
    }

    let result = bridge.search(query, Some(filters)).await?;
    let skills: Vec<&Skill> = result.skills.iter().take(limit).collect();

    if format == "json" {
        let output = serde_json::json!({
            "query": query,
            "total": result.total,
            "skills": skills,
        });
        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(());
    }

    if skills.is_empty() {
        println!("{}", "No skills found matching your query.".yellow());
        println!();
        println!("Try a different search term or browse popular skills with:");
        println!("  {}", "rimuru skills search popular".dimmed());
        return Ok(());
    }

    print_skills_table(&skills);

    println!();
    println!("  Found {} skills (showing {})", result.total, skills.len());
    println!();
    println!(
        "{}",
        "Use 'rimuru skills show <name>' to view skill details.".dimmed()
    );

    Ok(())
}

async fn cmd_skills_install(
    bridge: &SkillKitBridge,
    name: &str,
    agent: Option<&str>,
    all: bool,
    format: &str,
) -> anyhow::Result<()> {
    let agents: Vec<SkillKitAgent> = if all {
        println!(
            "{} {} for all agents...",
            "→".blue(),
            format!("Installing '{}'", name).cyan().bold()
        );
        SkillKitAgent::all().to_vec()
    } else if let Some(agent_str) = agent {
        let parsed_agent = SkillKitAgent::parse(agent_str).ok_or_else(|| {
            anyhow::anyhow!(
                "Unknown agent type: {}. Use 'rimuru skills agents' to see available agents.",
                agent_str
            )
        })?;
        println!(
            "{} {} for {}...",
            "→".blue(),
            format!("Installing '{}'", name).cyan().bold(),
            parsed_agent.display_name()
        );
        vec![parsed_agent]
    } else {
        println!(
            "{} {} (default agent)...",
            "→".blue(),
            format!("Installing '{}'", name).cyan().bold()
        );
        vec![SkillKitAgent::ClaudeCode]
    };

    let result = bridge.install(name, &agents).await?;

    if format == "json" {
        println!("{}", serde_json::to_string_pretty(&result)?);
        return Ok(());
    }

    print_install_result(&result);
    Ok(())
}

async fn cmd_skills_uninstall(bridge: &SkillKitBridge, name: &str) -> anyhow::Result<()> {
    println!(
        "{} {}",
        "→".blue(),
        format!("Uninstalling '{}'...", name).cyan().bold()
    );

    bridge.uninstall(name).await?;

    println!();
    println!(
        "{} {}",
        "✓".green().bold(),
        format!("Skill '{}' has been uninstalled", name).green()
    );

    Ok(())
}

async fn cmd_skills_translate(
    bridge: &SkillKitBridge,
    name: &str,
    from: &str,
    to: &str,
    format: &str,
) -> anyhow::Result<()> {
    let from_agent = SkillKitAgent::parse(from).ok_or_else(|| {
        anyhow::anyhow!(
            "Unknown source agent: {}. Use 'rimuru skills agents' to see available agents.",
            from
        )
    })?;
    let to_agent = SkillKitAgent::parse(to).ok_or_else(|| {
        anyhow::anyhow!(
            "Unknown target agent: {}. Use 'rimuru skills agents' to see available agents.",
            to
        )
    })?;

    println!(
        "{} {} from {} to {}...",
        "→".blue(),
        format!("Translating '{}'", name).cyan().bold(),
        from_agent.display_name().yellow(),
        to_agent.display_name().green()
    );
    println!();

    let result = bridge.translate(name, from_agent, to_agent).await?;

    if format == "json" {
        println!("{}", serde_json::to_string_pretty(&result)?);
        return Ok(());
    }

    print_translation_result(&result);
    Ok(())
}

async fn cmd_skills_list(
    bridge: &SkillKitBridge,
    agent: Option<&str>,
    format: &str,
    enabled: bool,
    disabled: bool,
) -> anyhow::Result<()> {
    let skills: Vec<InstalledSkill> = if let Some(agent_str) = agent {
        let parsed_agent = SkillKitAgent::parse(agent_str).ok_or_else(|| {
            anyhow::anyhow!(
                "Unknown agent type: {}. Use 'rimuru skills agents' to see available agents.",
                agent_str
            )
        })?;
        println!(
            "{} for {}",
            "Installed Skills".cyan().bold(),
            parsed_agent.display_name().yellow()
        );
        bridge.list_for_agent(parsed_agent).await?
    } else {
        println!("{}", "Installed Skills".cyan().bold());
        bridge.list_installed().await?
    };

    let filtered: Vec<&InstalledSkill> = skills
        .iter()
        .filter(|s| {
            if enabled && !s.enabled {
                return false;
            }
            if disabled && s.enabled {
                return false;
            }
            true
        })
        .collect();

    if format == "json" {
        println!("{}", serde_json::to_string_pretty(&filtered)?);
        return Ok(());
    }

    println!();

    if filtered.is_empty() {
        println!("{}", "No skills installed.".yellow());
        println!();
        println!("Install a skill with:");
        println!("  {}", "rimuru skills install <name>".cyan());
        println!();
        println!("Or search the marketplace:");
        println!("  {}", "rimuru skills search <query>".cyan());
        return Ok(());
    }

    print_installed_skills_table(&filtered);

    println!();
    println!("  Total: {} skills", filtered.len());

    Ok(())
}

async fn cmd_skills_recommend(
    bridge: &SkillKitBridge,
    workflow: Option<&str>,
    format: &str,
    limit: usize,
) -> anyhow::Result<()> {
    println!("{}", "Skill Recommendations".cyan().bold());
    println!();

    let recommendations: Vec<SkillRecommendation> = if let Some(workflow_desc) = workflow {
        println!(
            "  {} Based on workflow: {}",
            "→".blue(),
            workflow_desc.yellow()
        );
        println!();
        bridge.recommend_for_workflow(workflow_desc).await?
    } else {
        println!(
            "  {} Analyzing your setup for recommendations...",
            "→".blue()
        );
        println!();
        bridge.recommend().await?
    };

    let limited: Vec<&SkillRecommendation> = recommendations.iter().take(limit).collect();

    if format == "json" {
        println!("{}", serde_json::to_string_pretty(&limited)?);
        return Ok(());
    }

    if limited.is_empty() {
        println!("{}", "No recommendations available.".yellow());
        println!();
        println!("Try specifying a workflow with:");
        println!(
            "  {}",
            "rimuru skills recommend --workflow \"building a React app\"".cyan()
        );
        return Ok(());
    }

    print_recommendations_table(&limited);

    println!();
    println!(
        "{}",
        "Use 'rimuru skills install <name>' to install a recommended skill.".dimmed()
    );

    Ok(())
}

async fn cmd_skills_publish(
    bridge: &SkillKitBridge,
    path: &str,
    format: &str,
    skip_validation: bool,
) -> anyhow::Result<()> {
    println!("{}", "Publishing Skill to Marketplace".cyan().bold());
    println!();
    println!("  {} Path: {}", "→".blue(), path);

    if !skip_validation {
        println!("  {} Validating skill...", "→".blue());
        let valid = bridge.validate_skill(path).await?;
        if !valid {
            println!();
            println!(
                "{} {}",
                "✗".red().bold(),
                "Skill validation failed. Fix errors and try again.".red()
            );
            println!();
            println!(
                "Use {} to skip validation (not recommended).",
                "--skip-validation".yellow()
            );
            return Ok(());
        }
        println!("  {} Validation passed", "✓".green());
    }

    println!("  {} Publishing...", "→".blue());

    let result = bridge.publish(path).await?;

    if format == "json" {
        println!("{}", serde_json::to_string_pretty(&result)?);
        return Ok(());
    }

    print_publish_result(&result);
    Ok(())
}

async fn cmd_skills_show(bridge: &SkillKitBridge, name: &str, format: &str) -> anyhow::Result<()> {
    let skill = bridge.get_skill_details(name).await?;

    if format == "json" {
        println!("{}", serde_json::to_string_pretty(&skill)?);
        return Ok(());
    }

    print_skill_details(&skill);
    Ok(())
}

async fn cmd_skills_sync(bridge: &SkillKitBridge) -> anyhow::Result<()> {
    println!("{}", "Syncing with SkillKit Marketplace".cyan().bold());
    println!();

    println!("  {} Fetching latest skill data...", "→".blue());

    bridge.sync().await?;

    println!();
    println!(
        "{} {}",
        "✓".green().bold(),
        "Skills synchronized successfully!".green()
    );

    Ok(())
}

async fn cmd_skills_stats(bridge: &SkillKitBridge, format: &str) -> anyhow::Result<()> {
    let stats = bridge.get_marketplace_stats().await?;

    if format == "json" {
        println!("{}", serde_json::to_string_pretty(&stats)?);
        return Ok(());
    }

    println!("{}", "SkillKit Marketplace Statistics".cyan().bold());
    println!("{}", "═".repeat(50).dimmed());
    println!();

    println!("  {}", "Overview".yellow().bold());
    println!("    Total Skills:    {:>8}", stats.total_skills);
    println!(
        "    Total Downloads: {:>8}",
        format_number(stats.total_downloads)
    );
    println!("    Total Authors:   {:>8}", stats.total_authors);
    println!();

    if !stats.skills_by_agent.is_empty() {
        println!("  {}", "Skills by Agent".yellow().bold());
        let mut agents: Vec<_> = stats.skills_by_agent.iter().collect();
        agents.sort_by(|a, b| b.1.cmp(a.1));
        for (agent, count) in agents.iter().take(10) {
            println!(
                "    {} {:>6}",
                format!("{:<20}", agent.display_name()),
                count
            );
        }
        println!();
    }

    if !stats.trending_skills.is_empty() {
        println!("  {}", "Trending Skills".yellow().bold());
        for skill in stats.trending_skills.iter().take(5) {
            println!("    {} {}", "•".cyan(), skill.name);
        }
        println!();
    }

    println!(
        "  {} {}",
        "Last Updated:".dimmed(),
        stats.last_updated.format("%Y-%m-%d %H:%M:%S UTC")
    );

    Ok(())
}

async fn cmd_skills_agents(format: &str) -> anyhow::Result<()> {
    let agents = SkillKitAgent::all();

    if format == "json" {
        let output: Vec<serde_json::Value> = agents
            .iter()
            .map(|a| {
                serde_json::json!({
                    "id": a.as_str(),
                    "name": a.display_name()
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(());
    }

    println!("{}", "Supported Agents".cyan().bold());
    println!("{}", "═".repeat(50).dimmed());
    println!();

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec![
            Cell::new("ID").fg(Color::White),
            Cell::new("Name").fg(Color::White),
            Cell::new("Icon").fg(Color::White),
        ]);

    for agent in agents {
        table.add_row(vec![
            Cell::new(agent.as_str()),
            Cell::new(agent.display_name()),
            Cell::new(get_agent_icon(agent)),
        ]);
    }

    println!("{table}");
    println!();
    println!("  Total: {} agents supported", agents.len());

    Ok(())
}

fn print_skills_table(skills: &[&Skill]) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec![
            Cell::new("Name").fg(Color::White),
            Cell::new("Description").fg(Color::White),
            Cell::new("Author").fg(Color::White),
            Cell::new("Tags").fg(Color::White),
            Cell::new("Downloads").fg(Color::White),
        ]);

    for skill in skills {
        let description = truncate_string(&skill.description, 40);
        let author = skill.author.as_deref().unwrap_or("-");
        let tags = if skill.tags.is_empty() {
            "-".to_string()
        } else {
            skill
                .tags
                .iter()
                .take(3)
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        };
        let downloads = skill
            .downloads
            .map(format_number)
            .unwrap_or_else(|| "-".to_string());

        table.add_row(vec![
            Cell::new(&skill.name),
            Cell::new(description),
            Cell::new(author),
            Cell::new(tags),
            Cell::new(downloads),
        ]);
    }

    println!("{table}");
}

fn print_installed_skills_table(skills: &[&InstalledSkill]) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec![
            Cell::new("Name").fg(Color::White),
            Cell::new("Description").fg(Color::White),
            Cell::new("Agents").fg(Color::White),
            Cell::new("Status").fg(Color::White),
            Cell::new("Installed").fg(Color::White),
        ]);

    for installed in skills {
        let description = truncate_string(&installed.skill.description, 35);
        let agents = installed
            .installed_for
            .iter()
            .map(|a| get_agent_icon(a))
            .collect::<Vec<_>>()
            .join(" ");
        let status_cell = if installed.enabled {
            Cell::new("✓ Enabled").fg(Color::Green)
        } else {
            Cell::new("✗ Disabled").fg(Color::Yellow)
        };
        let installed_at = installed.installed_at.format("%Y-%m-%d").to_string();

        table.add_row(vec![
            Cell::new(&installed.skill.name),
            Cell::new(description),
            Cell::new(agents),
            status_cell,
            Cell::new(installed_at),
        ]);
    }

    println!("{table}");
}

fn print_recommendations_table(recommendations: &[&SkillRecommendation]) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec![
            Cell::new("#").fg(Color::White),
            Cell::new("Skill").fg(Color::White),
            Cell::new("Reason").fg(Color::White),
            Cell::new("Confidence").fg(Color::White),
        ]);

    for (idx, rec) in recommendations.iter().enumerate() {
        let confidence_cell = get_confidence_cell(rec.confidence);
        let reason = truncate_string(&rec.reason, 45);

        table.add_row(vec![
            Cell::new((idx + 1).to_string()),
            Cell::new(&rec.skill.name),
            Cell::new(reason),
            confidence_cell,
        ]);
    }

    println!("{table}");
}

fn print_skill_details(skill: &Skill) {
    println!("{}", "Skill Details".cyan().bold());
    println!("{}", "═".repeat(50).dimmed());
    println!();

    println!("  {:<15} {}", "Name:".bold(), skill.name);
    println!("  {:<15} {}", "Slug:".bold(), skill.slug);
    println!();

    println!("  {}", "Description".yellow().bold());
    for line in skill.description.lines() {
        println!("    {}", line);
    }
    println!();

    if let Some(ref author) = skill.author {
        println!("  {:<15} {}", "Author:".bold(), author);
    }
    if let Some(ref source) = skill.source {
        println!("  {:<15} {}", "Source:".bold(), source);
    }
    if let Some(ref repo) = skill.repository {
        println!("  {:<15} {}", "Repository:".bold(), repo);
    }
    if let Some(ref version) = skill.version {
        println!("  {:<15} {}", "Version:".bold(), version);
    }
    if let Some(downloads) = skill.downloads {
        println!("  {:<15} {}", "Downloads:".bold(), format_number(downloads));
    }
    println!();

    if !skill.tags.is_empty() {
        println!("  {}", "Tags".yellow().bold());
        println!("    {}", skill.tags.join(", "));
        println!();
    }

    if !skill.agents.is_empty() {
        println!("  {}", "Compatible Agents".yellow().bold());
        for agent in &skill.agents {
            println!("    {} {}", get_agent_icon(agent), agent.display_name());
        }
        println!();
    } else {
        println!("  {}", "Compatible Agents".yellow().bold());
        println!("    {} Universal (all agents)", "★".yellow());
        println!();
    }

    if let Some(ref created) = skill.created_at {
        println!(
            "  {} {}",
            "Created:".dimmed(),
            created.format("%Y-%m-%d %H:%M UTC")
        );
    }
    if let Some(ref updated) = skill.updated_at {
        println!(
            "  {} {}",
            "Updated:".dimmed(),
            updated.format("%Y-%m-%d %H:%M UTC")
        );
    }
}

fn print_install_result(result: &InstalledSkill) {
    println!();
    println!(
        "{} {}",
        "✓".green().bold(),
        format!("Skill '{}' installed successfully!", result.skill.name).green()
    );
    println!();

    println!("  {:<15} {}", "Path:".bold(), result.path);
    println!(
        "  {:<15} {}",
        "Installed for:".bold(),
        result
            .installed_for
            .iter()
            .map(|a| a.display_name())
            .collect::<Vec<_>>()
            .join(", ")
    );
    println!(
        "  {:<15} {}",
        "Status:".bold(),
        if result.enabled {
            "Enabled".green()
        } else {
            "Disabled".yellow()
        }
    );
}

fn print_translation_result(result: &TranslationResult) {
    if result.success {
        println!(
            "{} {}",
            "✓".green().bold(),
            format!("Skill '{}' translated successfully!", result.skill_name).green()
        );
        println!();

        println!(
            "  {:<15} {} → {}",
            "Translation:".bold(),
            result.from_agent.display_name(),
            result.to_agent.display_name()
        );

        if let Some(ref path) = result.output_path {
            println!("  {:<15} {}", "Output:".bold(), path);
        }

        if result.duration_ms > 0 {
            println!("  {:<15} {}ms", "Duration:".bold(), result.duration_ms);
        }

        if !result.warnings.is_empty() {
            println!();
            println!("  {}", "Warnings:".yellow().bold());
            for warning in &result.warnings {
                println!("    {} {}", "⚠".yellow(), warning);
            }
        }
    } else {
        println!(
            "{} {}",
            "✗".red().bold(),
            format!("Translation failed for '{}'", result.skill_name).red()
        );
        println!();

        for error in &result.errors {
            println!("  {} {}", "✗".red(), error);
        }
    }
}

fn print_publish_result(result: &PublishResult) {
    println!();

    if result.success {
        println!(
            "{} {}",
            "✓".green().bold(),
            format!(
                "Skill '{}' v{} published successfully!",
                result.skill_name, result.version
            )
            .green()
        );
        println!();

        if let Some(ref url) = result.marketplace_url {
            println!("  {} {}", "Marketplace URL:".bold(), url.cyan());
        }
    } else {
        println!(
            "{} {}",
            "✗".red().bold(),
            format!("Failed to publish '{}'", result.skill_name).red()
        );
        println!();

        for error in &result.errors {
            println!("  {} {}", "✗".red(), error);
        }
    }
}

fn get_agent_icon(agent: &SkillKitAgent) -> &'static str {
    match agent {
        SkillKitAgent::ClaudeCode => "⟁",
        SkillKitAgent::Cursor => "◫",
        SkillKitAgent::Codex => "◎",
        SkillKitAgent::GeminiCli => "✦",
        SkillKitAgent::OpenCode => "◇",
        SkillKitAgent::GithubCopilot => "◈",
        SkillKitAgent::Goose => "⬡",
        SkillKitAgent::Windsurf => "⌘",
        _ => "●",
    }
}

fn get_confidence_cell(confidence: f32) -> Cell {
    let percent = (confidence * 100.0) as u32;
    let label = format!("{}%", percent);

    if confidence >= 0.8 {
        Cell::new(label).fg(Color::Green)
    } else if confidence >= 0.6 {
        Cell::new(label).fg(Color::Yellow)
    } else {
        Cell::new(label).fg(Color::Red)
    }
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

fn format_number(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}
