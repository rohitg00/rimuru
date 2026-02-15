use rimuru_plugin_sdk::*;

define_exporter!(
    XmlExporterPlugin,
    name: "xml-exporter",
    version: "0.1.0",
    format: "xml",
    extension: "xml",
    author: "Example Author",
    description: "Export sessions and costs to XML format"
);

impl_plugin_base!(XmlExporterPlugin);

#[async_trait]
impl ExporterPlugin for XmlExporterPlugin {
    fn format(&self) -> &str {
        self.format_name()
    }

    fn file_extension(&self) -> &str {
        self.extension()
    }

    async fn export_sessions(&self, sessions: &[Session], options: ExportOptions) -> RimuruResult<Vec<u8>> {
        let mut xml = String::new();

        xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        xml.push('\n');
        xml.push_str("<sessions>\n");

        for session in sessions {
            xml.push_str("  <session>\n");
            xml.push_str(&format!("    <id>{}</id>\n", session.id));
            xml.push_str(&format!("    <agent_id>{}</agent_id>\n", session.agent_id));
            xml.push_str(&format!("    <status>{:?}</status>\n", session.status));

            let date_format = options.date_format.as_deref().unwrap_or("%Y-%m-%dT%H:%M:%SZ");
            xml.push_str(&format!("    <started_at>{}</started_at>\n", session.started_at.format(date_format)));

            if let Some(ended_at) = session.ended_at {
                xml.push_str(&format!("    <ended_at>{}</ended_at>\n", ended_at.format(date_format)));
            }

            if !session.metadata.is_null() {
                xml.push_str(&format!("    <metadata>{}</metadata>\n", escape_xml(&session.metadata.to_string())));
            }

            xml.push_str("  </session>\n");
        }

        xml.push_str("</sessions>\n");

        if options.pretty {
            Ok(xml.into_bytes())
        } else {
            let compact = xml.lines()
                .map(|l| l.trim())
                .collect::<Vec<_>>()
                .join("");
            Ok(compact.into_bytes())
        }
    }

    async fn export_costs(&self, costs: &[CostRecord], options: ExportOptions) -> RimuruResult<Vec<u8>> {
        let mut xml = String::new();

        xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        xml.push('\n');
        xml.push_str("<costs>\n");

        for cost in costs {
            xml.push_str("  <cost>\n");
            xml.push_str(&format!("    <id>{}</id>\n", cost.id));
            xml.push_str(&format!("    <session_id>{}</session_id>\n", cost.session_id));
            xml.push_str(&format!("    <agent_id>{}</agent_id>\n", cost.agent_id));
            xml.push_str(&format!("    <model_name>{}</model_name>\n", escape_xml(&cost.model_name)));
            xml.push_str(&format!("    <input_tokens>{}</input_tokens>\n", cost.input_tokens));
            xml.push_str(&format!("    <output_tokens>{}</output_tokens>\n", cost.output_tokens));
            xml.push_str(&format!("    <total_tokens>{}</total_tokens>\n", cost.total_tokens()));
            xml.push_str(&format!("    <cost_usd>{:.6}</cost_usd>\n", cost.cost_usd));

            let date_format = options.date_format.as_deref().unwrap_or("%Y-%m-%dT%H:%M:%SZ");
            xml.push_str(&format!("    <recorded_at>{}</recorded_at>\n", cost.recorded_at.format(date_format)));

            xml.push_str("  </cost>\n");
        }

        xml.push_str("</costs>\n");

        if options.pretty {
            Ok(xml.into_bytes())
        } else {
            let compact = xml.lines()
                .map(|l| l.trim())
                .collect::<Vec<_>>()
                .join("");
            Ok(compact.into_bytes())
        }
    }
}

impl XmlExporterPlugin {
    pub fn config_schema(&self) -> Option<serde_json::Value> {
        Some(helpers::create_config_schema(json!({
            "pretty_print": helpers::boolean_property("Format XML with indentation", Some(true)),
            "include_xml_declaration": helpers::boolean_property("Include XML declaration header", Some(true)),
            "date_format": helpers::string_property("Date format string", Some("%Y-%m-%dT%H:%M:%SZ")),
            "encoding": helpers::enum_property("Output encoding", &["UTF-8", "UTF-16", "ISO-8859-1"], Some("UTF-8"))
        })))
    }
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

pub fn create_plugin() -> Box<dyn Plugin> {
    Box::new(XmlExporterPlugin::new())
}

pub fn create_exporter_plugin() -> Box<dyn ExporterPlugin> {
    Box::new(XmlExporterPlugin::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_plugin_info() {
        let plugin = XmlExporterPlugin::new();
        let info = plugin.info();

        assert_eq!(info.name, "xml-exporter");
        assert_eq!(info.version, "0.1.0");
        assert!(info.capabilities.contains(&PluginCapability::Exporter));
    }

    #[tokio::test]
    async fn test_format_and_extension() {
        let plugin = XmlExporterPlugin::new();

        assert_eq!(plugin.format(), "xml");
        assert_eq!(plugin.file_extension(), "xml");
    }

    #[tokio::test]
    async fn test_export_empty_sessions() {
        let plugin = XmlExporterPlugin::new();
        let options = ExportOptions { pretty: true, ..Default::default() };

        let result = plugin.export_sessions(&[], options).await.unwrap();
        let xml = String::from_utf8(result).unwrap();

        assert!(xml.contains("<?xml"));
        assert!(xml.contains("<sessions>"));
        assert!(xml.contains("</sessions>"));
    }

    #[tokio::test]
    async fn test_export_empty_costs() {
        let plugin = XmlExporterPlugin::new();
        let options = ExportOptions { pretty: true, ..Default::default() };

        let result = plugin.export_costs(&[], options).await.unwrap();
        let xml = String::from_utf8(result).unwrap();

        assert!(xml.contains("<?xml"));
        assert!(xml.contains("<costs>"));
        assert!(xml.contains("</costs>"));
    }

    #[test]
    fn test_escape_xml() {
        assert_eq!(escape_xml("test & test"), "test &amp; test");
        assert_eq!(escape_xml("<tag>"), "&lt;tag&gt;");
        assert_eq!(escape_xml("\"quote\""), "&quot;quote&quot;");
    }

    #[tokio::test]
    async fn test_config_schema() {
        let plugin = XmlExporterPlugin::new();
        let schema = plugin.config_schema();

        assert!(schema.is_some());
        let schema = schema.unwrap();
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["pretty_print"].is_object());
    }
}
