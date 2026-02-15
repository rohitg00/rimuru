use chrono::Utc;
use rimuru_core::plugins::{
    PluginCapability, PluginConfig, PluginContext, PluginDependency, PluginEvent, PluginInfo,
    PluginPermission, PluginState, PluginStatus, PluginType,
};
use std::path::PathBuf;

mod plugin_info_tests {
    use super::*;

    #[test]
    fn test_plugin_info_new() {
        let info = PluginInfo::new("test-plugin", "1.0.0");

        assert_eq!(info.name, "test-plugin");
        assert_eq!(info.version, "1.0.0");
        assert!(info.author.is_empty());
        assert!(info.description.is_empty());
        assert!(info.capabilities.is_empty());
    }

    #[test]
    fn test_plugin_info_builder() {
        let info = PluginInfo::new("my-plugin", "2.1.0")
            .with_author("John Doe")
            .with_description("A test plugin")
            .with_capability(PluginCapability::Exporter)
            .with_capability(PluginCapability::Notifier);

        assert_eq!(info.name, "my-plugin");
        assert_eq!(info.version, "2.1.0");
        assert_eq!(info.author, "John Doe");
        assert_eq!(info.description, "A test plugin");
        assert_eq!(info.capabilities.len(), 2);
        assert!(info.capabilities.contains(&PluginCapability::Exporter));
        assert!(info.capabilities.contains(&PluginCapability::Notifier));
    }

    #[test]
    fn test_plugin_info_with_capabilities() {
        let capabilities = vec![
            PluginCapability::Agent,
            PluginCapability::View,
            PluginCapability::Hook,
        ];
        let info = PluginInfo::new("caps-plugin", "1.0.0").with_capabilities(capabilities.clone());

        assert_eq!(info.capabilities.len(), 3);
        assert_eq!(info.capabilities, capabilities);
    }

    #[test]
    fn test_plugin_info_serialization() {
        let info = PluginInfo::new("serialize-test", "3.0.0")
            .with_author("Test Author")
            .with_capability(PluginCapability::Custom);

        let json = serde_json::to_string(&info).unwrap();
        let deserialized: PluginInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(info.name, deserialized.name);
        assert_eq!(info.version, deserialized.version);
        assert_eq!(info.author, deserialized.author);
        assert_eq!(info.capabilities, deserialized.capabilities);
    }

    #[test]
    fn test_plugin_info_yaml_serialization() {
        let info =
            PluginInfo::new("yaml-test", "1.0.0").with_description("YAML serialization test");

        let yaml = serde_yaml::to_string(&info).unwrap();
        let deserialized: PluginInfo = serde_yaml::from_str(&yaml).unwrap();

        assert_eq!(info.name, deserialized.name);
        assert_eq!(info.description, deserialized.description);
    }
}

mod plugin_capability_tests {
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
    fn test_plugin_capability_serialization() {
        let capabilities = vec![
            PluginCapability::Agent,
            PluginCapability::Exporter,
            PluginCapability::Notifier,
            PluginCapability::View,
            PluginCapability::Hook,
            PluginCapability::Custom,
        ];

        for cap in capabilities {
            let json = serde_json::to_string(&cap).unwrap();
            let deserialized: PluginCapability = serde_json::from_str(&json).unwrap();
            assert_eq!(cap, deserialized);
        }
    }

    #[test]
    fn test_plugin_capability_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(PluginCapability::Agent);
        set.insert(PluginCapability::Exporter);
        set.insert(PluginCapability::Agent);

        assert_eq!(set.len(), 2);
    }
}

mod plugin_config_tests {
    use super::*;

    #[test]
    fn test_plugin_config_default() {
        let config = PluginConfig::default();

        assert!(config.enabled);
        assert_eq!(config.priority, 0);
        assert!(config.settings.is_object());
    }

    #[test]
    fn test_plugin_config_new() {
        let config = PluginConfig::new();
        assert!(config.enabled);
    }

    #[test]
    fn test_plugin_config_disabled() {
        let config = PluginConfig::disabled();
        assert!(!config.enabled);
    }

    #[test]
    fn test_plugin_config_with_setting() {
        let config = PluginConfig::new()
            .with_setting("api_key", "secret123")
            .with_setting("timeout", 30)
            .with_setting("enabled_features", vec!["a", "b", "c"]);

        assert_eq!(
            config.get_setting::<String>("api_key"),
            Some("secret123".to_string())
        );
        assert_eq!(config.get_setting::<i32>("timeout"), Some(30));

        let features: Option<Vec<String>> = config.get_setting("enabled_features");
        assert!(features.is_some());
        assert_eq!(features.unwrap().len(), 3);
    }

    #[test]
    fn test_plugin_config_get_missing_setting() {
        let config = PluginConfig::new();
        let missing: Option<String> = config.get_setting("nonexistent");
        assert!(missing.is_none());
    }

    #[test]
    fn test_plugin_config_serialization() {
        let config = PluginConfig::new()
            .with_setting("key", "value")
            .with_setting("number", 42);

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: PluginConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.enabled, deserialized.enabled);
        assert_eq!(
            config.get_setting::<String>("key"),
            deserialized.get_setting::<String>("key")
        );
    }
}

mod plugin_event_tests {
    use super::*;

    #[test]
    fn test_plugin_event_loaded() {
        let event = PluginEvent::loaded("my-plugin");

        assert_eq!(event.plugin_id(), "my-plugin");
        assert!(event.timestamp() <= Utc::now());
    }

    #[test]
    fn test_plugin_event_enabled() {
        let event = PluginEvent::enabled("enabled-plugin");
        assert_eq!(event.plugin_id(), "enabled-plugin");
    }

    #[test]
    fn test_plugin_event_disabled() {
        let event = PluginEvent::disabled("disabled-plugin");
        assert_eq!(event.plugin_id(), "disabled-plugin");
    }

    #[test]
    fn test_plugin_event_error() {
        let event = PluginEvent::error("error-plugin", "Something went wrong");
        assert_eq!(event.plugin_id(), "error-plugin");

        if let PluginEvent::Error { error, .. } = event {
            assert_eq!(error, "Something went wrong");
        } else {
            panic!("Expected Error event");
        }
    }

    #[test]
    fn test_plugin_event_serialization() {
        let events = vec![
            PluginEvent::loaded("p1"),
            PluginEvent::enabled("p2"),
            PluginEvent::disabled("p3"),
            PluginEvent::error("p4", "error msg"),
        ];

        for event in events {
            let json = serde_json::to_string(&event).unwrap();
            let deserialized: PluginEvent = serde_json::from_str(&json).unwrap();
            assert_eq!(event.plugin_id(), deserialized.plugin_id());
        }
    }
}

mod plugin_status_tests {
    use super::*;

    #[test]
    fn test_plugin_status_default() {
        assert_eq!(PluginStatus::default(), PluginStatus::Unloaded);
    }

    #[test]
    fn test_plugin_status_serialization() {
        let statuses = vec![
            PluginStatus::Loaded,
            PluginStatus::Enabled,
            PluginStatus::Disabled,
            PluginStatus::Error,
            PluginStatus::Unloaded,
        ];

        for status in statuses {
            let json = serde_json::to_string(&status).unwrap();
            let deserialized: PluginStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(status, deserialized);
        }
    }
}

mod plugin_state_tests {
    use super::*;

    #[test]
    fn test_plugin_state_new() {
        let info = PluginInfo::new("state-test", "1.0.0");
        let state = PluginState::new(info.clone());

        assert_eq!(state.status, PluginStatus::Unloaded);
        assert_eq!(state.info.name, "state-test");
        assert!(state.loaded_at.is_none());
        assert!(state.error.is_none());
        assert!(!state.is_active());
    }

    #[test]
    fn test_plugin_state_is_active() {
        let info = PluginInfo::new("active-test", "1.0.0");
        let mut state = PluginState::new(info);

        assert!(!state.is_active());

        state.status = PluginStatus::Enabled;
        assert!(state.is_active());

        state.status = PluginStatus::Disabled;
        assert!(!state.is_active());
    }

    #[test]
    fn test_plugin_state_serialization() {
        let info =
            PluginInfo::new("serialize-state", "2.0.0").with_capability(PluginCapability::Exporter);
        let mut state = PluginState::new(info);
        state.status = PluginStatus::Enabled;
        state.loaded_at = Some(Utc::now());

        let json = serde_json::to_string(&state).unwrap();
        let deserialized: PluginState = serde_json::from_str(&json).unwrap();

        assert_eq!(state.status, deserialized.status);
        assert_eq!(state.info.name, deserialized.info.name);
    }
}

mod plugin_dependency_tests {
    use super::*;

    #[test]
    fn test_plugin_dependency_required() {
        let dep = PluginDependency::required("base-plugin", ">=1.0.0");

        assert_eq!(dep.name, "base-plugin");
        assert_eq!(dep.version_requirement, ">=1.0.0");
        assert!(!dep.optional);
    }

    #[test]
    fn test_plugin_dependency_optional() {
        let dep = PluginDependency::optional("optional-feature", "^2.0.0");

        assert_eq!(dep.name, "optional-feature");
        assert_eq!(dep.version_requirement, "^2.0.0");
        assert!(dep.optional);
    }

    #[test]
    fn test_plugin_dependency_serialization() {
        let dep = PluginDependency::required("dep-name", "1.0.0");

        let json = serde_json::to_string(&dep).unwrap();
        let deserialized: PluginDependency = serde_json::from_str(&json).unwrap();

        assert_eq!(dep.name, deserialized.name);
        assert_eq!(dep.version_requirement, deserialized.version_requirement);
        assert_eq!(dep.optional, deserialized.optional);
    }
}

mod plugin_type_tests {
    use super::*;

    #[test]
    fn test_plugin_type_default() {
        assert_eq!(PluginType::default(), PluginType::Native);
    }

    #[test]
    fn test_plugin_type_serialization() {
        let types = vec![PluginType::Native, PluginType::Wasm, PluginType::Script];

        for plugin_type in types {
            let json = serde_json::to_string(&plugin_type).unwrap();
            let deserialized: PluginType = serde_json::from_str(&json).unwrap();
            assert_eq!(plugin_type, deserialized);
        }
    }
}

mod plugin_context_tests {
    use super::*;

    #[test]
    fn test_plugin_context_new() {
        let ctx = PluginContext::new("my-plugin", "/data/plugins/my-plugin");

        assert_eq!(ctx.plugin_id, "my-plugin");
        assert_eq!(ctx.data_dir, PathBuf::from("/data/plugins/my-plugin"));
        assert!(ctx.metadata.is_empty());
    }

    #[test]
    fn test_plugin_context_with_config() {
        let config = PluginConfig::new().with_setting("key", "value");
        let ctx = PluginContext::new("test", "/tmp").with_config(config);

        assert_eq!(
            ctx.config.get_setting::<String>("key"),
            Some("value".to_string())
        );
    }

    #[test]
    fn test_plugin_context_metadata() {
        let mut ctx = PluginContext::new("meta-test", "/tmp");

        ctx.set_metadata("string_value", "hello");
        ctx.set_metadata("number_value", 42);
        ctx.set_metadata("bool_value", true);
        ctx.set_metadata("array_value", vec![1, 2, 3]);

        assert_eq!(
            ctx.get_metadata::<String>("string_value"),
            Some("hello".to_string())
        );
        assert_eq!(ctx.get_metadata::<i32>("number_value"), Some(42));
        assert_eq!(ctx.get_metadata::<bool>("bool_value"), Some(true));

        let array: Option<Vec<i32>> = ctx.get_metadata("array_value");
        assert!(array.is_some());
        assert_eq!(array.unwrap(), vec![1, 2, 3]);
    }

    #[test]
    fn test_plugin_context_missing_metadata() {
        let ctx = PluginContext::new("empty", "/tmp");
        let missing: Option<String> = ctx.get_metadata("nonexistent");
        assert!(missing.is_none());
    }
}

mod plugin_permission_tests {
    use super::*;

    #[test]
    fn test_plugin_permission_creation() {
        let permission = PluginPermission {
            name: "file_read".to_string(),
            description: "Read files from disk".to_string(),
            required: true,
        };

        assert_eq!(permission.name, "file_read");
        assert_eq!(permission.description, "Read files from disk");
        assert!(permission.required);
    }

    #[test]
    fn test_plugin_permission_serialization() {
        let permission = PluginPermission {
            name: "network".to_string(),
            description: "Make network requests".to_string(),
            required: false,
        };

        let json = serde_json::to_string(&permission).unwrap();
        let deserialized: PluginPermission = serde_json::from_str(&json).unwrap();

        assert_eq!(permission.name, deserialized.name);
        assert_eq!(permission.description, deserialized.description);
        assert_eq!(permission.required, deserialized.required);
    }
}

mod plugin_lifecycle_tests {
    use super::*;

    #[test]
    fn test_plugin_full_lifecycle() {
        let info = PluginInfo::new("lifecycle-test", "1.0.0")
            .with_author("Test Author")
            .with_description("Testing plugin lifecycle")
            .with_capability(PluginCapability::Exporter);

        let mut state = PluginState::new(info);
        assert_eq!(state.status, PluginStatus::Unloaded);
        assert!(!state.is_active());

        state.status = PluginStatus::Loaded;
        state.loaded_at = Some(Utc::now());
        assert_eq!(state.status, PluginStatus::Loaded);
        assert!(state.loaded_at.is_some());

        state.status = PluginStatus::Enabled;
        assert!(state.is_active());

        state.status = PluginStatus::Disabled;
        assert!(!state.is_active());

        state.status = PluginStatus::Error;
        state.error = Some("Plugin crashed".to_string());
        assert!(!state.is_active());
        assert!(state.error.is_some());

        state.status = PluginStatus::Unloaded;
        state.loaded_at = None;
        assert_eq!(state.status, PluginStatus::Unloaded);
    }

    #[test]
    fn test_plugin_config_changes() {
        let mut config = PluginConfig::new();

        config = config.with_setting("initial", "value");
        assert_eq!(
            config.get_setting::<String>("initial"),
            Some("value".to_string())
        );

        config = config.with_setting("initial", "updated");
        assert_eq!(
            config.get_setting::<String>("initial"),
            Some("updated".to_string())
        );

        config = config.with_setting("new_key", 123);
        assert_eq!(config.get_setting::<i32>("new_key"), Some(123));
    }
}

mod builtin_plugin_tests {
    use rimuru_core::plugins::{
        create_builtin_exporter, create_builtin_notifier, is_builtin_plugin, list_builtin_plugins,
    };

    #[test]
    fn test_is_builtin_plugin() {
        assert!(is_builtin_plugin("csv-exporter"));
        assert!(is_builtin_plugin("json-exporter"));
        assert!(is_builtin_plugin("slack-notifier"));
        assert!(is_builtin_plugin("discord-notifier"));
        assert!(is_builtin_plugin("webhook-notifier"));

        assert!(!is_builtin_plugin("custom-plugin"));
        assert!(!is_builtin_plugin("my-exporter"));
    }

    #[test]
    fn test_list_builtin_plugins() {
        let plugins = list_builtin_plugins();

        assert!(!plugins.is_empty());
        assert!(plugins.iter().any(|p| p.name == "csv-exporter"));
        assert!(plugins.iter().any(|p| p.name == "json-exporter"));
        assert!(plugins.iter().any(|p| p.name == "slack-notifier"));
        assert!(plugins.iter().any(|p| p.name == "discord-notifier"));
        assert!(plugins.iter().any(|p| p.name == "webhook-notifier"));

        for plugin in &plugins {
            assert!(!plugin.version.is_empty());
            assert!(!plugin.capabilities.is_empty());
        }
    }

    #[test]
    fn test_create_builtin_exporter() {
        let csv_exporter = create_builtin_exporter("csv-exporter");
        assert!(csv_exporter.is_some());

        let json_exporter = create_builtin_exporter("json-exporter");
        assert!(json_exporter.is_some());

        let invalid = create_builtin_exporter("nonexistent");
        assert!(invalid.is_none());
    }

    #[test]
    fn test_create_builtin_notifier() {
        let slack = create_builtin_notifier("slack-notifier");
        assert!(slack.is_some());

        let discord = create_builtin_notifier("discord-notifier");
        assert!(discord.is_some());

        let webhook = create_builtin_notifier("webhook-notifier");
        assert!(webhook.is_some());

        let invalid = create_builtin_notifier("nonexistent");
        assert!(invalid.is_none());
    }
}
