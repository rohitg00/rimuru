#![allow(dead_code, unused_imports, unused_variables, unused_mut)]

use async_trait::async_trait;
use chrono::Utc;
use rimuru_core::models::{CostRecord, Session};
use rimuru_core::plugins::{
    ExportOptions, Notification, NotificationLevel, PluginCapability, PluginConfig, PluginContext,
    PluginDependency, PluginEvent, PluginInfo, PluginPermission, PluginState, PluginStatus,
    PluginType,
};
use rimuru_core::{ExporterPlugin, NotifierPlugin, Plugin, RimuruResult};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

struct MockPluginRegistry {
    plugins: Arc<RwLock<HashMap<String, PluginState>>>,
    events: Arc<RwLock<Vec<PluginEvent>>>,
}

impl MockPluginRegistry {
    fn new() -> Self {
        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
            events: Arc::new(RwLock::new(Vec::new())),
        }
    }

    fn register(&self, info: PluginInfo) {
        let state = PluginState::new(info.clone());
        self.plugins
            .write()
            .unwrap()
            .insert(info.name.clone(), state);
    }

    fn load(&self, name: &str) -> Result<(), String> {
        let mut plugins = self.plugins.write().unwrap();
        if let Some(state) = plugins.get_mut(name) {
            state.status = PluginStatus::Loaded;
            state.loaded_at = Some(Utc::now());
            self.events.write().unwrap().push(PluginEvent::loaded(name));
            Ok(())
        } else {
            Err(format!("Plugin '{}' not found", name))
        }
    }

    fn enable(&self, name: &str) -> Result<(), String> {
        let mut plugins = self.plugins.write().unwrap();
        if let Some(state) = plugins.get_mut(name) {
            if state.status != PluginStatus::Loaded && state.status != PluginStatus::Disabled {
                return Err(format!("Plugin '{}' must be loaded first", name));
            }
            state.status = PluginStatus::Enabled;
            self.events
                .write()
                .unwrap()
                .push(PluginEvent::enabled(name));
            Ok(())
        } else {
            Err(format!("Plugin '{}' not found", name))
        }
    }

    fn disable(&self, name: &str) -> Result<(), String> {
        let mut plugins = self.plugins.write().unwrap();
        if let Some(state) = plugins.get_mut(name) {
            if state.status != PluginStatus::Enabled {
                return Err(format!("Plugin '{}' is not enabled", name));
            }
            state.status = PluginStatus::Disabled;
            self.events
                .write()
                .unwrap()
                .push(PluginEvent::disabled(name));
            Ok(())
        } else {
            Err(format!("Plugin '{}' not found", name))
        }
    }

    fn unload(&self, name: &str) -> Result<(), String> {
        let mut plugins = self.plugins.write().unwrap();
        if let Some(state) = plugins.get_mut(name) {
            state.status = PluginStatus::Unloaded;
            state.loaded_at = None;
            self.events.write().unwrap().push(PluginEvent::Unloaded {
                plugin_id: name.to_string(),
                timestamp: Utc::now(),
            });
            Ok(())
        } else {
            Err(format!("Plugin '{}' not found", name))
        }
    }

    fn get_state(&self, name: &str) -> Option<PluginState> {
        self.plugins.read().unwrap().get(name).cloned()
    }

    fn list_plugins(&self) -> Vec<PluginState> {
        self.plugins.read().unwrap().values().cloned().collect()
    }

    fn get_events(&self) -> Vec<PluginEvent> {
        self.events.read().unwrap().clone()
    }

    fn set_config(&self, name: &str, config: PluginConfig) -> Result<(), String> {
        let mut plugins = self.plugins.write().unwrap();
        if let Some(state) = plugins.get_mut(name) {
            state.config = config;
            self.events
                .write()
                .unwrap()
                .push(PluginEvent::ConfigChanged {
                    plugin_id: name.to_string(),
                    timestamp: Utc::now(),
                });
            Ok(())
        } else {
            Err(format!("Plugin '{}' not found", name))
        }
    }
}

struct MockExporterPlugin {
    name: String,
    info: PluginInfo,
    initialized: bool,
}

impl MockExporterPlugin {
    fn new(name: &str) -> Self {
        let info = PluginInfo::new(name, "1.0.0")
            .with_description("Mock exporter plugin")
            .with_capability(PluginCapability::Exporter);
        Self {
            name: name.to_string(),
            info,
            initialized: false,
        }
    }
}

#[async_trait]
impl Plugin for MockExporterPlugin {
    fn info(&self) -> &PluginInfo {
        &self.info
    }

    async fn init(&mut self, _context: &PluginContext) -> RimuruResult<()> {
        self.initialized = true;
        Ok(())
    }

    async fn shutdown(&mut self) -> RimuruResult<()> {
        self.initialized = false;
        Ok(())
    }

    fn is_initialized(&self) -> bool {
        self.initialized
    }

    fn configure(&mut self, _config: PluginConfig) -> RimuruResult<()> {
        Ok(())
    }
}

#[async_trait]
impl ExporterPlugin for MockExporterPlugin {
    fn format(&self) -> &str {
        "json"
    }

    fn file_extension(&self) -> &str {
        "json"
    }

    async fn export_sessions(
        &self,
        sessions: &[Session],
        _options: ExportOptions,
    ) -> RimuruResult<Vec<u8>> {
        let json = serde_json::to_vec(sessions).unwrap_or_default();
        Ok(json)
    }

    async fn export_costs(
        &self,
        costs: &[CostRecord],
        _options: ExportOptions,
    ) -> RimuruResult<Vec<u8>> {
        let json = serde_json::to_vec(costs).unwrap_or_default();
        Ok(json)
    }
}

struct MockNotifierPlugin {
    name: String,
    info: PluginInfo,
    initialized: bool,
    notifications: Arc<RwLock<Vec<Notification>>>,
    should_fail: Arc<RwLock<bool>>,
}

impl MockNotifierPlugin {
    fn new(name: &str) -> Self {
        let info = PluginInfo::new(name, "1.0.0")
            .with_description("Mock notifier plugin")
            .with_capability(PluginCapability::Notifier);
        Self {
            name: name.to_string(),
            info,
            initialized: false,
            notifications: Arc::new(RwLock::new(Vec::new())),
            should_fail: Arc::new(RwLock::new(false)),
        }
    }

    fn get_notifications(&self) -> Vec<Notification> {
        self.notifications.read().unwrap().clone()
    }

    fn set_should_fail(&self, fail: bool) {
        *self.should_fail.write().unwrap() = fail;
    }
}

#[async_trait]
impl Plugin for MockNotifierPlugin {
    fn info(&self) -> &PluginInfo {
        &self.info
    }

    async fn init(&mut self, _context: &PluginContext) -> RimuruResult<()> {
        self.initialized = true;
        Ok(())
    }

    async fn shutdown(&mut self) -> RimuruResult<()> {
        self.initialized = false;
        Ok(())
    }

    fn is_initialized(&self) -> bool {
        self.initialized
    }

    fn configure(&mut self, _config: PluginConfig) -> RimuruResult<()> {
        Ok(())
    }
}

#[async_trait]
impl NotifierPlugin for MockNotifierPlugin {
    fn notification_type(&self) -> &str {
        "mock"
    }

    async fn send(&self, notification: Notification) -> RimuruResult<()> {
        if *self.should_fail.read().unwrap() {
            return Err(rimuru_core::RimuruError::plugin(
                "Mock notification failure",
            ));
        }
        self.notifications.write().unwrap().push(notification);
        Ok(())
    }

    async fn test_connection(&self) -> RimuruResult<bool> {
        Ok(!*self.should_fail.read().unwrap())
    }
}

mod plugin_registration {
    use super::*;

    #[test]
    fn test_register_plugin() {
        let registry = MockPluginRegistry::new();

        let info = PluginInfo::new("test-plugin", "1.0.0")
            .with_author("Test Author")
            .with_description("A test plugin")
            .with_capability(PluginCapability::Exporter);

        registry.register(info);

        let state = registry.get_state("test-plugin").unwrap();
        assert_eq!(state.info.name, "test-plugin");
        assert_eq!(state.info.version, "1.0.0");
        assert_eq!(state.status, PluginStatus::Unloaded);
    }

    #[test]
    fn test_register_multiple_plugins() {
        let registry = MockPluginRegistry::new();

        for i in 0..5 {
            let info = PluginInfo::new(format!("plugin-{}", i), "1.0.0");
            registry.register(info);
        }

        let plugins = registry.list_plugins();
        assert_eq!(plugins.len(), 5);
    }

    #[test]
    fn test_plugin_info_builder() {
        let info = PluginInfo::new("my-plugin", "2.0.0")
            .with_author("John Doe")
            .with_description("Does something useful")
            .with_capability(PluginCapability::Notifier)
            .with_capability(PluginCapability::Exporter);

        assert_eq!(info.name, "my-plugin");
        assert_eq!(info.version, "2.0.0");
        assert_eq!(info.author, "John Doe");
        assert_eq!(info.capabilities.len(), 2);
    }
}

mod plugin_lifecycle {
    use super::*;

    #[test]
    fn test_load_plugin() {
        let registry = MockPluginRegistry::new();
        let info = PluginInfo::new("loadable", "1.0.0");
        registry.register(info);

        registry.load("loadable").unwrap();

        let state = registry.get_state("loadable").unwrap();
        assert_eq!(state.status, PluginStatus::Loaded);
        assert!(state.loaded_at.is_some());
    }

    #[test]
    fn test_enable_plugin() {
        let registry = MockPluginRegistry::new();
        let info = PluginInfo::new("enableable", "1.0.0");
        registry.register(info);

        registry.load("enableable").unwrap();
        registry.enable("enableable").unwrap();

        let state = registry.get_state("enableable").unwrap();
        assert_eq!(state.status, PluginStatus::Enabled);
        assert!(state.is_active());
    }

    #[test]
    fn test_disable_plugin() {
        let registry = MockPluginRegistry::new();
        let info = PluginInfo::new("disableable", "1.0.0");
        registry.register(info);

        registry.load("disableable").unwrap();
        registry.enable("disableable").unwrap();
        registry.disable("disableable").unwrap();

        let state = registry.get_state("disableable").unwrap();
        assert_eq!(state.status, PluginStatus::Disabled);
        assert!(!state.is_active());
    }

    #[test]
    fn test_unload_plugin() {
        let registry = MockPluginRegistry::new();
        let info = PluginInfo::new("unloadable", "1.0.0");
        registry.register(info);

        registry.load("unloadable").unwrap();
        registry.unload("unloadable").unwrap();

        let state = registry.get_state("unloadable").unwrap();
        assert_eq!(state.status, PluginStatus::Unloaded);
        assert!(state.loaded_at.is_none());
    }

    #[test]
    fn test_full_lifecycle() {
        let registry = MockPluginRegistry::new();
        let info = PluginInfo::new("lifecycle-test", "1.0.0");
        registry.register(info);

        assert_eq!(
            registry.get_state("lifecycle-test").unwrap().status,
            PluginStatus::Unloaded
        );

        registry.load("lifecycle-test").unwrap();
        assert_eq!(
            registry.get_state("lifecycle-test").unwrap().status,
            PluginStatus::Loaded
        );

        registry.enable("lifecycle-test").unwrap();
        assert_eq!(
            registry.get_state("lifecycle-test").unwrap().status,
            PluginStatus::Enabled
        );

        registry.disable("lifecycle-test").unwrap();
        assert_eq!(
            registry.get_state("lifecycle-test").unwrap().status,
            PluginStatus::Disabled
        );

        registry.unload("lifecycle-test").unwrap();
        assert_eq!(
            registry.get_state("lifecycle-test").unwrap().status,
            PluginStatus::Unloaded
        );
    }

    #[test]
    fn test_cannot_enable_unloaded_plugin() {
        let registry = MockPluginRegistry::new();
        let info = PluginInfo::new("unloaded", "1.0.0");
        registry.register(info);

        let result = registry.enable("unloaded");
        assert!(result.is_err());
    }

    #[test]
    fn test_cannot_disable_non_enabled_plugin() {
        let registry = MockPluginRegistry::new();
        let info = PluginInfo::new("loaded-only", "1.0.0");
        registry.register(info);

        registry.load("loaded-only").unwrap();

        let result = registry.disable("loaded-only");
        assert!(result.is_err());
    }
}

mod plugin_events {
    use super::*;

    #[test]
    fn test_load_event() {
        let registry = MockPluginRegistry::new();
        let info = PluginInfo::new("event-test", "1.0.0");
        registry.register(info);

        registry.load("event-test").unwrap();

        let events = registry.get_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].plugin_id(), "event-test");
        assert!(matches!(events[0], PluginEvent::Loaded { .. }));
    }

    #[test]
    fn test_enable_disable_events() {
        let registry = MockPluginRegistry::new();
        let info = PluginInfo::new("events", "1.0.0");
        registry.register(info);

        registry.load("events").unwrap();
        registry.enable("events").unwrap();
        registry.disable("events").unwrap();

        let events = registry.get_events();
        assert_eq!(events.len(), 3);
        assert!(matches!(events[0], PluginEvent::Loaded { .. }));
        assert!(matches!(events[1], PluginEvent::Enabled { .. }));
        assert!(matches!(events[2], PluginEvent::Disabled { .. }));
    }

    #[test]
    fn test_event_timestamps() {
        let registry = MockPluginRegistry::new();
        let info = PluginInfo::new("timestamp-test", "1.0.0");
        registry.register(info);

        let before = Utc::now();
        registry.load("timestamp-test").unwrap();
        let after = Utc::now();

        let events = registry.get_events();
        let event_time = events[0].timestamp();
        assert!(event_time >= before);
        assert!(event_time <= after);
    }

    #[test]
    fn test_plugin_event_builders() {
        let loaded = PluginEvent::loaded("test-plugin");
        assert_eq!(loaded.plugin_id(), "test-plugin");

        let enabled = PluginEvent::enabled("test-plugin");
        assert_eq!(enabled.plugin_id(), "test-plugin");

        let disabled = PluginEvent::disabled("test-plugin");
        assert_eq!(disabled.plugin_id(), "test-plugin");

        let error = PluginEvent::error("test-plugin", "Something went wrong");
        assert_eq!(error.plugin_id(), "test-plugin");
        if let PluginEvent::Error { error: msg, .. } = error {
            assert_eq!(msg, "Something went wrong");
        }
    }
}

mod plugin_configuration {
    use super::*;

    #[test]
    fn test_plugin_config_default() {
        let config = PluginConfig::default();
        assert!(config.enabled);
        assert_eq!(config.priority, 0);
    }

    #[test]
    fn test_plugin_config_disabled() {
        let config = PluginConfig::disabled();
        assert!(!config.enabled);
    }

    #[test]
    fn test_plugin_config_with_settings() {
        let config = PluginConfig::new()
            .with_setting("api_key", "secret123")
            .with_setting("timeout", 30)
            .with_setting("enabled_features", vec!["a", "b", "c"]);

        let api_key: Option<String> = config.get_setting("api_key");
        assert_eq!(api_key, Some("secret123".to_string()));

        let timeout: Option<i64> = config.get_setting("timeout");
        assert_eq!(timeout, Some(30));

        let features: Option<Vec<String>> = config.get_setting("enabled_features");
        assert!(features.is_some());
        assert_eq!(features.unwrap().len(), 3);
    }

    #[test]
    fn test_set_plugin_config() {
        let registry = MockPluginRegistry::new();
        let info = PluginInfo::new("configurable", "1.0.0");
        registry.register(info);

        let new_config =
            PluginConfig::new().with_setting("webhook_url", "https://example.com/hook");

        registry.set_config("configurable", new_config).unwrap();

        let state = registry.get_state("configurable").unwrap();
        let url: Option<String> = state.config.get_setting("webhook_url");
        assert_eq!(url, Some("https://example.com/hook".to_string()));

        let events = registry.get_events();
        assert!(events
            .iter()
            .any(|e| matches!(e, PluginEvent::ConfigChanged { .. })));
    }
}

mod plugin_dependencies {
    use super::*;

    #[test]
    fn test_required_dependency() {
        let dep = PluginDependency::required("core-plugin", ">=1.0.0");
        assert_eq!(dep.name, "core-plugin");
        assert_eq!(dep.version_requirement, ">=1.0.0");
        assert!(!dep.optional);
    }

    #[test]
    fn test_optional_dependency() {
        let dep = PluginDependency::optional("optional-feature", "^2.0.0");
        assert_eq!(dep.name, "optional-feature");
        assert!(dep.optional);
    }

    #[test]
    fn test_multiple_dependencies() {
        let deps = [
            PluginDependency::required("core", "1.0.0"),
            PluginDependency::required("logging", "2.0.0"),
            PluginDependency::optional("analytics", "1.5.0"),
        ];

        let required: Vec<_> = deps.iter().filter(|d| !d.optional).collect();
        assert_eq!(required.len(), 2);

        let optional: Vec<_> = deps.iter().filter(|d| d.optional).collect();
        assert_eq!(optional.len(), 1);
    }
}

mod plugin_context {
    use super::*;

    #[test]
    fn test_plugin_context_creation() {
        let ctx = PluginContext::new("my-plugin", "/data/plugins/my-plugin");

        assert_eq!(ctx.plugin_id, "my-plugin");
        assert_eq!(ctx.data_dir, PathBuf::from("/data/plugins/my-plugin"));
    }

    #[test]
    fn test_plugin_context_with_config() {
        let config = PluginConfig::new().with_setting("debug", true);

        let ctx = PluginContext::new("test", "/tmp").with_config(config);

        let debug: Option<bool> = ctx.config.get_setting("debug");
        assert_eq!(debug, Some(true));
    }

    #[test]
    fn test_plugin_context_metadata() {
        let mut ctx = PluginContext::new("meta-test", "/tmp");

        ctx.set_metadata("version", "1.2.3");
        ctx.set_metadata("initialized", true);
        ctx.set_metadata("features", vec!["feature1", "feature2"]);

        let version: Option<String> = ctx.get_metadata("version");
        assert_eq!(version, Some("1.2.3".to_string()));

        let initialized: Option<bool> = ctx.get_metadata("initialized");
        assert_eq!(initialized, Some(true));

        let nonexistent: Option<String> = ctx.get_metadata("nonexistent");
        assert!(nonexistent.is_none());
    }
}

mod exporter_plugin_flow {
    use super::*;

    #[tokio::test]
    async fn test_exporter_supported_format() {
        let plugin = MockExporterPlugin::new("format-test");

        assert_eq!(plugin.format(), "json");
        assert_eq!(plugin.file_extension(), "json");
    }

    #[tokio::test]
    async fn test_exporter_plugin_info() {
        let plugin = MockExporterPlugin::new("info-test");

        let info = plugin.info();
        assert_eq!(info.name, "info-test");
        assert!(info.capabilities.contains(&PluginCapability::Exporter));
    }

    #[tokio::test]
    async fn test_exporter_export_sessions() {
        let plugin = MockExporterPlugin::new("session-export");

        let sessions: Vec<Session> = vec![];
        let options = ExportOptions::default();

        let result = plugin.export_sessions(&sessions, options).await.unwrap();
        assert!(!result.is_empty());
    }
}

mod notifier_plugin_flow {
    use super::*;

    #[tokio::test]
    async fn test_notifier_plugin_send() {
        let plugin = MockNotifierPlugin::new("test-notifier");

        let notification = Notification::info("Test Notification", "This is a test message");

        plugin.send(notification).await.unwrap();

        let notifications = plugin.get_notifications();
        assert_eq!(notifications.len(), 1);
        assert_eq!(notifications[0].title, "Test Notification");
    }

    #[tokio::test]
    async fn test_notifier_plugin_failure() {
        let plugin = MockNotifierPlugin::new("failing-notifier");
        plugin.set_should_fail(true);

        let notification = Notification::error("Error", "Something went wrong");

        let result = plugin.send(notification).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_notifier_notification_levels() {
        let info = Notification::info("Info", "info msg");
        assert_eq!(info.level, NotificationLevel::Info);

        let warn = Notification::warning("Warn", "warn msg");
        assert_eq!(warn.level, NotificationLevel::Warning);

        let err = Notification::error("Err", "err msg");
        assert_eq!(err.level, NotificationLevel::Error);

        let crit = Notification::critical("Crit", "crit msg");
        assert_eq!(crit.level, NotificationLevel::Critical);
    }

    #[tokio::test]
    async fn test_multiple_notifications() {
        let plugin = MockNotifierPlugin::new("multi-notify");

        for i in 0..5 {
            let notification =
                Notification::info(format!("Notification {}", i), format!("Message {}", i));
            plugin.send(notification).await.unwrap();
        }

        let notifications = plugin.get_notifications();
        assert_eq!(notifications.len(), 5);
    }

    #[tokio::test]
    async fn test_notifier_test_connection() {
        let plugin = MockNotifierPlugin::new("conn-test");
        assert!(plugin.test_connection().await.unwrap());

        plugin.set_should_fail(true);
        assert!(!plugin.test_connection().await.unwrap());
    }
}

mod plugin_types {
    use super::*;

    #[test]
    fn test_plugin_capability_display() {
        assert_eq!(PluginCapability::Agent.to_string(), "agent");
        assert_eq!(PluginCapability::Exporter.to_string(), "exporter");
        assert_eq!(PluginCapability::Notifier.to_string(), "notifier");
        assert_eq!(PluginCapability::View.to_string(), "view");
        assert_eq!(PluginCapability::Hook.to_string(), "hook");
        assert_eq!(PluginCapability::Custom.to_string(), "custom");
    }

    #[test]
    fn test_plugin_status_default() {
        let status = PluginStatus::default();
        assert_eq!(status, PluginStatus::Unloaded);
    }

    #[test]
    fn test_plugin_type_default() {
        let ptype = PluginType::default();
        assert_eq!(ptype, PluginType::Native);
    }

    #[test]
    fn test_plugin_state_is_active() {
        let info = PluginInfo::new("state-test", "1.0.0");
        let mut state = PluginState::new(info);

        assert!(!state.is_active());

        state.status = PluginStatus::Enabled;
        assert!(state.is_active());

        state.status = PluginStatus::Disabled;
        assert!(!state.is_active());
    }
}

mod error_handling {
    use super::*;

    #[test]
    fn test_load_nonexistent_plugin() {
        let registry = MockPluginRegistry::new();

        let result = registry.load("nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_enable_nonexistent_plugin() {
        let registry = MockPluginRegistry::new();

        let result = registry.enable("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_disable_nonexistent_plugin() {
        let registry = MockPluginRegistry::new();

        let result = registry.disable("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_set_config_nonexistent_plugin() {
        let registry = MockPluginRegistry::new();

        let result = registry.set_config("nonexistent", PluginConfig::default());
        assert!(result.is_err());
    }
}

mod plugin_permissions {
    use super::*;

    #[test]
    fn test_plugin_permission() {
        let permission = PluginPermission {
            name: "network".to_string(),
            description: "Access to network operations".to_string(),
            required: true,
        };

        assert_eq!(permission.name, "network");
        assert!(permission.required);
    }

    #[test]
    fn test_multiple_permissions() {
        let permissions = [
            PluginPermission {
                name: "filesystem".to_string(),
                description: "Read/write files".to_string(),
                required: true,
            },
            PluginPermission {
                name: "network".to_string(),
                description: "HTTP requests".to_string(),
                required: false,
            },
        ];

        let required: Vec<_> = permissions.iter().filter(|p| p.required).collect();
        assert_eq!(required.len(), 1);
    }
}
