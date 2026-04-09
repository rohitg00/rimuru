mod commands;
mod events;
mod state;
mod tray;
mod window_state;

use rimuru_core::{DEFAULT_ENGINE_URL, RimuruWorker, StateKV};
use state::AppState;
use tauri::{Emitter, Manager};
use tracing::info;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("rimuru=info".parse().unwrap()),
        )
        .init();

    let api_port: u16 = std::env::var("RIMURU_API_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3100);

    let engine_url =
        std::env::var("RIMURU_ENGINE_URL").unwrap_or_else(|_| DEFAULT_ENGINE_URL.to_string());

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state == tauri_plugin_global_shortcut::ShortcutState::Pressed
                        && let Some(window) = app.get_webview_window("main")
                    {
                        if window.is_visible().unwrap_or(false) {
                            let _ = window.hide();
                        } else {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(),
        )
        .setup(move |app| {
            use tauri_plugin_global_shortcut::GlobalShortcutExt;
            if let Err(e) = app.global_shortcut().register("CmdOrCtrl+Shift+R") {
                tracing::warn!("Failed to register global shortcut: {}", e);
            }

            let worker = RimuruWorker::new(&engine_url);
            let iii = worker.iii().clone();
            let kv = StateKV::new(iii.clone());

            let app_state = AppState::new(iii, kv.clone(), api_port);
            app.manage(app_state);

            let handle = app.handle().clone();

            tauri::async_runtime::spawn(async move {
                if let Err(e) = worker.start().await {
                    tracing::error!("Failed to start worker: {}", e);
                    return;
                }
                info!("Worker started (API served by iii-http on engine port)");

                let _ = handle.emit("worker-ready", serde_json::json!({"port": api_port}));
            });

            tray::setup_tray(app.handle())?;
            window_state::restore_window_state(app.handle());

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                window_state::save_window_state(window.app_handle());
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::agents::list_agents,
            commands::agents::get_agent,
            commands::agents::register_agent,
            commands::agents::unregister_agent,
            commands::agents::detect_agents,
            commands::agents::connect_agent,
            commands::agents::disconnect_agent,
            commands::agents::sync_agents,
            commands::costs::get_cost_summary,
            commands::costs::get_cost_breakdown,
            commands::costs::get_cost_history,
            commands::costs::record_cost,
            commands::costs::get_daily_rollup,
            commands::sessions::list_sessions,
            commands::sessions::get_session,
            commands::sessions::get_active_sessions,
            commands::sessions::get_session_history,
            commands::metrics::get_system_metrics,
            commands::metrics::get_metrics_history,
            commands::metrics::get_hardware_info,
            commands::metrics::detect_hardware,
            commands::metrics::get_model_advisor,
            commands::sync::trigger_sync,
            commands::sync::get_sync_status,
            commands::settings::get_settings,
            commands::settings::update_setting,
            commands::settings::get_health,
            commands::settings::get_version,
            commands::settings::get_port,
            commands::hooks::list_hooks,
            commands::hooks::register_hook,
            commands::hooks::dispatch_hook,
            commands::hooks::delete_hook,
            commands::plugins::list_plugins,
            commands::plugins::install_plugin,
            commands::plugins::uninstall_plugin,
            commands::plugins::start_plugin,
            commands::plugins::stop_plugin,
            commands::skills::search_skills,
            commands::skills::install_skill,
            commands::skills::translate_skill,
            commands::skills::recommend_skills,
            commands::export::export_costs,
            commands::export::export_sessions,
            commands::export::export_agents,
            commands::export::open_external,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
