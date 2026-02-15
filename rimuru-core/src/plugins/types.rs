use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub capabilities: Vec<PluginCapability>,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub license: Option<String>,
}

impl PluginInfo {
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            author: String::new(),
            description: String::new(),
            capabilities: Vec::new(),
            homepage: None,
            repository: None,
            license: None,
        }
    }

    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = author.into();
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    pub fn with_capability(mut self, capability: PluginCapability) -> Self {
        self.capabilities.push(capability);
        self
    }

    pub fn with_capabilities(mut self, capabilities: Vec<PluginCapability>) -> Self {
        self.capabilities = capabilities;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginCapability {
    Agent,
    Exporter,
    Notifier,
    View,
    Hook,
    Custom,
}

impl std::fmt::Display for PluginCapability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginCapability::Agent => write!(f, "agent"),
            PluginCapability::Exporter => write!(f, "exporter"),
            PluginCapability::Notifier => write!(f, "notifier"),
            PluginCapability::View => write!(f, "view"),
            PluginCapability::Hook => write!(f, "hook"),
            PluginCapability::Custom => write!(f, "custom"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub enabled: bool,
    pub settings: serde_json::Value,
    #[serde(default)]
    pub priority: i32,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            settings: serde_json::Value::Object(serde_json::Map::new()),
            priority: 0,
        }
    }
}

impl PluginConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn disabled() -> Self {
        Self {
            enabled: false,
            ..Default::default()
        }
    }

    pub fn with_setting<V: Serialize>(mut self, key: &str, value: V) -> Self {
        if let serde_json::Value::Object(ref mut map) = self.settings {
            if let Ok(v) = serde_json::to_value(value) {
                map.insert(key.to_string(), v);
            }
        }
        self
    }

    pub fn get_setting<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        if let serde_json::Value::Object(ref map) = self.settings {
            map.get(key)
                .and_then(|v| serde_json::from_value(v.clone()).ok())
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum PluginEvent {
    Loaded {
        plugin_id: String,
        timestamp: DateTime<Utc>,
    },
    Enabled {
        plugin_id: String,
        timestamp: DateTime<Utc>,
    },
    Disabled {
        plugin_id: String,
        timestamp: DateTime<Utc>,
    },
    Error {
        plugin_id: String,
        error: String,
        timestamp: DateTime<Utc>,
    },
    ConfigChanged {
        plugin_id: String,
        timestamp: DateTime<Utc>,
    },
    Unloaded {
        plugin_id: String,
        timestamp: DateTime<Utc>,
    },
}

impl PluginEvent {
    pub fn loaded(plugin_id: impl Into<String>) -> Self {
        Self::Loaded {
            plugin_id: plugin_id.into(),
            timestamp: Utc::now(),
        }
    }

    pub fn enabled(plugin_id: impl Into<String>) -> Self {
        Self::Enabled {
            plugin_id: plugin_id.into(),
            timestamp: Utc::now(),
        }
    }

    pub fn disabled(plugin_id: impl Into<String>) -> Self {
        Self::Disabled {
            plugin_id: plugin_id.into(),
            timestamp: Utc::now(),
        }
    }

    pub fn error(plugin_id: impl Into<String>, error: impl Into<String>) -> Self {
        Self::Error {
            plugin_id: plugin_id.into(),
            error: error.into(),
            timestamp: Utc::now(),
        }
    }

    pub fn plugin_id(&self) -> &str {
        match self {
            Self::Loaded { plugin_id, .. }
            | Self::Enabled { plugin_id, .. }
            | Self::Disabled { plugin_id, .. }
            | Self::Error { plugin_id, .. }
            | Self::ConfigChanged { plugin_id, .. }
            | Self::Unloaded { plugin_id, .. } => plugin_id,
        }
    }

    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            Self::Loaded { timestamp, .. }
            | Self::Enabled { timestamp, .. }
            | Self::Disabled { timestamp, .. }
            | Self::Error { timestamp, .. }
            | Self::ConfigChanged { timestamp, .. }
            | Self::Unloaded { timestamp, .. } => *timestamp,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum PluginStatus {
    Loaded,
    Enabled,
    Disabled,
    Error,
    #[default]
    Unloaded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginState {
    pub status: PluginStatus,
    pub info: PluginInfo,
    pub config: PluginConfig,
    pub loaded_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
}

impl PluginState {
    pub fn new(info: PluginInfo) -> Self {
        Self {
            status: PluginStatus::Unloaded,
            info,
            config: PluginConfig::default(),
            loaded_at: None,
            error: None,
        }
    }

    pub fn is_active(&self) -> bool {
        self.status == PluginStatus::Enabled
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDependency {
    pub name: String,
    pub version_requirement: String,
    pub optional: bool,
}

impl PluginDependency {
    pub fn required(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version_requirement: version.into(),
            optional: false,
        }
    }

    pub fn optional(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version_requirement: version.into(),
            optional: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginPermission {
    pub name: String,
    pub description: String,
    pub required: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum PluginType {
    #[default]
    Native,
    Wasm,
    Script,
}

#[derive(Debug, Clone)]
pub struct PluginContext {
    pub plugin_id: String,
    pub data_dir: std::path::PathBuf,
    pub config: PluginConfig,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl PluginContext {
    pub fn new(plugin_id: impl Into<String>, data_dir: impl Into<std::path::PathBuf>) -> Self {
        Self {
            plugin_id: plugin_id.into(),
            data_dir: data_dir.into(),
            config: PluginConfig::default(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_config(mut self, config: PluginConfig) -> Self {
        self.config = config;
        self
    }

    pub fn set_metadata<V: Serialize>(&mut self, key: &str, value: V) {
        if let Ok(v) = serde_json::to_value(value) {
            self.metadata.insert(key.to_string(), v);
        }
    }

    pub fn get_metadata<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        self.metadata
            .get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }
}
