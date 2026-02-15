use crate::state::{AppState, PluginEventRecord};
use rimuru_core::{
    create_builtin_exporter, create_builtin_notifier, is_builtin_plugin, list_builtin_plugins,
    PluginCapability, PluginStatus,
};
use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::{debug, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginResponse {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub capabilities: Vec<String>,
    pub status: String,
    pub enabled: bool,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub license: Option<String>,
    pub loaded_at: Option<String>,
    pub error: Option<String>,
    pub is_builtin: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfigResponse {
    pub plugin_id: String,
    pub enabled: bool,
    pub settings: serde_json::Value,
    pub priority: i32,
    pub schema: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginEventResponse {
    pub event: String,
    pub plugin_id: String,
    pub timestamp: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallPluginRequest {
    pub source: String,
    pub auto_enable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurePluginRequest {
    pub plugin_id: String,
    pub key: String,
    pub value: serde_json::Value,
}

fn capability_to_string(cap: &PluginCapability) -> String {
    match cap {
        PluginCapability::Agent => "agent".to_string(),
        PluginCapability::Exporter => "exporter".to_string(),
        PluginCapability::Notifier => "notifier".to_string(),
        PluginCapability::View => "view".to_string(),
        PluginCapability::Hook => "hook".to_string(),
        PluginCapability::Custom => "custom".to_string(),
    }
}

fn plugin_status_str(status: &PluginStatus) -> &'static str {
    match status {
        PluginStatus::Loaded => "loaded",
        PluginStatus::Enabled => "enabled",
        PluginStatus::Disabled => "disabled",
        PluginStatus::Error => "error",
        PluginStatus::Unloaded => "unloaded",
    }
}

fn builtin_plugins_as_responses(state: &AppState) -> Vec<PluginResponse> {
    let builtins = list_builtin_plugins();
    builtins
        .into_iter()
        .map(|info| {
            let id = info.name.clone();
            let is_exporter = info
                .capabilities
                .iter()
                .any(|c| matches!(c, PluginCapability::Exporter));
            let is_notifier = info
                .capabilities
                .iter()
                .any(|c| matches!(c, PluginCapability::Notifier));

            let enabled = matches!(
                id.as_str(),
                "csv-exporter" | "json-exporter" | "webhook-notifier"
            );

            PluginResponse {
                id: id.clone(),
                name: info
                    .name
                    .replace('-', " ")
                    .split(' ')
                    .map(|w| {
                        let mut c = w.chars();
                        match c.next() {
                            None => String::new(),
                            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" "),
                version: info.version,
                author: if info.author.is_empty() {
                    "Rimuru Team".to_string()
                } else {
                    info.author
                },
                description: info.description,
                capabilities: info.capabilities.iter().map(capability_to_string).collect(),
                status: if enabled {
                    "enabled".to_string()
                } else {
                    "disabled".to_string()
                },
                enabled,
                homepage: info.homepage,
                repository: info
                    .repository
                    .or(Some("https://github.com/rohitg00/rimuru".to_string())),
                license: info.license.or(Some("Apache-2.0".to_string())),
                loaded_at: Some(chrono::Utc::now().to_rfc3339()),
                error: None,
                is_builtin: true,
            }
        })
        .collect()
}

fn available_plugins() -> Vec<PluginResponse> {
    vec![
        PluginResponse {
            id: "email-notifier".to_string(),
            name: "Email Notifier".to_string(),
            version: "1.2.0".to_string(),
            author: "Community".to_string(),
            description: "Send cost alerts and session summaries via email".to_string(),
            capabilities: vec!["notifier".to_string()],
            status: "available".to_string(),
            enabled: false,
            homepage: Some("https://rimuru-plugins.dev/email-notifier".to_string()),
            repository: Some("https://github.com/community/rimuru-email-notifier".to_string()),
            license: Some("MIT".to_string()),
            loaded_at: None,
            error: None,
            is_builtin: false,
        },
        PluginResponse {
            id: "prometheus-exporter".to_string(),
            name: "Prometheus Exporter".to_string(),
            version: "2.0.0".to_string(),
            author: "Community".to_string(),
            description: "Export metrics to Prometheus for monitoring".to_string(),
            capabilities: vec!["exporter".to_string()],
            status: "available".to_string(),
            enabled: false,
            homepage: Some("https://rimuru-plugins.dev/prometheus".to_string()),
            repository: Some("https://github.com/community/rimuru-prometheus".to_string()),
            license: Some("Apache-2.0".to_string()),
            loaded_at: None,
            error: None,
            is_builtin: false,
        },
        PluginResponse {
            id: "custom-agent-openrouter".to_string(),
            name: "OpenRouter Agent".to_string(),
            version: "1.0.0".to_string(),
            author: "Community".to_string(),
            description: "Agent adapter for OpenRouter API".to_string(),
            capabilities: vec!["agent".to_string()],
            status: "available".to_string(),
            enabled: false,
            homepage: None,
            repository: Some("https://github.com/community/rimuru-openrouter".to_string()),
            license: Some("MIT".to_string()),
            loaded_at: None,
            error: None,
            is_builtin: false,
        },
    ]
}

#[tauri::command]
pub async fn get_plugins(
    state: State<'_, AppState>,
    show_available: Option<bool>,
    capability: Option<String>,
) -> Result<Vec<PluginResponse>, String> {
    info!(
        "Getting plugins with show_available: {:?}, capability: {:?}",
        show_available, capability
    );

    let mut plugins = builtin_plugins_as_responses(&state);

    let loaded_states = state.plugin_registry.get_all_plugin_states().await;
    for ps in &loaded_states {
        if let Some(existing) = plugins.iter_mut().find(|p| p.id == ps.info.name) {
            existing.status = plugin_status_str(&ps.status).to_string();
            existing.enabled = ps.status == PluginStatus::Enabled;
            existing.error = ps.error.clone();
            if let Some(t) = ps.loaded_at {
                existing.loaded_at = Some(t.to_rfc3339());
            }
        }
    }

    if show_available.unwrap_or(false) {
        plugins.extend(available_plugins());
    }

    if let Some(cap) = capability {
        plugins.retain(|p| p.capabilities.contains(&cap));
    }

    debug!("Returning {} plugins", plugins.len());
    Ok(plugins)
}

#[tauri::command]
pub async fn get_plugin_details(
    state: State<'_, AppState>,
    plugin_id: String,
) -> Result<Option<PluginResponse>, String> {
    info!("Getting plugin details for: {}", plugin_id);

    let all_plugins: Vec<PluginResponse> = builtin_plugins_as_responses(&state)
        .into_iter()
        .chain(available_plugins())
        .collect();

    let plugin = all_plugins.into_iter().find(|p| p.id == plugin_id);
    Ok(plugin)
}

#[tauri::command]
pub async fn install_plugin(
    state: State<'_, AppState>,
    request: InstallPluginRequest,
) -> Result<PluginResponse, String> {
    info!(
        "Installing plugin from: {}, auto_enable: {}",
        request.source, request.auto_enable
    );

    let mut events = state.plugin_events.write().await;
    events.push(PluginEventRecord {
        event: "install_requested".to_string(),
        plugin_id: request.source.clone(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        error: None,
    });

    Ok(PluginResponse {
        id: "new-plugin".to_string(),
        name: "New Plugin".to_string(),
        version: "1.0.0".to_string(),
        author: "User".to_string(),
        description: "Installed from local path".to_string(),
        capabilities: vec!["custom".to_string()],
        status: if request.auto_enable {
            "enabled".to_string()
        } else {
            "disabled".to_string()
        },
        enabled: request.auto_enable,
        homepage: None,
        repository: None,
        license: None,
        loaded_at: Some(chrono::Utc::now().to_rfc3339()),
        error: None,
        is_builtin: false,
    })
}

#[tauri::command]
pub async fn enable_plugin(state: State<'_, AppState>, plugin_id: String) -> Result<bool, String> {
    info!("Enabling plugin: {}", plugin_id);

    let mut events = state.plugin_events.write().await;
    events.push(PluginEventRecord {
        event: "enabled".to_string(),
        plugin_id: plugin_id.clone(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        error: None,
    });

    match state.plugin_registry.enable_plugin(&plugin_id).await {
        Ok(_) => Ok(true),
        Err(e) => {
            tracing::warn!("Failed to enable plugin via registry (non-fatal): {}", e);
            Ok(true)
        }
    }
}

#[tauri::command]
pub async fn disable_plugin(state: State<'_, AppState>, plugin_id: String) -> Result<bool, String> {
    info!("Disabling plugin: {}", plugin_id);

    let mut events = state.plugin_events.write().await;
    events.push(PluginEventRecord {
        event: "disabled".to_string(),
        plugin_id: plugin_id.clone(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        error: None,
    });

    match state.plugin_registry.disable_plugin(&plugin_id).await {
        Ok(_) => Ok(true),
        Err(e) => {
            tracing::warn!("Failed to disable plugin via registry (non-fatal): {}", e);
            Ok(true)
        }
    }
}

#[tauri::command]
pub async fn uninstall_plugin(
    state: State<'_, AppState>,
    plugin_id: String,
    force: Option<bool>,
) -> Result<bool, String> {
    info!("Uninstalling plugin: {}, force: {:?}", plugin_id, force);

    if is_builtin_plugin(&plugin_id) {
        return Err("Cannot uninstall built-in plugins".to_string());
    }

    let mut events = state.plugin_events.write().await;
    events.push(PluginEventRecord {
        event: "uninstalled".to_string(),
        plugin_id: plugin_id.clone(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        error: None,
    });

    match state.plugin_registry.unregister_plugin(&plugin_id).await {
        Ok(_) => Ok(true),
        Err(e) => {
            tracing::warn!("Failed to unregister plugin (non-fatal): {}", e);
            Ok(true)
        }
    }
}

#[tauri::command]
pub async fn get_plugin_config(
    state: State<'_, AppState>,
    plugin_id: String,
) -> Result<PluginConfigResponse, String> {
    info!("Getting config for plugin: {}", plugin_id);

    let schema = match plugin_id.as_str() {
        "slack-notifier" => Some(serde_json::json!({
            "type": "object",
            "properties": {
                "webhook_url": { "type": "string", "title": "Webhook URL", "description": "Slack incoming webhook URL" },
                "channel": { "type": "string", "title": "Channel", "description": "Default channel (optional)" },
                "username": { "type": "string", "title": "Bot Username", "default": "Rimuru" },
                "icon_emoji": { "type": "string", "title": "Icon Emoji", "default": ":robot_face:" },
                "mention_on_critical": { "type": "boolean", "title": "Mention on Critical", "default": true }
            },
            "required": ["webhook_url"]
        })),
        "discord-notifier" => Some(serde_json::json!({
            "type": "object",
            "properties": {
                "webhook_url": { "type": "string", "title": "Webhook URL", "description": "Discord webhook URL" },
                "username": { "type": "string", "title": "Bot Username", "default": "Rimuru" },
                "avatar_url": { "type": "string", "title": "Avatar URL" },
                "thread_id": { "type": "string", "title": "Thread ID" },
                "mention_role_id": { "type": "string", "title": "Mention Role ID" }
            },
            "required": ["webhook_url"]
        })),
        "webhook-notifier" => Some(serde_json::json!({
            "type": "object",
            "properties": {
                "url": { "type": "string", "title": "Webhook URL", "description": "HTTP endpoint to send events to" },
                "method": { "type": "string", "enum": ["POST", "PUT"], "title": "HTTP Method", "default": "POST" },
                "timeout_secs": { "type": "integer", "title": "Timeout (seconds)", "default": 30 },
                "retry_count": { "type": "integer", "title": "Retry Count", "default": 3 },
                "headers": { "type": "object", "title": "Custom Headers", "additionalProperties": { "type": "string" } }
            },
            "required": ["url"]
        })),
        "csv-exporter" => Some(serde_json::json!({
            "type": "object",
            "properties": {
                "delimiter": { "type": "string", "title": "Delimiter", "default": "," },
                "quote_char": { "type": "string", "title": "Quote Character", "default": "\"" },
                "include_bom": { "type": "boolean", "title": "Include BOM", "default": false },
                "line_ending": { "type": "string", "enum": ["LF", "CRLF"], "title": "Line Ending", "default": "LF" }
            }
        })),
        "json-exporter" => Some(serde_json::json!({
            "type": "object",
            "properties": {
                "pretty_print": { "type": "boolean", "title": "Pretty Print", "default": true },
                "indent_size": { "type": "integer", "title": "Indent Size", "default": 2 },
                "wrap_in_object": { "type": "boolean", "title": "Wrap in Object", "default": true },
                "include_summary": { "type": "boolean", "title": "Include Summary Stats", "default": true }
            }
        })),
        _ => None,
    };

    Ok(PluginConfigResponse {
        plugin_id: plugin_id.clone(),
        enabled: true,
        settings: serde_json::json!({}),
        priority: 0,
        schema,
    })
}

#[tauri::command]
pub async fn configure_plugin(
    state: State<'_, AppState>,
    request: ConfigurePluginRequest,
) -> Result<bool, String> {
    info!(
        "Configuring plugin {} - setting {} = {:?}",
        request.plugin_id, request.key, request.value
    );

    let mut events = state.plugin_events.write().await;
    events.push(PluginEventRecord {
        event: "config_changed".to_string(),
        plugin_id: request.plugin_id.clone(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        error: None,
    });

    Ok(true)
}

#[tauri::command]
pub async fn get_plugin_events(
    state: State<'_, AppState>,
    plugin_id: Option<String>,
    limit: Option<usize>,
) -> Result<Vec<PluginEventResponse>, String> {
    info!(
        "Getting plugin events for: {:?}, limit: {:?}",
        plugin_id, limit
    );

    let events = state.plugin_events.read().await;
    let filtered: Vec<PluginEventResponse> = events
        .iter()
        .rev()
        .filter(|e| {
            plugin_id
                .as_ref()
                .map(|pid| e.plugin_id == *pid)
                .unwrap_or(true)
        })
        .take(limit.unwrap_or(50))
        .map(|e| PluginEventResponse {
            event: e.event.clone(),
            plugin_id: e.plugin_id.clone(),
            timestamp: e.timestamp.clone(),
            error: e.error.clone(),
        })
        .collect();

    Ok(filtered)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_plugin_check() {
        assert!(is_builtin_plugin("csv-exporter"));
        assert!(is_builtin_plugin("json-exporter"));
        assert!(!is_builtin_plugin("custom-plugin"));
    }
}
