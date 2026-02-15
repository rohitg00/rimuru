#![allow(
    dead_code,
    unused_imports,
    unused_variables,
    unused_mut,
    clippy::too_many_arguments,
    clippy::needless_borrows_for_generic_args,
    clippy::manual_clamp,
    clippy::redundant_closure,
    clippy::new_without_default,
    clippy::manual_saturating_arithmetic,
    deprecated
)]

pub mod commands;
pub mod events;
pub mod groupchat;
pub mod playbooks;
pub mod pty;
pub mod remote;
pub mod state;
pub mod tray;
pub mod window_state;

use std::path::PathBuf;

use rimuru_core::{
    adapters::{
        AgentAdapter, ClaudeCodeAdapter, ClaudeCodeConfig, CodexAdapter, CodexConfig,
        CopilotAdapter, CopilotConfig, CursorAdapter, CursorConfig, GooseAdapter, GooseConfig,
        OpenCodeAdapter, OpenCodeConfig,
    },
    Agent, AgentRepository, AgentScanner, AgentType, ClaudeCodeCostCalculator, CostRecord,
    CostRepository, Repository, SessionRepository,
};
use state::AppState;
use tauri::{AppHandle, Manager};
use tracing_subscriber::EnvFilter;

async fn discover_and_persist_agents(
    state: &AppState,
    app: &AppHandle,
) -> Result<(), Box<dyn std::error::Error>> {
    let scanner = AgentScanner::new();
    let scan_result = scanner.scan_all().await?;
    let agent_repo = AgentRepository::new(state.db.pool().clone());

    for discovered in scan_result.discovered {
        let name = discovered.installation.name.clone();
        if agent_repo.get_by_name(&name).await?.is_some() {
            tracing::debug!("Agent '{}' already registered, skipping", name);
            continue;
        }

        let config_dir = discovered
            .installation
            .config_dir
            .to_string_lossy()
            .to_string();
        let config = serde_json::json!({
            "config_dir": config_dir,
            "is_configured": discovered.installation.is_configured,
            "executable_path": discovered.installation.executable_path.map(|p| p.to_string_lossy().to_string()),
        });

        let agent = Agent::new(name.clone(), discovered.installation.agent_type, config);
        match agent_repo.create(&agent).await {
            Ok(_) => tracing::info!("Auto-registered agent: {}", name),
            Err(e) => tracing::warn!("Failed to register agent '{}': {}", name, e),
        }
    }

    if let Err(e) = sync_sessions_from_agents(state, app).await {
        tracing::error!("Failed to sync sessions from agents: {}", e);
    }

    Ok(())
}

fn extract_session_tokens(metadata: &serde_json::Value) -> (i64, i64, String) {
    let input = metadata
        .get("input_tokens")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let output = metadata
        .get("output_tokens")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let model = metadata
        .get("model")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    (input, output, model)
}

async fn sync_sessions_from_agents(
    state: &AppState,
    app: &AppHandle,
) -> Result<(), Box<dyn std::error::Error>> {
    let agent_repo = AgentRepository::new(state.db.pool().clone());
    let session_repo = SessionRepository::new(state.db.pool().clone());
    let cost_repo = CostRepository::new(state.db.pool().clone());
    let calculator = ClaudeCodeCostCalculator::new();
    let agents = agent_repo.get_all().await?;

    for agent in &agents {
        let config_dir_str = agent
            .config
            .get("config_dir")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        let config_dir = PathBuf::from(config_dir_str);

        let existing_sessions = session_repo.get_by_agent(agent.id).await?;
        if !existing_sessions.is_empty() {
            tracing::debug!(
                "Agent '{}' already has {} sessions, skipping sync",
                agent.name,
                existing_sessions.len()
            );
            continue;
        }

        let sessions_result: Result<Vec<rimuru_core::Session>, _> = match agent.agent_type {
            AgentType::ClaudeCode => {
                let config = ClaudeCodeConfig::default()
                    .with_config_dir(config_dir.clone())
                    .with_projects_dir(config_dir.join("projects"));
                let adapter = ClaudeCodeAdapter::new(&agent.name, config);
                adapter.get_sessions().await
            }
            AgentType::Codex => {
                let config = CodexConfig::default()
                    .with_config_dir(config_dir.clone())
                    .with_data_dir(config_dir.clone());
                let adapter = CodexAdapter::new(&agent.name, config);
                adapter.get_sessions().await
            }
            AgentType::Goose => {
                let config = GooseConfig::default()
                    .with_config_dir(config_dir.clone())
                    .with_data_dir(config_dir.clone());
                let adapter = GooseAdapter::new(&agent.name, config);
                adapter.get_sessions().await
            }
            AgentType::Cursor => {
                let adapter = CursorAdapter::new(&agent.name, CursorConfig::default());
                adapter.get_sessions().await
            }
            AgentType::Copilot => {
                let adapter = CopilotAdapter::new(&agent.name, CopilotConfig::default());
                adapter.get_sessions().await
            }
            AgentType::OpenCode => {
                let config = OpenCodeConfig::default().with_config_dir(config_dir.clone());
                let adapter = OpenCodeAdapter::new(&agent.name, config);
                adapter.get_sessions().await
            }
        };

        match sessions_result {
            Ok(sessions) => {
                let count = sessions.len();
                for mut session in sessions {
                    session.agent_id = agent.id;
                    let is_completed = session.ended_at.is_some();
                    if let Err(e) = session_repo.create(&session).await {
                        tracing::warn!(
                            "Failed to persist session for agent '{}': {}",
                            agent.name,
                            e
                        );
                        continue;
                    }

                    if is_completed {
                        events::emit_session_ended(
                            app,
                            &session.id.to_string(),
                            &agent.id.to_string(),
                        );
                    } else {
                        events::emit_session_started(
                            app,
                            &session.id.to_string(),
                            &agent.id.to_string(),
                        );
                    }

                    let (input_tokens, output_tokens, model) =
                        extract_session_tokens(&session.metadata);

                    if input_tokens > 0 || output_tokens > 0 {
                        let cost_usd = calculator
                            .calculate_cost(input_tokens, output_tokens, &model)
                            .unwrap_or(0.0);
                        let cost_record = CostRecord::new(
                            session.id,
                            agent.id,
                            model.clone(),
                            input_tokens,
                            output_tokens,
                            cost_usd,
                        );
                        if let Err(e) = cost_repo.record_cost(&cost_record).await {
                            tracing::warn!(
                                "Failed to record cost for session '{}': {}",
                                session.id,
                                e
                            );
                        } else {
                            events::emit_cost_recorded(
                                app,
                                &session.id.to_string(),
                                &model,
                                cost_usd,
                                input_tokens + output_tokens,
                            );
                        }
                    }
                }
                if count > 0 {
                    tracing::info!("Synced {} sessions for agent '{}'", count, agent.name);
                }
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to get sessions for agent '{}' ({}): {}",
                    agent.name,
                    agent.agent_type,
                    e
                );
            }
        }
    }

    let cleaned = sqlx::query(
        "UPDATE sessions SET status = 'completed' WHERE status = 'active' AND ended_at IS NOT NULL",
    )
    .execute(state.db.pool())
    .await
    .map(|r| r.rows_affected())
    .unwrap_or(0);
    if cleaned > 0 {
        tracing::info!("Marked {} sessions with ended_at as completed", cleaned);
    }

    let stale = sqlx::query(
        "UPDATE sessions SET status = 'completed', ended_at = COALESCE(ended_at, started_at) WHERE status = 'active' AND ended_at IS NULL"
    )
    .execute(state.db.pool())
    .await
    .map(|r| r.rows_affected())
    .unwrap_or(0);
    if stale > 0 {
        tracing::info!("Marked {} sessions without ended_at as completed", stale);
    }

    backfill_cost_records(state, app).await;

    Ok(())
}

async fn backfill_cost_records(state: &AppState, app: &AppHandle) {
    let session_repo = SessionRepository::new(state.db.pool().clone());
    let cost_repo = CostRepository::new(state.db.pool().clone());
    let calculator = ClaudeCodeCostCalculator::new();

    let existing_costs = cost_repo.get_all().await.unwrap_or_default();
    if !existing_costs.is_empty() {
        let all_same_day = existing_costs
            .windows(2)
            .all(|w| w[0].recorded_at.date_naive() == w[1].recorded_at.date_naive());
        if all_same_day && existing_costs.len() > 10 {
            tracing::info!(
                "Clearing {} cost records with same date for re-backfill",
                existing_costs.len()
            );
            for cost in &existing_costs {
                let _ = cost_repo.delete(cost.id).await;
            }
        } else {
            tracing::debug!(
                "Cost records already exist ({}), skipping backfill",
                existing_costs.len()
            );
            return;
        }
    }

    let all_sessions = session_repo.get_all().await.unwrap_or_default();
    let mut created = 0u64;
    for session in &all_sessions {
        let (input_tokens, output_tokens, model) = extract_session_tokens(&session.metadata);

        if input_tokens == 0 && output_tokens == 0 {
            continue;
        }

        let cost_usd = calculator
            .calculate_cost(input_tokens, output_tokens, &model)
            .unwrap_or(0.0);
        let mut cost_record = CostRecord::new(
            session.id,
            session.agent_id,
            model.clone(),
            input_tokens,
            output_tokens,
            cost_usd,
        );
        cost_record.recorded_at = session.started_at;
        if cost_repo.record_cost(&cost_record).await.is_ok() {
            created += 1;
            events::emit_cost_recorded(
                app,
                &session.id.to_string(),
                &model,
                cost_usd,
                input_tokens + output_tokens,
            );
        }
    }
    if created > 0 {
        tracing::info!("Backfilled {} cost records from existing sessions", created);
    }
}

pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            let app_handle = app.handle().clone();

            if let Err(e) = tray::setup_tray(&app_handle) {
                tracing::error!("Failed to setup system tray: {}", e);
            }

            if let Some(window) = app.get_webview_window("main") {
                window_state::restore_window_state(&window);
                window_state::setup_window_state_persistence(&window);
            }

            tauri::async_runtime::spawn(async move {
                match AppState::new().await {
                    Ok(state) => {
                        state.pty_manager.set_app_handle(app_handle.clone());
                        app_handle.manage(state.pty_manager.clone());
                        app_handle.manage(state.clone());

                        if let Err(e) = discover_and_persist_agents(&state, &app_handle).await {
                            tracing::error!("Failed to auto-discover agents: {}", e);
                        }

                        if let Err(e) = events::start_event_emitter(app_handle.clone(), state).await
                        {
                            tracing::error!("Failed to start event emitter: {}", e);
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to initialize app state: {}", e);
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::agents::get_agents,
            commands::agents::get_agent_details,
            commands::agents::scan_agents,
            commands::agents::add_agent,
            commands::sessions::get_sessions,
            commands::sessions::get_session_details,
            commands::sessions::get_active_sessions,
            commands::costs::get_cost_summary,
            commands::costs::get_cost_breakdown,
            commands::costs::get_cost_history,
            commands::metrics::get_system_metrics,
            commands::metrics::get_metrics_history,
            commands::sync::trigger_sync,
            commands::sync::get_sync_status,
            commands::skills::search_skills,
            commands::skills::get_installed_skills,
            commands::skills::get_skill_details,
            commands::skills::install_skill,
            commands::skills::uninstall_skill,
            commands::skills::translate_skill,
            commands::skills::get_skill_recommendations,
            commands::skills::publish_skill,
            commands::skills::enable_skill,
            commands::skills::disable_skill,
            commands::skills::sync_skills,
            commands::skills::get_skillkit_status,
            commands::plugins::get_plugins,
            commands::plugins::get_plugin_details,
            commands::plugins::install_plugin,
            commands::plugins::enable_plugin,
            commands::plugins::disable_plugin,
            commands::plugins::uninstall_plugin,
            commands::plugins::get_plugin_config,
            commands::plugins::configure_plugin,
            commands::plugins::get_plugin_events,
            commands::hooks::get_hooks,
            commands::hooks::get_hook_handlers,
            commands::hooks::get_hook_executions,
            commands::hooks::trigger_hook,
            commands::hooks::enable_hook_handler,
            commands::hooks::disable_hook_handler,
            commands::hooks::get_hook_stats,
            commands::settings::get_settings,
            commands::settings::save_settings,
            commands::settings::get_db_stats,
            commands::export::export_sessions,
            commands::export::export_costs,
            commands::pty_commands::launch_session,
            commands::pty_commands::write_to_session,
            commands::pty_commands::resize_session,
            commands::pty_commands::terminate_session,
            commands::pty_commands::list_live_sessions,
            commands::pty_commands::get_live_session,
            commands::pty_commands::create_git_worktree,
            commands::pty_commands::cleanup_git_worktree,
            commands::pty_commands::list_git_worktrees,
            commands::pty_commands::discover_agent_sessions,
            commands::pty_commands::list_playbooks,
            commands::pty_commands::load_playbook,
            commands::groupchat::create_chat_room,
            commands::groupchat::send_chat_message,
            commands::groupchat::get_chat_messages,
            commands::groupchat::list_chat_rooms,
            commands::groupchat::close_chat_room,
            commands::remote_commands::start_remote_server,
            commands::remote_commands::stop_remote_server,
            commands::remote_commands::get_remote_status,
            commands::filesystem::read_directory,
            commands::filesystem::get_directory_stats,
            commands::filesystem::get_git_info,
            commands::filesystem::read_file_preview,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
