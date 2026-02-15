//! JSON Exporter Plugin
//!
//! Exports sessions and costs to JSON format.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::RimuruResult;
use crate::models::{CostRecord, Session};
use crate::plugins::traits::{ExportOptions, ExporterPlugin, Plugin};
use crate::plugins::types::{PluginCapability, PluginConfig, PluginContext, PluginInfo};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonExporterConfig {
    pub pretty_print: bool,
    pub indent_size: usize,
    pub include_null_fields: bool,
    pub wrap_in_object: bool,
    pub root_key_sessions: String,
    pub root_key_costs: String,
}

impl Default for JsonExporterConfig {
    fn default() -> Self {
        Self {
            pretty_print: true,
            indent_size: 2,
            include_null_fields: true,
            wrap_in_object: true,
            root_key_sessions: "sessions".to_string(),
            root_key_costs: "costs".to_string(),
        }
    }
}

pub struct JsonExporterPlugin {
    info: PluginInfo,
    config: JsonExporterConfig,
    initialized: bool,
}

impl JsonExporterPlugin {
    pub fn new() -> Self {
        Self {
            info: PluginInfo::new("json-exporter", "1.0.0")
                .with_author("Rimuru Team")
                .with_description("Export sessions and costs to JSON format")
                .with_capability(PluginCapability::Exporter),
            config: JsonExporterConfig::default(),
            initialized: false,
        }
    }

    fn serialize_with_options<T: Serialize>(
        &self,
        data: &T,
        options: &ExportOptions,
    ) -> RimuruResult<Vec<u8>> {
        let use_pretty = options.pretty || self.config.pretty_print;

        let result = if use_pretty {
            let indent = " ".repeat(self.config.indent_size).into_bytes();
            let formatter = serde_json::ser::PrettyFormatter::with_indent(&indent);
            let mut writer = Vec::new();
            let mut serializer = serde_json::Serializer::with_formatter(&mut writer, formatter);
            data.serialize(&mut serializer)?;
            writer
        } else {
            serde_json::to_vec(data)?
        };

        Ok(result)
    }
}

impl Default for JsonExporterPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for JsonExporterPlugin {
    fn info(&self) -> &PluginInfo {
        &self.info
    }

    async fn init(&mut self, _ctx: &PluginContext) -> RimuruResult<()> {
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

    fn configure(&mut self, config: PluginConfig) -> RimuruResult<()> {
        if let Some(pretty_print) = config.get_setting::<bool>("pretty_print") {
            self.config.pretty_print = pretty_print;
        }
        if let Some(indent_size) = config.get_setting::<usize>("indent_size") {
            self.config.indent_size = indent_size.min(8);
        }
        if let Some(include_null_fields) = config.get_setting::<bool>("include_null_fields") {
            self.config.include_null_fields = include_null_fields;
        }
        if let Some(wrap_in_object) = config.get_setting::<bool>("wrap_in_object") {
            self.config.wrap_in_object = wrap_in_object;
        }
        if let Some(root_key_sessions) = config.get_setting::<String>("root_key_sessions") {
            self.config.root_key_sessions = root_key_sessions;
        }
        if let Some(root_key_costs) = config.get_setting::<String>("root_key_costs") {
            self.config.root_key_costs = root_key_costs;
        }
        Ok(())
    }

    fn config_schema(&self) -> Option<serde_json::Value> {
        Some(serde_json::json!({
            "type": "object",
            "properties": {
                "pretty_print": {
                    "type": "boolean",
                    "description": "Format JSON with indentation and newlines",
                    "default": true
                },
                "indent_size": {
                    "type": "integer",
                    "description": "Number of spaces for indentation",
                    "default": 2,
                    "minimum": 1,
                    "maximum": 8
                },
                "include_null_fields": {
                    "type": "boolean",
                    "description": "Include fields with null values",
                    "default": true
                },
                "wrap_in_object": {
                    "type": "boolean",
                    "description": "Wrap array in an object with root key",
                    "default": true
                },
                "root_key_sessions": {
                    "type": "string",
                    "description": "Root key for sessions array when wrap_in_object is true",
                    "default": "sessions"
                },
                "root_key_costs": {
                    "type": "string",
                    "description": "Root key for costs array when wrap_in_object is true",
                    "default": "costs"
                }
            }
        }))
    }
}

#[derive(Serialize)]
struct SessionExport<'a> {
    id: String,
    agent_id: String,
    status: String,
    started_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    ended_at: Option<String>,
    metadata: &'a serde_json::Value,
}

#[derive(Serialize)]
struct CostExport {
    id: String,
    session_id: String,
    agent_id: String,
    model_name: String,
    input_tokens: i64,
    output_tokens: i64,
    total_tokens: i64,
    cost_usd: f64,
    recorded_at: String,
}

impl<'a> From<&'a Session> for SessionExport<'a> {
    fn from(session: &'a Session) -> Self {
        Self {
            id: session.id.to_string(),
            agent_id: session.agent_id.to_string(),
            status: session.status.to_string(),
            started_at: session.started_at.to_rfc3339(),
            ended_at: session.ended_at.map(|dt| dt.to_rfc3339()),
            metadata: &session.metadata,
        }
    }
}

impl From<&CostRecord> for CostExport {
    fn from(cost: &CostRecord) -> Self {
        Self {
            id: cost.id.to_string(),
            session_id: cost.session_id.to_string(),
            agent_id: cost.agent_id.to_string(),
            model_name: cost.model_name.clone(),
            input_tokens: cost.input_tokens,
            output_tokens: cost.output_tokens,
            total_tokens: cost.total_tokens(),
            cost_usd: cost.cost_usd,
            recorded_at: cost.recorded_at.to_rfc3339(),
        }
    }
}

#[async_trait]
impl ExporterPlugin for JsonExporterPlugin {
    fn format(&self) -> &str {
        "json"
    }

    fn file_extension(&self) -> &str {
        "json"
    }

    async fn export_sessions(
        &self,
        sessions: &[Session],
        options: ExportOptions,
    ) -> RimuruResult<Vec<u8>> {
        let exports: Vec<SessionExport> = sessions.iter().map(SessionExport::from).collect();

        if self.config.wrap_in_object {
            let mut wrapper = serde_json::Map::new();
            wrapper.insert(
                self.config.root_key_sessions.clone(),
                serde_json::to_value(&exports)?,
            );
            wrapper.insert("count".to_string(), serde_json::json!(exports.len()));
            wrapper.insert(
                "exported_at".to_string(),
                serde_json::json!(chrono::Utc::now().to_rfc3339()),
            );
            self.serialize_with_options(&wrapper, &options)
        } else {
            self.serialize_with_options(&exports, &options)
        }
    }

    async fn export_costs(
        &self,
        costs: &[CostRecord],
        options: ExportOptions,
    ) -> RimuruResult<Vec<u8>> {
        let exports: Vec<CostExport> = costs.iter().map(CostExport::from).collect();

        if self.config.wrap_in_object {
            let total_cost: f64 = costs.iter().map(|c| c.cost_usd).sum();
            let total_tokens: i64 = costs.iter().map(|c| c.total_tokens()).sum();

            let mut wrapper = serde_json::Map::new();
            wrapper.insert(
                self.config.root_key_costs.clone(),
                serde_json::to_value(&exports)?,
            );
            wrapper.insert("count".to_string(), serde_json::json!(exports.len()));
            wrapper.insert("total_cost_usd".to_string(), serde_json::json!(total_cost));
            wrapper.insert("total_tokens".to_string(), serde_json::json!(total_tokens));
            wrapper.insert(
                "exported_at".to_string(),
                serde_json::json!(chrono::Utc::now().to_rfc3339()),
            );
            self.serialize_with_options(&wrapper, &options)
        } else {
            self.serialize_with_options(&exports, &options)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::SessionStatus;
    use chrono::Utc;
    use uuid::Uuid;

    fn create_test_session() -> Session {
        Session {
            id: Uuid::new_v4(),
            agent_id: Uuid::new_v4(),
            status: SessionStatus::Completed,
            started_at: Utc::now(),
            ended_at: Some(Utc::now()),
            metadata: serde_json::json!({"project": "test"}),
        }
    }

    fn create_test_cost() -> CostRecord {
        CostRecord {
            id: Uuid::new_v4(),
            session_id: Uuid::new_v4(),
            agent_id: Uuid::new_v4(),
            model_name: "gpt-4".to_string(),
            input_tokens: 1000,
            output_tokens: 500,
            cost_usd: 0.05,
            recorded_at: Utc::now(),
        }
    }

    #[test]
    fn test_json_exporter_new() {
        let plugin = JsonExporterPlugin::new();
        assert_eq!(plugin.info().name, "json-exporter");
        assert_eq!(plugin.info().version, "1.0.0");
        assert!(!plugin.is_initialized());
    }

    #[tokio::test]
    async fn test_export_sessions_wrapped() {
        let plugin = JsonExporterPlugin::new();
        let sessions = vec![create_test_session()];
        let options = ExportOptions::default();

        let result = plugin.export_sessions(&sessions, options).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&result).unwrap();

        assert!(json["sessions"].is_array());
        assert_eq!(json["count"], 1);
        assert!(json["exported_at"].is_string());
    }

    #[tokio::test]
    async fn test_export_sessions_unwrapped() {
        let mut plugin = JsonExporterPlugin::new();
        plugin.config.wrap_in_object = false;

        let sessions = vec![create_test_session()];
        let options = ExportOptions::default();

        let result = plugin.export_sessions(&sessions, options).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&result).unwrap();

        assert!(json.is_array());
        assert_eq!(json.as_array().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_export_costs_wrapped() {
        let plugin = JsonExporterPlugin::new();
        let costs = vec![create_test_cost(), create_test_cost()];
        let options = ExportOptions::default();

        let result = plugin.export_costs(&costs, options).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&result).unwrap();

        assert!(json["costs"].is_array());
        assert_eq!(json["count"], 2);
        assert!(json["total_cost_usd"].is_number());
        assert!(json["total_tokens"].is_number());
    }

    #[tokio::test]
    async fn test_export_costs_totals() {
        let plugin = JsonExporterPlugin::new();
        let costs = vec![create_test_cost(), create_test_cost()];
        let options = ExportOptions::default();

        let result = plugin.export_costs(&costs, options).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&result).unwrap();

        let total_cost = json["total_cost_usd"].as_f64().unwrap();
        assert!((total_cost - 0.10).abs() < 0.001);

        let total_tokens = json["total_tokens"].as_i64().unwrap();
        assert_eq!(total_tokens, 3000);
    }

    #[tokio::test]
    async fn test_export_empty() {
        let plugin = JsonExporterPlugin::new();
        let options = ExportOptions::default();

        let result = plugin.export_sessions(&[], options).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&result).unwrap();

        assert_eq!(json["count"], 0);
        assert!(json["sessions"].as_array().unwrap().is_empty());
    }

    #[test]
    fn test_config_schema() {
        let plugin = JsonExporterPlugin::new();
        let schema = plugin.config_schema().unwrap();

        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["pretty_print"].is_object());
        assert!(schema["properties"]["indent_size"].is_object());
        assert!(schema["properties"]["wrap_in_object"].is_object());
    }

    #[test]
    fn test_configure() {
        let mut plugin = JsonExporterPlugin::new();
        let config = PluginConfig::new()
            .with_setting("pretty_print", false)
            .with_setting("indent_size", 4)
            .with_setting("wrap_in_object", false)
            .with_setting("root_key_sessions", "data");

        plugin.configure(config).unwrap();

        assert!(!plugin.config.pretty_print);
        assert_eq!(plugin.config.indent_size, 4);
        assert!(!plugin.config.wrap_in_object);
        assert_eq!(plugin.config.root_key_sessions, "data");
    }

    #[tokio::test]
    async fn test_init_and_shutdown() {
        let mut plugin = JsonExporterPlugin::new();
        let ctx = PluginContext::new("json-exporter", "/tmp");

        assert!(!plugin.is_initialized());

        plugin.init(&ctx).await.unwrap();
        assert!(plugin.is_initialized());

        plugin.shutdown().await.unwrap();
        assert!(!plugin.is_initialized());
    }

    #[test]
    fn test_format_and_extension() {
        let plugin = JsonExporterPlugin::new();
        assert_eq!(plugin.format(), "json");
        assert_eq!(plugin.file_extension(), "json");
    }

    #[tokio::test]
    async fn test_pretty_vs_compact() {
        let mut plugin = JsonExporterPlugin::new();
        let sessions = vec![create_test_session()];

        plugin.config.pretty_print = true;
        let pretty_result = plugin
            .export_sessions(&sessions, ExportOptions::default())
            .await
            .unwrap();
        let pretty_str = String::from_utf8(pretty_result).unwrap();

        plugin.config.pretty_print = false;
        let compact_result = plugin
            .export_sessions(
                &sessions,
                ExportOptions {
                    pretty: false,
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        let compact_str = String::from_utf8(compact_result).unwrap();

        assert!(pretty_str.contains('\n'));
        assert!(!compact_str.contains('\n'));
    }

    #[test]
    fn test_session_export_conversion() {
        let session = create_test_session();
        let export = SessionExport::from(&session);

        assert_eq!(export.id, session.id.to_string());
        assert_eq!(export.status, "completed");
    }

    #[test]
    fn test_cost_export_conversion() {
        let cost = create_test_cost();
        let export = CostExport::from(&cost);

        assert_eq!(export.id, cost.id.to_string());
        assert_eq!(export.total_tokens, 1500);
    }
}
