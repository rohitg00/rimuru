use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::models::{CostRecord, MetricsSnapshot, Session};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Hook {
    PreSessionStart,
    PostSessionEnd,
    OnCostRecorded,
    OnMetricsCollected,
    OnAgentConnect,
    OnAgentDisconnect,
    OnSyncComplete,
    OnPluginLoaded,
    OnPluginUnloaded,
    OnConfigChanged,
    OnError,
    Custom(String),
}

impl Hook {
    pub fn name(&self) -> &str {
        match self {
            Hook::PreSessionStart => "pre_session_start",
            Hook::PostSessionEnd => "post_session_end",
            Hook::OnCostRecorded => "on_cost_recorded",
            Hook::OnMetricsCollected => "on_metrics_collected",
            Hook::OnAgentConnect => "on_agent_connect",
            Hook::OnAgentDisconnect => "on_agent_disconnect",
            Hook::OnSyncComplete => "on_sync_complete",
            Hook::OnPluginLoaded => "on_plugin_loaded",
            Hook::OnPluginUnloaded => "on_plugin_unloaded",
            Hook::OnConfigChanged => "on_config_changed",
            Hook::OnError => "on_error",
            Hook::Custom(name) => name,
        }
    }

    pub fn from_name(name: &str) -> Self {
        match name {
            "pre_session_start" => Hook::PreSessionStart,
            "post_session_end" => Hook::PostSessionEnd,
            "on_cost_recorded" => Hook::OnCostRecorded,
            "on_metrics_collected" => Hook::OnMetricsCollected,
            "on_agent_connect" => Hook::OnAgentConnect,
            "on_agent_disconnect" => Hook::OnAgentDisconnect,
            "on_sync_complete" => Hook::OnSyncComplete,
            "on_plugin_loaded" => Hook::OnPluginLoaded,
            "on_plugin_unloaded" => Hook::OnPluginUnloaded,
            "on_config_changed" => Hook::OnConfigChanged,
            "on_error" => Hook::OnError,
            custom => Hook::Custom(custom.to_string()),
        }
    }

    pub fn all_standard() -> Vec<Self> {
        vec![
            Hook::PreSessionStart,
            Hook::PostSessionEnd,
            Hook::OnCostRecorded,
            Hook::OnMetricsCollected,
            Hook::OnAgentConnect,
            Hook::OnAgentDisconnect,
            Hook::OnSyncComplete,
            Hook::OnPluginLoaded,
            Hook::OnPluginUnloaded,
            Hook::OnConfigChanged,
            Hook::OnError,
        ]
    }
}

impl std::fmt::Display for Hook {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
#[derive(Default)]
pub enum HookData {
    Session(Session),
    Cost(CostRecord),
    Metrics(MetricsSnapshot),
    Agent {
        agent_id: Uuid,
        agent_name: String,
        agent_type: String,
    },
    Sync {
        provider: String,
        models_synced: usize,
        duration_ms: u64,
    },
    Plugin {
        plugin_id: String,
        plugin_name: String,
    },
    Config {
        changed_keys: Vec<String>,
    },
    Error {
        error_code: String,
        error_message: String,
        source: Option<String>,
    },
    Custom(serde_json::Value),
    #[default]
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookContext {
    pub hook: Hook,
    pub data: HookData,
    pub timestamp: DateTime<Utc>,
    pub source: String,
    pub correlation_id: Uuid,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl HookContext {
    pub fn new(hook: Hook, data: HookData) -> Self {
        Self {
            hook,
            data,
            timestamp: Utc::now(),
            source: String::new(),
            correlation_id: Uuid::new_v4(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = source.into();
        self
    }

    pub fn with_correlation_id(mut self, id: Uuid) -> Self {
        self.correlation_id = id;
        self
    }

    pub fn with_metadata<V: Serialize>(mut self, key: impl Into<String>, value: V) -> Self {
        if let Ok(v) = serde_json::to_value(value) {
            self.metadata.insert(key.into(), v);
        }
        self
    }

    pub fn get_metadata<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        self.metadata
            .get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    pub fn session_start(session: Session) -> Self {
        Self::new(Hook::PreSessionStart, HookData::Session(session)).with_source("session_manager")
    }

    pub fn session_end(session: Session) -> Self {
        Self::new(Hook::PostSessionEnd, HookData::Session(session)).with_source("session_manager")
    }

    pub fn cost_recorded(cost: CostRecord) -> Self {
        Self::new(Hook::OnCostRecorded, HookData::Cost(cost)).with_source("cost_tracker")
    }

    pub fn metrics_collected(metrics: MetricsSnapshot) -> Self {
        Self::new(Hook::OnMetricsCollected, HookData::Metrics(metrics))
            .with_source("metrics_collector")
    }

    pub fn agent_connect(
        agent_id: Uuid,
        agent_name: impl Into<String>,
        agent_type: impl Into<String>,
    ) -> Self {
        Self::new(
            Hook::OnAgentConnect,
            HookData::Agent {
                agent_id,
                agent_name: agent_name.into(),
                agent_type: agent_type.into(),
            },
        )
        .with_source("adapter_manager")
    }

    pub fn agent_disconnect(
        agent_id: Uuid,
        agent_name: impl Into<String>,
        agent_type: impl Into<String>,
    ) -> Self {
        Self::new(
            Hook::OnAgentDisconnect,
            HookData::Agent {
                agent_id,
                agent_name: agent_name.into(),
                agent_type: agent_type.into(),
            },
        )
        .with_source("adapter_manager")
    }

    pub fn sync_complete(
        provider: impl Into<String>,
        models_synced: usize,
        duration_ms: u64,
    ) -> Self {
        Self::new(
            Hook::OnSyncComplete,
            HookData::Sync {
                provider: provider.into(),
                models_synced,
                duration_ms,
            },
        )
        .with_source("sync_scheduler")
    }

    pub fn error(
        error_code: impl Into<String>,
        error_message: impl Into<String>,
        source: Option<String>,
    ) -> Self {
        Self::new(
            Hook::OnError,
            HookData::Error {
                error_code: error_code.into(),
                error_message: error_message.into(),
                source,
            },
        )
        .with_source("error_handler")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum HookResult {
    #[default]
    Continue,
    Abort {
        reason: String,
    },
    Modified {
        data: HookData,
        message: Option<String>,
    },
    Skip,
}

impl HookResult {
    pub fn ok() -> Self {
        HookResult::Continue
    }

    pub fn abort(reason: impl Into<String>) -> Self {
        HookResult::Abort {
            reason: reason.into(),
        }
    }

    pub fn modified(data: HookData) -> Self {
        HookResult::Modified {
            data,
            message: None,
        }
    }

    pub fn modified_with_message(data: HookData, message: impl Into<String>) -> Self {
        HookResult::Modified {
            data,
            message: Some(message.into()),
        }
    }

    pub fn skip() -> Self {
        HookResult::Skip
    }

    pub fn is_continue(&self) -> bool {
        matches!(self, HookResult::Continue)
    }

    pub fn is_abort(&self) -> bool {
        matches!(self, HookResult::Abort { .. })
    }

    pub fn is_modified(&self) -> bool {
        matches!(self, HookResult::Modified { .. })
    }

    pub fn is_skip(&self) -> bool {
        matches!(self, HookResult::Skip)
    }

    pub fn get_modified_data(&self) -> Option<&HookData> {
        match self {
            HookResult::Modified { data, .. } => Some(data),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookExecution {
    pub id: Uuid,
    pub hook: Hook,
    pub handler_name: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub result: Option<HookResult>,
    pub error: Option<String>,
    pub duration_ms: Option<u64>,
}

impl HookExecution {
    pub fn new(hook: Hook, handler_name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            hook,
            handler_name: handler_name.into(),
            started_at: Utc::now(),
            completed_at: None,
            result: None,
            error: None,
            duration_ms: None,
        }
    }

    pub fn complete(mut self, result: HookResult) -> Self {
        let now = Utc::now();
        self.completed_at = Some(now);
        self.duration_ms = Some((now - self.started_at).num_milliseconds() as u64);
        self.result = Some(result);
        self
    }

    pub fn fail(mut self, error: impl Into<String>) -> Self {
        let now = Utc::now();
        self.completed_at = Some(now);
        self.duration_ms = Some((now - self.started_at).num_milliseconds() as u64);
        self.error = Some(error.into());
        self
    }

    pub fn is_successful(&self) -> bool {
        self.error.is_none() && self.result.is_some()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookHandlerInfo {
    pub name: String,
    pub hook: Hook,
    pub priority: i32,
    pub enabled: bool,
    pub plugin_id: Option<String>,
    pub description: Option<String>,
}

impl HookHandlerInfo {
    pub fn new(name: impl Into<String>, hook: Hook) -> Self {
        Self {
            name: name.into(),
            hook,
            priority: 0,
            enabled: true,
            plugin_id: None,
            description: None,
        }
    }

    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn from_plugin(mut self, plugin_id: impl Into<String>) -> Self {
        self.plugin_id = Some(plugin_id.into());
        self
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct HookConfig {
    pub timeout_ms: u64,
    pub enabled: bool,
    pub max_handlers: usize,
    pub parallel_execution: bool,
}

impl Default for HookConfig {
    fn default() -> Self {
        Self {
            timeout_ms: 5000,
            enabled: true,
            max_handlers: 100,
            parallel_execution: false,
        }
    }
}

impl HookConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_timeout(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    pub fn disabled(mut self) -> Self {
        self.enabled = false;
        self
    }

    pub fn with_max_handlers(mut self, max: usize) -> Self {
        self.max_handlers = max;
        self
    }

    pub fn parallel(mut self) -> Self {
        self.parallel_execution = true;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_name() {
        assert_eq!(Hook::PreSessionStart.name(), "pre_session_start");
        assert_eq!(Hook::PostSessionEnd.name(), "post_session_end");
        assert_eq!(Hook::OnCostRecorded.name(), "on_cost_recorded");
        assert_eq!(Hook::Custom("my_hook".to_string()).name(), "my_hook");
    }

    #[test]
    fn test_hook_from_name() {
        assert_eq!(Hook::from_name("pre_session_start"), Hook::PreSessionStart);
        assert_eq!(Hook::from_name("post_session_end"), Hook::PostSessionEnd);
        assert_eq!(
            Hook::from_name("custom_hook"),
            Hook::Custom("custom_hook".to_string())
        );
    }

    #[test]
    fn test_hook_context_creation() {
        let ctx = HookContext::new(Hook::OnCostRecorded, HookData::None)
            .with_source("test")
            .with_metadata("key", "value");

        assert_eq!(ctx.hook, Hook::OnCostRecorded);
        assert_eq!(ctx.source, "test");
        assert_eq!(ctx.get_metadata::<String>("key"), Some("value".to_string()));
    }

    #[test]
    fn test_hook_result_variants() {
        assert!(HookResult::ok().is_continue());
        assert!(HookResult::abort("reason").is_abort());
        assert!(HookResult::modified(HookData::None).is_modified());
        assert!(HookResult::skip().is_skip());
    }

    #[test]
    fn test_hook_execution() {
        let execution = HookExecution::new(Hook::PreSessionStart, "test_handler");
        assert!(execution.completed_at.is_none());
        assert!(execution.result.is_none());

        let completed = execution.complete(HookResult::ok());
        assert!(completed.completed_at.is_some());
        assert!(completed.is_successful());
    }

    #[test]
    fn test_hook_handler_info() {
        let info = HookHandlerInfo::new("cost_alert", Hook::OnCostRecorded)
            .with_priority(10)
            .with_description("Alerts when cost exceeds threshold")
            .from_plugin("builtin");

        assert_eq!(info.name, "cost_alert");
        assert_eq!(info.priority, 10);
        assert_eq!(info.plugin_id, Some("builtin".to_string()));
    }

    #[test]
    fn test_hook_config() {
        let config = HookConfig::new()
            .with_timeout(10000)
            .with_max_handlers(50)
            .parallel();

        assert_eq!(config.timeout_ms, 10000);
        assert_eq!(config.max_handlers, 50);
        assert!(config.parallel_execution);
    }
}
