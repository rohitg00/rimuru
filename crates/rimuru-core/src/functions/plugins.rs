use chrono::Utc;
use iii_sdk::III;
use serde_json::{json, Value};

use crate::models::{PluginLanguage, PluginManifest, PluginState, PluginStatus};
use crate::state::StateKV;

pub fn register(iii: &III, kv: &StateKV) {
    register_install(iii, kv);
    register_uninstall(iii, kv);
    register_list(iii, kv);
    register_lifecycle(iii, kv);
}

fn register_install(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function("rimuru.plugins.install", move |input: Value| {
        let kv = kv.clone();
        async move {
            let plugin_id = input
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| iii_sdk::IIIError::Handler("id is required".into()))?
                .to_string();

            let name = input
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or(&plugin_id)
                .to_string();

            let version = input
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("0.1.0")
                .to_string();

            let description = input
                .get("description")
                .and_then(|v| v.as_str())
                .map(String::from);

            let language: PluginLanguage = input
                .get("language")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or(PluginLanguage::TypeScript);

            let binary_path = input
                .get("binary_path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| iii_sdk::IIIError::Handler("binary_path is required".into()))?
                .to_string();

            let functions: Vec<String> = input
                .get("functions")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default();

            let existing: Option<PluginManifest> = kv
                .get("plugins", &plugin_id)
                .await
                .map_err(|e| iii_sdk::IIIError::Handler(e.to_string()))?;

            if existing.is_some() {
                return Err(iii_sdk::IIIError::Handler(format!(
                    "plugin already installed: {}",
                    plugin_id
                )));
            }

            let state = PluginState {
                plugin_id: plugin_id.clone(),
                status: PluginStatus::Installing,
                pid: None,
                started_at: None,
                last_error: None,
                restart_count: 0,
            };

            kv.set("plugin_state", &plugin_id, &state)
                .await
                .map_err(|e| iii_sdk::IIIError::Handler(e.to_string()))?;

            let path = std::path::Path::new(&binary_path);
            if !path.exists() {
                let failed_state = PluginState {
                    plugin_id: plugin_id.clone(),
                    status: PluginStatus::Error,
                    pid: None,
                    started_at: None,
                    last_error: Some(format!("binary not found: {}", binary_path)),
                    restart_count: 0,
                };
                kv.set("plugin_state", &plugin_id, &failed_state)
                    .await
                    .map_err(|e| iii_sdk::IIIError::Handler(e.to_string()))?;

                return Err(iii_sdk::IIIError::Handler(format!(
                    "binary not found: {}",
                    binary_path
                )));
            }

            let manifest = PluginManifest {
                id: plugin_id.clone(),
                name,
                version,
                description,
                language,
                binary_path,
                functions,
                hooks: vec![],
                enabled: true,
                installed_at: Utc::now(),
            };

            kv.set("plugins", &plugin_id, &manifest)
                .await
                .map_err(|e| iii_sdk::IIIError::Handler(e.to_string()))?;

            let ready_state = PluginState {
                plugin_id: plugin_id.clone(),
                status: PluginStatus::Stopped,
                pid: None,
                started_at: None,
                last_error: None,
                restart_count: 0,
            };
            kv.set("plugin_state", &plugin_id, &ready_state)
                .await
                .map_err(|e| iii_sdk::IIIError::Handler(e.to_string()))?;

            Ok(json!({
                "plugin": manifest,
                "state": ready_state,
                "installed": true
            }))
        }
    });
}

fn register_uninstall(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function("rimuru.plugins.uninstall", move |input: Value| {
        let kv = kv.clone();
        async move {
            let plugin_id = input
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| iii_sdk::IIIError::Handler("id is required".into()))?
                .to_string();

            let manifest: PluginManifest = kv
                .get("plugins", &plugin_id)
                .await
                .map_err(|e| iii_sdk::IIIError::Handler(e.to_string()))?
                .ok_or_else(|| {
                    iii_sdk::IIIError::Handler(format!("plugin not found: {}", plugin_id))
                })?;

            let state: Option<PluginState> = kv
                .get("plugin_state", &plugin_id)
                .await
                .map_err(|e| iii_sdk::IIIError::Handler(e.to_string()))?;

            if let Some(ref s) = state {
                if s.status == PluginStatus::Running {
                    if let Some(pid) = s.pid {
                        let _ = tokio::process::Command::new("kill")
                            .arg(pid.to_string())
                            .output()
                            .await;
                    }
                }
            }

            kv.delete("plugins", &plugin_id)
                .await
                .map_err(|e| iii_sdk::IIIError::Handler(e.to_string()))?;

            kv.delete("plugin_state", &plugin_id)
                .await
                .map_err(|e| iii_sdk::IIIError::Handler(e.to_string()))?;

            Ok(json!({
                "uninstalled": plugin_id,
                "name": manifest.name
            }))
        }
    });
}

fn register_list(iii: &III, _kv: &StateKV) {
    iii.register_function("rimuru.plugins.list", move |_input: Value| {
        async move {
            let result = crate::discovery::discover_plugins().await;
            Ok(json!({
                "plugins": result,
                "total": result.len()
            }))
        }
    });
}

fn register_lifecycle(iii: &III, kv: &StateKV) {
    let kv_start = kv.clone();
    iii.register_function("rimuru.plugins.start", move |input: Value| {
        let kv = kv_start.clone();
        async move {
            let plugin_id = input
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| iii_sdk::IIIError::Handler("id is required".into()))?
                .to_string();

            let manifest: PluginManifest = kv
                .get("plugins", &plugin_id)
                .await
                .map_err(|e| iii_sdk::IIIError::Handler(e.to_string()))?
                .ok_or_else(|| {
                    iii_sdk::IIIError::Handler(format!("plugin not found: {}", plugin_id))
                })?;

            if !manifest.enabled {
                return Err(iii_sdk::IIIError::Handler(format!(
                    "plugin is disabled: {}",
                    plugin_id
                )));
            }

            let child = tokio::process::Command::new(&manifest.binary_path)
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn();

            match child {
                Ok(child) => {
                    let pid = child.id().unwrap_or(0);

                    let state = PluginState {
                        plugin_id: plugin_id.clone(),
                        status: PluginStatus::Running,
                        pid: Some(pid),
                        started_at: Some(Utc::now()),
                        last_error: None,
                        restart_count: 0,
                    };

                    kv.set("plugin_state", &plugin_id, &state)
                        .await
                        .map_err(|e| iii_sdk::IIIError::Handler(e.to_string()))?;

                    Ok(json!({
                        "plugin_id": plugin_id,
                        "status": "running",
                        "pid": pid
                    }))
                }
                Err(e) => {
                    let state = PluginState {
                        plugin_id: plugin_id.clone(),
                        status: PluginStatus::Error,
                        pid: None,
                        started_at: None,
                        last_error: Some(e.to_string()),
                        restart_count: 0,
                    };

                    kv.set("plugin_state", &plugin_id, &state)
                        .await
                        .map_err(|e| iii_sdk::IIIError::Handler(e.to_string()))?;

                    Err(iii_sdk::IIIError::Handler(format!(
                        "failed to start plugin: {}",
                        e
                    )))
                }
            }
        }
    });

    let kv_stop = kv.clone();
    iii.register_function("rimuru.plugins.stop", move |input: Value| {
        let kv = kv_stop.clone();
        async move {
            let plugin_id = input
                .get("id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| iii_sdk::IIIError::Handler("id is required".into()))?
                .to_string();

            let current_state: PluginState = kv
                .get("plugin_state", &plugin_id)
                .await
                .map_err(|e| iii_sdk::IIIError::Handler(e.to_string()))?
                .ok_or_else(|| {
                    iii_sdk::IIIError::Handler(format!("plugin not found: {}", plugin_id))
                })?;

            if current_state.status != PluginStatus::Running {
                return Err(iii_sdk::IIIError::Handler(format!(
                    "plugin is not running: {}",
                    plugin_id
                )));
            }

            if let Some(pid) = current_state.pid {
                let kill_result = tokio::process::Command::new("kill")
                    .arg(pid.to_string())
                    .output()
                    .await;

                if let Err(e) = kill_result {
                    return Err(iii_sdk::IIIError::Handler(format!(
                        "failed to stop plugin process: {}",
                        e
                    )));
                }
            }

            let state = PluginState {
                plugin_id: plugin_id.clone(),
                status: PluginStatus::Stopped,
                pid: None,
                started_at: None,
                last_error: None,
                restart_count: current_state.restart_count,
            };

            kv.set("plugin_state", &plugin_id, &state)
                .await
                .map_err(|e| iii_sdk::IIIError::Handler(e.to_string()))?;

            Ok(json!({
                "plugin_id": plugin_id,
                "status": "stopped"
            }))
        }
    });
}
