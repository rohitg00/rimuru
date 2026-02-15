#![allow(
    dead_code,
    unused_imports,
    unused_variables,
    clippy::too_many_arguments,
    clippy::needless_borrows_for_generic_args,
    clippy::map_entry
)]

pub mod macros;
mod prelude;

pub use prelude::*;

pub use rimuru_core::error::{RimuruError, RimuruResult};

pub use rimuru_core::plugins::{
    create_builtin_exporter, create_builtin_notifier, create_example_manifest, is_builtin_plugin,
    list_builtin_plugins, AgentCapability, AgentPlugin, CapabilitiesSection, ConfigSection,
    CsvExporterConfig, CsvExporterPlugin, DiscordNotifierConfig, DiscordNotifierPlugin,
    DynAgentPlugin, DynExporterPlugin, DynNotifierPlugin, DynPlugin, DynViewPlugin, ExportData,
    ExportOptions, ExporterCapability, ExporterPlugin, HookRegistration, HttpMethod,
    JsonExporterConfig, JsonExporterPlugin, LineEnding, Notification, NotificationLevel,
    NotifierCapability, NotifierPlugin, Plugin, PluginCapability, PluginConfig, PluginContext,
    PluginDependency, PluginEvent, PluginFactory, PluginInfo, PluginManifest, PluginMetadata,
    PluginPermission, PluginState, PluginStatus, PluginType, SessionCallback, SlackNotifierConfig,
    SlackNotifierPlugin, ViewAction, ViewCapability, ViewContext, ViewInput, ViewOutput,
    ViewPlugin, WebhookNotifierConfig, WebhookNotifierPlugin, WidgetData,
};

pub use rimuru_core::hooks::{
    trigger_hook, trigger_hook_with_data, CostAlertConfig, CostAlertHandler, DynHookHandler, Hook,
    HookConfig, HookContext, HookData, HookExecution, HookHandler, HookHandlerInfo, HookManager,
    HookResult, MetricsExportConfig, MetricsExportHandler, SessionLogConfig, SessionLogFormat,
    SessionLogHandler, SessionStartLogHandler, WebhookConfig, WebhookHandler,
};

pub use rimuru_core::models::{AgentType, CostRecord, MetricsSnapshot, Session, SessionStatus};

pub mod version {
    pub const SDK_VERSION: &str = env!("CARGO_PKG_VERSION");
    pub const MIN_RIMURU_VERSION: &str = "0.1.0";

    pub fn is_compatible(rimuru_version: &str) -> bool {
        let min_parts: Vec<u32> = MIN_RIMURU_VERSION
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect();
        let version_parts: Vec<u32> = rimuru_version
            .split('.')
            .filter_map(|s| s.parse().ok())
            .collect();

        if version_parts.len() < 2 || min_parts.len() < 2 {
            return false;
        }

        (version_parts[0], version_parts[1]) >= (min_parts[0], min_parts[1])
    }
}

pub mod helpers {
    use serde_json::json;

    pub fn create_config_schema(properties: serde_json::Value) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": properties
        })
    }

    pub fn string_property(description: &str, default: Option<&str>) -> serde_json::Value {
        let mut prop = json!({
            "type": "string",
            "description": description
        });
        if let Some(d) = default {
            prop["default"] = json!(d);
        }
        prop
    }

    pub fn number_property(description: &str, default: Option<f64>) -> serde_json::Value {
        let mut prop = json!({
            "type": "number",
            "description": description
        });
        if let Some(d) = default {
            prop["default"] = json!(d);
        }
        prop
    }

    pub fn integer_property(description: &str, default: Option<i64>) -> serde_json::Value {
        let mut prop = json!({
            "type": "integer",
            "description": description
        });
        if let Some(d) = default {
            prop["default"] = json!(d);
        }
        prop
    }

    pub fn boolean_property(description: &str, default: Option<bool>) -> serde_json::Value {
        let mut prop = json!({
            "type": "boolean",
            "description": description
        });
        if let Some(d) = default {
            prop["default"] = json!(d);
        }
        prop
    }

    pub fn enum_property(
        description: &str,
        values: &[&str],
        default: Option<&str>,
    ) -> serde_json::Value {
        let mut prop = json!({
            "type": "string",
            "description": description,
            "enum": values
        });
        if let Some(d) = default {
            prop["default"] = json!(d);
        }
        prop
    }

    pub fn array_property(description: &str, item_type: &str) -> serde_json::Value {
        json!({
            "type": "array",
            "description": description,
            "items": {
                "type": item_type
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::helpers;
    use crate::version;

    #[test]
    fn test_version_compatibility() {
        assert!(version::is_compatible("0.1.0"));
        assert!(version::is_compatible("0.2.0"));
        assert!(version::is_compatible("1.0.0"));
        assert!(!version::is_compatible("0.0.1"));
    }

    #[test]
    fn test_config_schema_helpers() {
        let schema = helpers::create_config_schema(serde_json::json!({
            "name": helpers::string_property("Plugin name", Some("default")),
            "count": helpers::integer_property("Item count", Some(10)),
            "enabled": helpers::boolean_property("Enable feature", Some(true)),
            "level": helpers::enum_property("Log level", &["debug", "info", "warn", "error"], Some("info"))
        }));

        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["name"]["type"] == "string");
        assert!(schema["properties"]["count"]["type"] == "integer");
    }
}
