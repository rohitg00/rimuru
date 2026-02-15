use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::error::{RimuruError, RimuruResult};

use super::types::{PluginCapability, PluginDependency, PluginPermission, PluginType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub plugin: PluginMetadata,
    #[serde(default)]
    pub capabilities: CapabilitiesSection,
    #[serde(default)]
    pub dependencies: Vec<PluginDependency>,
    #[serde(default)]
    pub permissions: Vec<PluginPermission>,
    #[serde(default)]
    pub config: ConfigSection,
    #[serde(default)]
    pub hooks: Vec<HookRegistration>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub description: String,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub license: Option<String>,
    #[serde(default)]
    pub plugin_type: PluginType,
    pub entry_point: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CapabilitiesSection {
    #[serde(default)]
    pub agent: Option<AgentCapability>,
    #[serde(default)]
    pub exporter: Option<ExporterCapability>,
    #[serde(default)]
    pub notifier: Option<NotifierCapability>,
    #[serde(default)]
    pub view: Option<ViewCapability>,
}

impl CapabilitiesSection {
    pub fn to_capabilities(&self) -> Vec<PluginCapability> {
        let mut caps = Vec::new();
        if self.agent.is_some() {
            caps.push(PluginCapability::Agent);
        }
        if self.exporter.is_some() {
            caps.push(PluginCapability::Exporter);
        }
        if self.notifier.is_some() {
            caps.push(PluginCapability::Notifier);
        }
        if self.view.is_some() {
            caps.push(PluginCapability::View);
        }
        caps
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCapability {
    pub agent_type: String,
    #[serde(default)]
    pub supports_sessions: bool,
    #[serde(default)]
    pub supports_costs: bool,
    #[serde(default)]
    pub supports_streaming: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExporterCapability {
    pub format: String,
    pub file_extension: String,
    #[serde(default)]
    pub supports_sessions: bool,
    #[serde(default)]
    pub supports_costs: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotifierCapability {
    pub notification_type: String,
    #[serde(default)]
    pub supports_batch: bool,
    #[serde(default)]
    pub rate_limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewCapability {
    pub view_name: String,
    pub view_title: String,
    pub keybind: Option<char>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConfigSection {
    #[serde(default)]
    pub schema: Option<serde_json::Value>,
    #[serde(default)]
    pub defaults: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookRegistration {
    pub hook: String,
    pub handler: String,
    #[serde(default)]
    pub priority: i32,
}

impl PluginManifest {
    pub const FILENAME: &'static str = "rimuru-plugin.toml";

    pub async fn load_from_file(path: &Path) -> RimuruResult<Self> {
        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| RimuruError::plugin(format!("Failed to read manifest file: {}", e)))?;
        Self::parse(&content)
    }

    pub fn parse(content: &str) -> RimuruResult<Self> {
        toml::from_str(content)
            .map_err(|e| RimuruError::plugin(format!("Failed to parse manifest: {}", e)))
    }

    pub fn validate(&self) -> RimuruResult<()> {
        if self.plugin.name.is_empty() {
            return Err(RimuruError::plugin("Plugin name cannot be empty"));
        }

        if self.plugin.version.is_empty() {
            return Err(RimuruError::plugin("Plugin version cannot be empty"));
        }

        if !Self::is_valid_version(&self.plugin.version) {
            return Err(RimuruError::plugin(format!(
                "Invalid version format: {}. Expected semver (e.g., 1.0.0)",
                self.plugin.version
            )));
        }

        if !Self::is_valid_name(&self.plugin.name) {
            return Err(RimuruError::plugin(format!(
                "Invalid plugin name: {}. Names must be alphanumeric with hyphens/underscores",
                self.plugin.name
            )));
        }

        Ok(())
    }

    fn is_valid_version(version: &str) -> bool {
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() < 2 || parts.len() > 3 {
            return false;
        }
        parts.iter().all(|p| p.parse::<u32>().is_ok())
    }

    fn is_valid_name(name: &str) -> bool {
        !name.is_empty()
            && name
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    }

    pub fn plugin_id(&self) -> String {
        format!("{}@{}", self.plugin.name, self.plugin.version)
    }

    pub fn capabilities(&self) -> Vec<PluginCapability> {
        self.capabilities.to_capabilities()
    }

    pub fn to_toml(&self) -> RimuruResult<String> {
        toml::to_string_pretty(self)
            .map_err(|e| RimuruError::plugin(format!("Failed to serialize manifest: {}", e)))
    }
}

pub fn create_example_manifest() -> PluginManifest {
    PluginManifest {
        plugin: PluginMetadata {
            name: "example-plugin".to_string(),
            version: "0.1.0".to_string(),
            author: "Your Name".to_string(),
            description: "An example Rimuru plugin".to_string(),
            homepage: Some("https://example.com".to_string()),
            repository: Some("https://github.com/example/plugin".to_string()),
            license: Some("MIT".to_string()),
            plugin_type: PluginType::Native,
            entry_point: Some("libexample_plugin.so".to_string()),
        },
        capabilities: CapabilitiesSection {
            agent: None,
            exporter: Some(ExporterCapability {
                format: "csv".to_string(),
                file_extension: "csv".to_string(),
                supports_sessions: true,
                supports_costs: true,
            }),
            notifier: None,
            view: None,
        },
        dependencies: vec![],
        permissions: vec![PluginPermission {
            name: "filesystem".to_string(),
            description: "Access to write export files".to_string(),
            required: true,
        }],
        config: ConfigSection {
            schema: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "delimiter": {
                        "type": "string",
                        "default": ","
                    },
                    "include_headers": {
                        "type": "boolean",
                        "default": true
                    }
                }
            })),
            defaults: HashMap::from([
                ("delimiter".to_string(), serde_json::json!(",")),
                ("include_headers".to_string(), serde_json::json!(true)),
            ]),
        },
        hooks: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_validation() {
        let manifest = create_example_manifest();
        assert!(manifest.validate().is_ok());
    }

    #[test]
    fn test_invalid_version() {
        let mut manifest = create_example_manifest();
        manifest.plugin.version = "invalid".to_string();
        assert!(manifest.validate().is_err());
    }

    #[test]
    fn test_invalid_name() {
        let mut manifest = create_example_manifest();
        manifest.plugin.name = "invalid name with spaces".to_string();
        assert!(manifest.validate().is_err());
    }

    #[test]
    fn test_parse_manifest() {
        let toml_content = r#"
[plugin]
name = "test-plugin"
version = "1.0.0"
author = "Test Author"
description = "A test plugin"

[capabilities.exporter]
format = "json"
file_extension = "json"
supports_sessions = true
supports_costs = true
"#;

        let manifest = PluginManifest::parse(toml_content).unwrap();
        assert_eq!(manifest.plugin.name, "test-plugin");
        assert_eq!(manifest.plugin.version, "1.0.0");
        assert!(manifest.capabilities.exporter.is_some());
    }
}
