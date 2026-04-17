use std::sync::Arc;
use std::time::Duration;

use iii_sdk::{III, TriggerRequest};
use serde_json::json;
use tauri::{
    AppHandle, Emitter, Manager, Runtime,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
};
use tokio::sync::Mutex;

pub const TRAY_ID: &str = "rimuru-main-tray";

struct TrayMenuItems<R: Runtime> {
    pause: MenuItem<R>,
}

pub fn setup_tray<R: Runtime>(app: &AppHandle<R>) -> Result<(), Box<dyn std::error::Error>> {
    let show = MenuItem::with_id(app, "show", "Show Rimuru", true, None::<&str>)?;
    let hide = MenuItem::with_id(app, "hide", "Hide to Tray", true, None::<&str>)?;

    let dashboard = MenuItem::with_id(app, "dashboard", "Open dashboard", true, None::<&str>)?;
    let active_sessions = MenuItem::with_id(
        app,
        "active_sessions",
        "Show active sessions",
        true,
        None::<&str>,
    )?;
    let agents = MenuItem::with_id(app, "agents", "Agents", true, None::<&str>)?;
    let sessions = MenuItem::with_id(app, "sessions", "Sessions", true, None::<&str>)?;
    let costs = MenuItem::with_id(app, "costs", "Costs", true, None::<&str>)?;

    let quick_budget = MenuItem::with_id(
        app,
        "quick_budget",
        "Quick budget override",
        true,
        None::<&str>,
    )?;
    let pause_label = pause_label_for(false);
    let pause = MenuItem::with_id(app, "toggle_tracking", pause_label, true, None::<&str>)?;

    let settings = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    let about = MenuItem::with_id(app, "about", "About Rimuru", true, None::<&str>)?;

    let quit = MenuItem::with_id(app, "quit", "Quit Rimuru", true, None::<&str>)?;

    let sep1 = PredefinedMenuItem::separator(app)?;
    let sep2 = PredefinedMenuItem::separator(app)?;
    let sep3 = PredefinedMenuItem::separator(app)?;
    let sep4 = PredefinedMenuItem::separator(app)?;

    let menu = Menu::with_items(
        app,
        &[
            &show,
            &hide,
            &sep1,
            &dashboard,
            &active_sessions,
            &agents,
            &sessions,
            &costs,
            &sep2,
            &quick_budget,
            &pause,
            &sep3,
            &settings,
            &about,
            &sep4,
            &quit,
        ],
    )?;

    app.manage(TrayMenuItems {
        pause: pause.clone(),
    });

    TrayIconBuilder::with_id(TRAY_ID)
        .menu(&menu)
        .tooltip("Rimuru")
        .on_menu_event(|app, event| {
            let id = event.id().as_ref();
            match id {
                "show" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "hide" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.hide();
                    }
                }
                "dashboard" => navigate(app, "/"),
                "active_sessions" => navigate(app, "/sessions"),
                "quick_budget" => navigate(app, "/settings?focus=budget"),
                "agents" | "sessions" | "costs" | "settings" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                    let _ = app.emit("navigate", serde_json::json!({ "page": id }));
                }
                "toggle_tracking" => toggle_tracking(app.clone()),
                "about" => {
                    let _ = app.emit("about", serde_json::json!({}));
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let tauri::tray::TrayIconEvent::Click {
                button: tauri::tray::MouseButton::Left,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                    } else {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
        })
        .build(app)?;

    let app_for_init = app.clone();
    tauri::async_runtime::spawn(async move {
        init_tracking_label(app_for_init).await;
    });

    Ok(())
}

fn navigate<R: Runtime>(app: &AppHandle<R>, route: &str) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
        let hash = if let Some(stripped) = route.strip_prefix('/') {
            format!("#/{}", stripped)
        } else {
            format!("#{}", route)
        };
        let script = format!("window.location.hash = '{}';", hash.replace('\'', "\\'"));
        let _ = window.eval(&script);
    }
    let _ = app.emit("navigate", json!({ "route": route }));
}

fn pause_label_for(paused: bool) -> &'static str {
    if paused {
        "Resume tracking"
    } else {
        "Pause tracking"
    }
}

fn toggle_tracking<R: Runtime>(app: AppHandle<R>) {
    tauri::async_runtime::spawn(async move {
        let kv = match app.try_state::<crate::state::AppState>() {
            Some(state) => state.kv.clone(),
            None => return,
        };
        let current: bool = kv
            .get::<bool>("desktop", "tracking_paused")
            .await
            .ok()
            .flatten()
            .unwrap_or(false);
        let next = !current;
        if let Err(e) = kv.set("desktop", "tracking_paused", &next).await {
            tracing::warn!("failed to persist tracking_paused: {}", e);
            return;
        }
        if let Some(items) = app.try_state::<TrayMenuItems<R>>()
            && let Err(e) = items.pause.set_text(pause_label_for(next))
        {
            tracing::warn!("failed to update pause menu label: {}", e);
        }
        let _ = app.emit("tracking-paused", json!({ "paused": next }));
    });
}

async fn init_tracking_label<R: Runtime>(app: AppHandle<R>) {
    let Some(state) = app.try_state::<crate::state::AppState>() else {
        return;
    };
    let paused = state
        .kv
        .get::<bool>("desktop", "tracking_paused")
        .await
        .ok()
        .flatten()
        .unwrap_or(false);
    if paused && let Some(items) = app.try_state::<TrayMenuItems<R>>() {
        let _ = items.pause.set_text(pause_label_for(paused));
    }
}

pub fn spawn_tooltip_updater<R: Runtime>(app: AppHandle<R>, iii: III) {
    let iii = Arc::new(iii);
    let last = Arc::new(Mutex::new(String::from("Rimuru")));
    tauri::async_runtime::spawn(async move {
        let mut ticker = tokio::time::interval(Duration::from_secs(30));
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        loop {
            ticker.tick().await;
            let tooltip = match fetch_tooltip(&iii).await {
                Ok(t) => t,
                Err(e) => {
                    tracing::debug!("tooltip fetch skipped: {}", e);
                    continue;
                }
            };
            let mut guard = last.lock().await;
            if *guard == tooltip {
                continue;
            }
            if let Some(tray) = app.tray_by_id(TRAY_ID) {
                if let Err(e) = tray.set_tooltip(Some(&tooltip)) {
                    tracing::debug!("set_tooltip failed: {}", e);
                } else {
                    *guard = tooltip;
                }
            }
        }
    });
}

async fn fetch_tooltip(iii: &III) -> Result<String, String> {
    let since = chrono::Utc::now()
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .map(|dt| {
            chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(dt, chrono::Utc).to_rfc3339()
        });

    let result = iii
        .trigger(TriggerRequest {
            function_id: "rimuru.costs.summary".to_string(),
            payload: json!({ "since": since }),
            action: None,
            timeout_ms: Some(5000),
        })
        .await
        .map_err(|e| e.to_string())?;

    let body = result.get("body").unwrap_or(&result);
    let summary = body.get("summary").unwrap_or(body);
    let total = summary
        .get("total_cost")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let sessions = summary
        .get("total_records")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let agents = summary
        .get("by_agent")
        .and_then(|v| v.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    Ok(format!(
        "Today: ${:.2} • {} sessions • {} agents",
        total, sessions, agents
    ))
}
