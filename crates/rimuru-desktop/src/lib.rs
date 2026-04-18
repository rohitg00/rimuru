mod commands;
mod events;
mod notifications;
mod state;
mod tray;
mod window_state;

use iii_sdk::RegisterFunctionMessage;
use notifications::{
    BudgetThresholdCtx, NotificationDispatcher, NotificationKind, NotificationPreferences,
    OptimizationCtx, RunawayCtx, SessionCostCtx,
};
use rimuru_core::{DEFAULT_ENGINE_URL, RimuruWorker, StateKV};
use serde_json::{Value, json};
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
        .plugin(tauri_plugin_notification::init())
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

            let app_state = AppState::new(iii.clone(), kv.clone(), api_port);
            app.manage(app_state);

            {
                use tauri::plugin::PermissionState;
                use tauri_plugin_notification::NotificationExt;
                let notifier = app.notification();
                match notifier.permission_state() {
                    Ok(PermissionState::Prompt) | Ok(PermissionState::PromptWithRationale) => {
                        if let Err(e) = notifier.request_permission() {
                            tracing::warn!("Failed to request notification permission: {e}");
                        }
                    }
                    Ok(PermissionState::Denied) => {
                        tracing::warn!(
                            "Notification permission denied by OS; notifications will be suppressed"
                        );
                    }
                    Ok(PermissionState::Granted) => {}
                    Err(e) => {
                        tracing::warn!("Failed to read notification permission: {e}");
                    }
                }
            }

            let dispatcher = NotificationDispatcher::new(app.handle().clone());
            register_notification_dispatcher(&iii, &kv, dispatcher);

            let handle = app.handle().clone();
            let iii_for_hooks = iii.clone();

            tauri::async_runtime::spawn(async move {
                if let Err(e) = worker.start().await {
                    tracing::error!("Failed to start worker: {}", e);
                    return;
                }
                info!("Worker started (API served by iii-http on engine port)");

                register_notification_hooks(&iii_for_hooks).await;

                let _ = handle.emit("worker-ready", serde_json::json!({"port": api_port}));
            });

            tray::setup_tray(app.handle())?;
            tray::spawn_tooltip_updater(app.handle().clone(), iii.clone());
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

const DESKTOP_NOTIFY_FN: &str = "rimuru.desktop.notify";

const NOTIFICATION_EVENT_TYPES: &[&str] = &[
    "budget.warning",
    "budget.exceeded",
    "session.cost_milestone",
    "runaway.detected",
    "optimize.opportunity",
];

fn register_notification_dispatcher<R: tauri::Runtime>(
    iii: &iii_sdk::III,
    kv: &StateKV,
    dispatcher: NotificationDispatcher<R>,
) {
    let kv = kv.clone();
    iii.register_function_with(
        RegisterFunctionMessage::with_id(DESKTOP_NOTIFY_FN.to_string()),
        move |input: Value| {
            let kv = kv.clone();
            let dispatcher = dispatcher.clone();
            async move {
                let input = extract_input(&input);
                let event_type = input
                    .get("event_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let payload = input.get("payload").cloned().unwrap_or(json!({}));

                let prefs = NotificationPreferences::load(&kv).await;

                let kind = map_event_to_kind(&event_type, &payload, &prefs);

                let mut dispatched = false;
                let mut suppressed = false;

                if let Some(kind) = kind {
                    let enabled = match &kind {
                        NotificationKind::BudgetThreshold { .. } => prefs.budget_enabled,
                        NotificationKind::SessionCostMilestone(_) => prefs.session_cost_enabled,
                        NotificationKind::RunawayDetected(_) => prefs.runaway_enabled,
                        NotificationKind::OptimizationOpportunity(_) => prefs.optimization_enabled,
                    };
                    if enabled {
                        if let Err(e) = dispatcher.dispatch(&kind) {
                            tracing::warn!("notification dispatch failed: {}", e);
                        } else {
                            dispatched = true;
                        }
                    } else {
                        suppressed = true;
                    }
                }

                Ok(json!({
                    "event_type": event_type,
                    "dispatched": dispatched,
                    "suppressed": suppressed,
                }))
            }
        },
    );
}

fn extract_input(input: &Value) -> Value {
    input
        .get("body")
        .cloned()
        .or_else(|| input.get("payload").cloned())
        .unwrap_or_else(|| input.clone())
}

fn map_event_to_kind(
    event_type: &str,
    payload: &Value,
    _prefs: &NotificationPreferences,
) -> Option<NotificationKind> {
    match event_type {
        "budget.warning" | "budget.exceeded" => {
            let monthly_spent = payload
                .get("monthly_spent")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let daily_spent = payload
                .get("daily_spent")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);
            let current = if daily_spent > 0.0 {
                daily_spent
            } else {
                monthly_spent
            };
            let limit = payload.get("limit").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let percent = payload
                .get("percent")
                .and_then(|v| v.as_f64())
                .unwrap_or_else(|| {
                    if limit > 0.0 {
                        current / limit * 100.0
                    } else {
                        0.0
                    }
                });
            let level: u8 = if event_type == "budget.exceeded" || percent >= 100.0 {
                100
            } else if percent >= 80.0 {
                80
            } else {
                50
            };
            Some(NotificationKind::BudgetThreshold {
                level,
                ctx: BudgetThresholdCtx {
                    current,
                    limit,
                    percent,
                    agent: payload
                        .get("agent")
                        .and_then(|v| v.as_str())
                        .map(String::from),
                },
            })
        }
        "session.cost_milestone" => {
            let session_id = payload
                .get("session_id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let cost = payload.get("cost").and_then(|v| v.as_f64()).unwrap_or(0.0);
            Some(NotificationKind::SessionCostMilestone(SessionCostCtx {
                session_id,
                cost,
            }))
        }
        "runaway.detected" => {
            let session_id = payload
                .get("session_id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let agent = payload
                .get("agent")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            let tool_count = payload
                .get("tool_count")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32;
            Some(NotificationKind::RunawayDetected(RunawayCtx {
                agent,
                session_id,
                tool_count,
            }))
        }
        "optimize.opportunity" => {
            let recommendation = payload
                .get("recommendation")
                .and_then(|v| v.as_str())
                .unwrap_or("A new optimization opportunity was identified")
                .to_string();
            Some(NotificationKind::OptimizationOpportunity(OptimizationCtx {
                recommendation,
            }))
        }
        _ => None,
    }
}

async fn register_notification_hooks(iii: &iii_sdk::III) {
    for event_type in NOTIFICATION_EVENT_TYPES {
        let res = iii
            .trigger(iii_sdk::TriggerRequest {
                function_id: "rimuru.hooks.register".to_string(),
                payload: json!({
                    "event_type": *event_type,
                    "function_id": DESKTOP_NOTIFY_FN,
                    "priority": 100,
                }),
                action: None,
                timeout_ms: Some(5000),
            })
            .await;
        if let Err(e) = res {
            tracing::warn!(
                "failed to register desktop notification hook for {}: {}",
                event_type,
                e
            );
        }
    }
}
