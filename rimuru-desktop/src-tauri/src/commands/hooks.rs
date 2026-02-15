use crate::state::AppState;
use rimuru_core::{Hook, HookContext, HookData, HookResult as CoreHookResult};
use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::{debug, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookTypeResponse {
    pub name: String,
    pub description: String,
    pub data_type: String,
    pub handler_count: usize,
    pub enabled: bool,
    pub last_triggered: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookHandlerResponse {
    pub id: String,
    pub name: String,
    pub hook_type: String,
    pub priority: i32,
    pub enabled: bool,
    pub plugin_id: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookExecutionResponse {
    pub id: String,
    pub hook_type: String,
    pub handler_name: String,
    pub status: String,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub duration_ms: Option<u64>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerHookRequest {
    pub hook_name: String,
    pub data: Option<serde_json::Value>,
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerHookResponse {
    pub success: bool,
    pub handlers_executed: usize,
    pub aborted: bool,
    pub abort_reason: Option<String>,
    pub execution_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookStatsResponse {
    pub total_hook_types: usize,
    pub active_handlers: usize,
    pub total_handlers: usize,
    pub total_executions: usize,
    pub successful_executions: usize,
    pub failed_executions: usize,
    pub aborted_executions: usize,
}

fn hook_description(hook: &Hook) -> &'static str {
    match hook {
        Hook::PreSessionStart => "Fired before a session starts",
        Hook::PostSessionEnd => "Fired after a session ends",
        Hook::OnCostRecorded => "Fired when a cost is recorded",
        Hook::OnMetricsCollected => "Fired when metrics are collected",
        Hook::OnAgentConnect => "Fired when an agent connects",
        Hook::OnAgentDisconnect => "Fired when an agent disconnects",
        Hook::OnSyncComplete => "Fired after sync completes",
        Hook::OnPluginLoaded => "Fired when a plugin is loaded",
        Hook::OnPluginUnloaded => "Fired when a plugin is unloaded",
        Hook::OnConfigChanged => "Fired when config changes",
        Hook::OnError => "Fired on errors",
        Hook::Custom(_) => "Custom hook",
    }
}

fn hook_data_type(hook: &Hook) -> &'static str {
    match hook {
        Hook::PreSessionStart | Hook::PostSessionEnd => "Session",
        Hook::OnCostRecorded => "Cost",
        Hook::OnMetricsCollected => "Metrics",
        Hook::OnAgentConnect | Hook::OnAgentDisconnect => "Agent",
        Hook::OnSyncComplete => "Sync",
        Hook::OnPluginLoaded | Hook::OnPluginUnloaded => "Plugin",
        Hook::OnConfigChanged => "Config",
        Hook::OnError => "Error",
        Hook::Custom(_) => "Custom",
    }
}

#[tauri::command]
pub async fn get_hooks(state: State<'_, AppState>) -> Result<Vec<HookTypeResponse>, String> {
    info!("Getting all hook types");

    let all_hooks = Hook::all_standard();
    let all_handlers = state.hook_manager.get_all_handlers().await;
    let last_triggered = state.hook_last_triggered.read().await;

    let mut responses = Vec::new();
    for hook in all_hooks {
        let handler_count = all_handlers.get(&hook).map(|h| h.len()).unwrap_or(0);

        responses.push(HookTypeResponse {
            name: hook.name().to_string(),
            description: hook_description(&hook).to_string(),
            data_type: hook_data_type(&hook).to_string(),
            handler_count,
            enabled: true,
            last_triggered: last_triggered.get(hook.name()).map(|t| t.to_rfc3339()),
        });
    }

    Ok(responses)
}

#[tauri::command]
pub async fn get_hook_handlers(
    state: State<'_, AppState>,
    hook_type: Option<String>,
) -> Result<Vec<HookHandlerResponse>, String> {
    info!("Getting hook handlers for: {:?}", hook_type);

    let all_handlers = state.hook_manager.get_all_handlers().await;

    let mut responses = Vec::new();
    for (hook, handlers) in &all_handlers {
        if let Some(ref ht) = hook_type {
            if hook.name() != ht.as_str() {
                continue;
            }
        }
        for h in handlers {
            responses.push(HookHandlerResponse {
                id: format!("{}/{}", hook.name(), h.name),
                name: h.name.clone(),
                hook_type: hook.name().to_string(),
                priority: h.priority,
                enabled: h.enabled,
                plugin_id: h.plugin_id.clone(),
                description: h.description.clone(),
            });
        }
    }

    debug!("Returning {} handlers", responses.len());
    Ok(responses)
}

#[tauri::command]
pub async fn get_hook_executions(
    state: State<'_, AppState>,
    hook_type: Option<String>,
    limit: Option<usize>,
) -> Result<Vec<HookExecutionResponse>, String> {
    info!(
        "Getting hook executions for: {:?}, limit: {:?}",
        hook_type, limit
    );

    let limit = limit.unwrap_or(100);

    let executions = if let Some(ht) = hook_type {
        let hook = Hook::from_name(&ht);
        state
            .hook_manager
            .get_executions_for_hook(&hook, limit)
            .await
    } else {
        state.hook_manager.get_recent_executions(limit).await
    };

    let responses: Vec<HookExecutionResponse> = executions
        .into_iter()
        .map(|e| {
            let status = if e.error.is_some() {
                "failed".to_string()
            } else if e.result.as_ref().map(|r| r.is_skip()).unwrap_or(false) {
                "skipped".to_string()
            } else {
                "success".to_string()
            };

            HookExecutionResponse {
                id: e.id.to_string(),
                hook_type: e.hook.name().to_string(),
                handler_name: e.handler_name,
                status,
                started_at: e.started_at.to_rfc3339(),
                completed_at: e.completed_at.map(|t| t.to_rfc3339()),
                duration_ms: e.duration_ms,
                error: e.error,
            }
        })
        .collect();

    Ok(responses)
}

#[tauri::command]
pub async fn trigger_hook(
    state: State<'_, AppState>,
    request: TriggerHookRequest,
) -> Result<TriggerHookResponse, String> {
    info!(
        "Triggering hook: {} with source: {:?}",
        request.hook_name, request.source
    );

    let hook = Hook::from_name(&request.hook_name);

    let data = match request.data {
        Some(v) => HookData::Custom(v),
        None => HookData::None,
    };

    let ctx = HookContext::new(hook.clone(), data)
        .with_source(request.source.unwrap_or_else(|| "desktop_ui".to_string()));

    let handlers = state.hook_manager.get_handlers(&hook).await;
    let handler_count = handlers.iter().filter(|h| h.enabled).count();

    {
        let mut last_triggered = state.hook_last_triggered.write().await;
        last_triggered.insert(hook.name().to_string(), chrono::Utc::now());
    }

    let execution_id = ctx.correlation_id.to_string();

    match state.hook_manager.execute(ctx).await {
        Ok(_result) => Ok(TriggerHookResponse {
            success: true,
            handlers_executed: handler_count,
            aborted: false,
            abort_reason: None,
            execution_id,
        }),
        Err(e) => {
            let err_str = e.to_string();
            let is_aborted = err_str.contains("aborted");
            Ok(TriggerHookResponse {
                success: !is_aborted,
                handlers_executed: handler_count,
                aborted: is_aborted,
                abort_reason: if is_aborted {
                    Some(err_str.clone())
                } else {
                    None
                },
                execution_id,
            })
        }
    }
}

#[tauri::command]
pub async fn enable_hook_handler(
    state: State<'_, AppState>,
    handler_id: String,
) -> Result<bool, String> {
    info!("Enabling hook handler: {}", handler_id);

    let parts: Vec<&str> = handler_id.splitn(2, '/').collect();
    if parts.len() != 2 {
        return Err(format!(
            "Invalid handler ID format '{}'. Expected 'hook_name/handler_name'",
            handler_id
        ));
    }

    let hook = Hook::from_name(parts[0]);
    state
        .hook_manager
        .enable_handler(&hook, parts[1])
        .await
        .map_err(|e| e.to_string())?;

    Ok(true)
}

#[tauri::command]
pub async fn disable_hook_handler(
    state: State<'_, AppState>,
    handler_id: String,
) -> Result<bool, String> {
    info!("Disabling hook handler: {}", handler_id);

    let parts: Vec<&str> = handler_id.splitn(2, '/').collect();
    if parts.len() != 2 {
        return Err(format!(
            "Invalid handler ID format '{}'. Expected 'hook_name/handler_name'",
            handler_id
        ));
    }

    let hook = Hook::from_name(parts[0]);
    state
        .hook_manager
        .disable_handler(&hook, parts[1])
        .await
        .map_err(|e| e.to_string())?;

    Ok(true)
}

#[tauri::command]
pub async fn get_hook_stats(state: State<'_, AppState>) -> Result<HookStatsResponse, String> {
    info!("Getting hook stats");

    let all_handlers = state.hook_manager.get_all_handlers().await;
    let total_handlers: usize = all_handlers.values().map(|h| h.len()).sum();
    let active_handlers: usize = all_handlers
        .values()
        .flat_map(|h| h.iter())
        .filter(|h| h.enabled)
        .count();

    let executions = state.hook_manager.get_recent_executions(10000).await;
    let total = executions.len();
    let successful = executions
        .iter()
        .filter(|e| e.error.is_none() && e.result.is_some())
        .count();
    let failed = executions.iter().filter(|e| e.error.is_some()).count();
    let aborted = total.saturating_sub(successful + failed);

    Ok(HookStatsResponse {
        total_hook_types: Hook::all_standard().len(),
        active_handlers,
        total_handlers,
        total_executions: total,
        successful_executions: successful,
        failed_executions: failed,
        aborted_executions: aborted,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_descriptions() {
        let hooks = Hook::all_standard();
        assert_eq!(hooks.len(), 11);
        for hook in &hooks {
            assert!(!hook_description(hook).is_empty());
            assert!(!hook_data_type(hook).is_empty());
        }
    }
}
