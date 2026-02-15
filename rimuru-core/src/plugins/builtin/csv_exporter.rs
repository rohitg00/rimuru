//! CSV Exporter Plugin
//!
//! Exports sessions and costs to CSV format.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::io::Write;

use crate::error::RimuruResult;
use crate::models::{CostRecord, Session};
use crate::plugins::traits::{ExportOptions, ExporterPlugin, Plugin};
use crate::plugins::types::{PluginCapability, PluginConfig, PluginContext, PluginInfo};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsvExporterConfig {
    pub delimiter: char,
    pub quote_char: char,
    pub escape_char: char,
    pub include_bom: bool,
    pub line_ending: LineEnding,
}

impl Default for CsvExporterConfig {
    fn default() -> Self {
        Self {
            delimiter: ',',
            quote_char: '"',
            escape_char: '"',
            include_bom: false,
            line_ending: LineEnding::Lf,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LineEnding {
    Lf,
    Crlf,
}

impl LineEnding {
    fn as_str(&self) -> &'static str {
        match self {
            LineEnding::Lf => "\n",
            LineEnding::Crlf => "\r\n",
        }
    }
}

pub struct CsvExporterPlugin {
    info: PluginInfo,
    config: CsvExporterConfig,
    initialized: bool,
}

impl CsvExporterPlugin {
    pub fn new() -> Self {
        Self {
            info: PluginInfo::new("csv-exporter", "1.0.0")
                .with_author("Rimuru Team")
                .with_description("Export sessions and costs to CSV format")
                .with_capability(PluginCapability::Exporter),
            config: CsvExporterConfig::default(),
            initialized: false,
        }
    }

    fn escape_field(&self, field: &str) -> String {
        let needs_quoting = field.contains(self.config.delimiter)
            || field.contains(self.config.quote_char)
            || field.contains('\n')
            || field.contains('\r');

        if needs_quoting {
            let escaped = field.replace(
                self.config.quote_char,
                &format!("{}{}", self.config.escape_char, self.config.quote_char),
            );
            format!(
                "{}{}{}",
                self.config.quote_char, escaped, self.config.quote_char
            )
        } else {
            field.to_string()
        }
    }

    fn format_datetime(
        &self,
        dt: &chrono::DateTime<chrono::Utc>,
        options: &ExportOptions,
    ) -> String {
        if let Some(ref format) = options.date_format {
            dt.format(format).to_string()
        } else {
            dt.to_rfc3339()
        }
    }

    fn write_row(&self, writer: &mut Vec<u8>, fields: &[String]) {
        let line = fields
            .iter()
            .map(|f| self.escape_field(f))
            .collect::<Vec<_>>()
            .join(&self.config.delimiter.to_string());
        write!(writer, "{}{}", line, self.config.line_ending.as_str()).unwrap();
    }

    fn session_headers() -> Vec<&'static str> {
        vec![
            "id",
            "agent_id",
            "status",
            "started_at",
            "ended_at",
            "metadata",
        ]
    }

    fn cost_headers() -> Vec<&'static str> {
        vec![
            "id",
            "session_id",
            "agent_id",
            "model_name",
            "input_tokens",
            "output_tokens",
            "cost_usd",
            "recorded_at",
        ]
    }
}

impl Default for CsvExporterPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for CsvExporterPlugin {
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
        if let Some(delimiter) = config.get_setting::<String>("delimiter") {
            if let Some(c) = delimiter.chars().next() {
                self.config.delimiter = c;
            }
        }
        if let Some(quote_char) = config.get_setting::<String>("quote_char") {
            if let Some(c) = quote_char.chars().next() {
                self.config.quote_char = c;
            }
        }
        if let Some(include_bom) = config.get_setting::<bool>("include_bom") {
            self.config.include_bom = include_bom;
        }
        if let Some(line_ending) = config.get_setting::<String>("line_ending") {
            self.config.line_ending = match line_ending.as_str() {
                "crlf" => LineEnding::Crlf,
                _ => LineEnding::Lf,
            };
        }
        Ok(())
    }

    fn config_schema(&self) -> Option<serde_json::Value> {
        Some(serde_json::json!({
            "type": "object",
            "properties": {
                "delimiter": {
                    "type": "string",
                    "description": "Field delimiter character",
                    "default": ",",
                    "maxLength": 1
                },
                "quote_char": {
                    "type": "string",
                    "description": "Quote character for escaping",
                    "default": "\"",
                    "maxLength": 1
                },
                "include_bom": {
                    "type": "boolean",
                    "description": "Include UTF-8 BOM at the start of the file",
                    "default": false
                },
                "line_ending": {
                    "type": "string",
                    "description": "Line ending style",
                    "enum": ["lf", "crlf"],
                    "default": "lf"
                }
            }
        }))
    }
}

#[async_trait]
impl ExporterPlugin for CsvExporterPlugin {
    fn format(&self) -> &str {
        "csv"
    }

    fn file_extension(&self) -> &str {
        "csv"
    }

    async fn export_sessions(
        &self,
        sessions: &[Session],
        options: ExportOptions,
    ) -> RimuruResult<Vec<u8>> {
        let mut output = Vec::new();

        if self.config.include_bom {
            output.extend_from_slice(&[0xEF, 0xBB, 0xBF]);
        }

        if options.include_headers {
            let headers: Vec<String> = Self::session_headers()
                .into_iter()
                .map(String::from)
                .collect();
            self.write_row(&mut output, &headers);
        }

        for session in sessions {
            let fields = vec![
                session.id.to_string(),
                session.agent_id.to_string(),
                session.status.to_string(),
                self.format_datetime(&session.started_at, &options),
                session
                    .ended_at
                    .map(|dt| self.format_datetime(&dt, &options))
                    .unwrap_or_default(),
                serde_json::to_string(&session.metadata).unwrap_or_default(),
            ];
            self.write_row(&mut output, &fields);
        }

        Ok(output)
    }

    async fn export_costs(
        &self,
        costs: &[CostRecord],
        options: ExportOptions,
    ) -> RimuruResult<Vec<u8>> {
        let mut output = Vec::new();

        if self.config.include_bom {
            output.extend_from_slice(&[0xEF, 0xBB, 0xBF]);
        }

        if options.include_headers {
            let headers: Vec<String> = Self::cost_headers().into_iter().map(String::from).collect();
            self.write_row(&mut output, &headers);
        }

        for cost in costs {
            let fields = vec![
                cost.id.to_string(),
                cost.session_id.to_string(),
                cost.agent_id.to_string(),
                cost.model_name.clone(),
                cost.input_tokens.to_string(),
                cost.output_tokens.to_string(),
                format!("{:.6}", cost.cost_usd),
                self.format_datetime(&cost.recorded_at, &options),
            ];
            self.write_row(&mut output, &fields);
        }

        Ok(output)
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
            metadata: serde_json::json!({"test": true}),
        }
    }

    fn create_test_cost() -> CostRecord {
        CostRecord {
            id: Uuid::new_v4(),
            session_id: Uuid::new_v4(),
            agent_id: Uuid::new_v4(),
            model_name: "claude-3-opus".to_string(),
            input_tokens: 1000,
            output_tokens: 500,
            cost_usd: 0.05,
            recorded_at: Utc::now(),
        }
    }

    #[test]
    fn test_csv_exporter_new() {
        let plugin = CsvExporterPlugin::new();
        assert_eq!(plugin.info().name, "csv-exporter");
        assert_eq!(plugin.info().version, "1.0.0");
        assert!(!plugin.is_initialized());
    }

    #[test]
    fn test_escape_field_simple() {
        let plugin = CsvExporterPlugin::new();
        assert_eq!(plugin.escape_field("simple"), "simple");
    }

    #[test]
    fn test_escape_field_with_delimiter() {
        let plugin = CsvExporterPlugin::new();
        assert_eq!(plugin.escape_field("hello,world"), "\"hello,world\"");
    }

    #[test]
    fn test_escape_field_with_quotes() {
        let plugin = CsvExporterPlugin::new();
        assert_eq!(
            plugin.escape_field("say \"hello\""),
            "\"say \"\"hello\"\"\""
        );
    }

    #[tokio::test]
    async fn test_export_sessions_with_headers() {
        let plugin = CsvExporterPlugin::new();
        let sessions = vec![create_test_session()];
        let options = ExportOptions {
            include_headers: true,
            ..Default::default()
        };

        let result = plugin.export_sessions(&sessions, options).await.unwrap();
        let content = String::from_utf8(result).unwrap();

        assert!(content.starts_with("id,agent_id,status,started_at,ended_at,metadata\n"));
        assert!(content.contains("completed"));
    }

    #[tokio::test]
    async fn test_export_sessions_without_headers() {
        let plugin = CsvExporterPlugin::new();
        let sessions = vec![create_test_session()];
        let options = ExportOptions {
            include_headers: false,
            ..Default::default()
        };

        let result = plugin.export_sessions(&sessions, options).await.unwrap();
        let content = String::from_utf8(result).unwrap();

        assert!(!content.contains("id,agent_id"));
        assert!(content.contains("completed"));
    }

    #[tokio::test]
    async fn test_export_costs_with_headers() {
        let plugin = CsvExporterPlugin::new();
        let costs = vec![create_test_cost()];
        let options = ExportOptions {
            include_headers: true,
            ..Default::default()
        };

        let result = plugin.export_costs(&costs, options).await.unwrap();
        let content = String::from_utf8(result).unwrap();

        assert!(content.starts_with(
            "id,session_id,agent_id,model_name,input_tokens,output_tokens,cost_usd,recorded_at\n"
        ));
        assert!(content.contains("claude-3-opus"));
        assert!(content.contains("1000"));
        assert!(content.contains("500"));
    }

    #[tokio::test]
    async fn test_export_empty_sessions() {
        let plugin = CsvExporterPlugin::new();
        let options = ExportOptions {
            include_headers: true,
            ..Default::default()
        };

        let result = plugin.export_sessions(&[], options).await.unwrap();
        let content = String::from_utf8(result).unwrap();

        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 1);
    }

    #[test]
    fn test_config_schema() {
        let plugin = CsvExporterPlugin::new();
        let schema = plugin.config_schema().unwrap();

        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["delimiter"].is_object());
        assert!(schema["properties"]["quote_char"].is_object());
        assert!(schema["properties"]["include_bom"].is_object());
        assert!(schema["properties"]["line_ending"].is_object());
    }

    #[test]
    fn test_configure() {
        let mut plugin = CsvExporterPlugin::new();
        let config = PluginConfig::new()
            .with_setting("delimiter", ";")
            .with_setting("include_bom", true)
            .with_setting("line_ending", "crlf");

        plugin.configure(config).unwrap();

        assert_eq!(plugin.config.delimiter, ';');
        assert!(plugin.config.include_bom);
        assert!(matches!(plugin.config.line_ending, LineEnding::Crlf));
    }

    #[tokio::test]
    async fn test_init_and_shutdown() {
        let mut plugin = CsvExporterPlugin::new();
        let ctx = PluginContext::new("csv-exporter", "/tmp");

        assert!(!plugin.is_initialized());

        plugin.init(&ctx).await.unwrap();
        assert!(plugin.is_initialized());

        plugin.shutdown().await.unwrap();
        assert!(!plugin.is_initialized());
    }

    #[test]
    fn test_format_and_extension() {
        let plugin = CsvExporterPlugin::new();
        assert_eq!(plugin.format(), "csv");
        assert_eq!(plugin.file_extension(), "csv");
    }

    #[tokio::test]
    async fn test_export_with_bom() {
        let mut plugin = CsvExporterPlugin::new();
        plugin.config.include_bom = true;

        let sessions = vec![create_test_session()];
        let options = ExportOptions {
            include_headers: false,
            ..Default::default()
        };

        let result = plugin.export_sessions(&sessions, options).await.unwrap();

        assert_eq!(&result[0..3], &[0xEF, 0xBB, 0xBF]);
    }
}
