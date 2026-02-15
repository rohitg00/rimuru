use rimuru_plugin_sdk::*;

define_agent!(
    ExampleAgentPlugin,
    name: "example-agent",
    version: "0.1.0",
    agent_type: "example",
    author: "Example Author",
    description: "An example agent plugin demonstrating how to create a Rimuru agent adapter"
);

impl_plugin_base!(ExampleAgentPlugin);

#[async_trait]
impl AgentPlugin for ExampleAgentPlugin {
    fn agent_type(&self) -> &str {
        self.agent_type_name()
    }

    async fn connect(&mut self) -> RimuruResult<()> {
        info!("ExampleAgent: Connecting to example service...");
        self.connected = true;
        Ok(())
    }

    async fn disconnect(&mut self) -> RimuruResult<()> {
        info!("ExampleAgent: Disconnecting from example service...");
        self.connected = false;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    async fn get_sessions(&self) -> RimuruResult<Vec<Session>> {
        if !self.connected {
            return Err(RimuruError::plugin("Not connected to agent"));
        }

        let session = Session {
            id: Uuid::new_v4(),
            agent_id: Uuid::new_v4(),
            status: SessionStatus::Completed,
            started_at: Utc::now() - chrono::Duration::hours(1),
            ended_at: Some(Utc::now()),
            metadata: serde_json::json!({
                "model": "example-model",
                "project_path": "/example/project",
                "total_tokens": 1000,
                "input_tokens": 800,
                "output_tokens": 200,
                "estimated_cost": 0.01,
                "example_key": "example_value"
            }),
        };

        Ok(vec![session])
    }

    async fn get_costs(&self) -> RimuruResult<Vec<CostRecord>> {
        if !self.connected {
            return Err(RimuruError::plugin("Not connected to agent"));
        }

        let session_id = Uuid::new_v4();
        let agent_id = Uuid::new_v4();
        let cost = CostRecord::new(
            session_id,
            agent_id,
            "example-model".to_string(),
            800,
            200,
            0.01,
        );

        Ok(vec![cost])
    }

    async fn watch_sessions(&self, callback: SessionCallback) -> RimuruResult<()> {
        if !self.connected {
            return Err(RimuruError::plugin("Not connected to agent"));
        }

        let session = Session::new(
            Uuid::new_v4(),
            serde_json::json!({
                "model": "example-model",
                "project_path": "/example/project"
            }),
        );

        callback(session);
        Ok(())
    }
}

impl ExampleAgentPlugin {
    pub fn config_schema(&self) -> Option<serde_json::Value> {
        Some(helpers::create_config_schema(json!({
            "api_endpoint": helpers::string_property("API endpoint URL", Some("https://api.example.com")),
            "api_key": helpers::string_property("API key for authentication", None),
            "sync_interval_seconds": helpers::integer_property("Interval between syncs", Some(300)),
            "max_sessions": helpers::integer_property("Maximum sessions to fetch", Some(100)),
            "include_inactive": helpers::boolean_property("Include inactive sessions", Some(false))
        })))
    }
}

pub fn create_plugin() -> Box<dyn Plugin> {
    Box::new(ExampleAgentPlugin::new())
}

pub fn create_agent_plugin() -> Box<dyn AgentPlugin> {
    Box::new(ExampleAgentPlugin::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_plugin_info() {
        let plugin = ExampleAgentPlugin::new();
        let info = plugin.info();

        assert_eq!(info.name, "example-agent");
        assert_eq!(info.version, "0.1.0");
        assert!(info.capabilities.contains(&PluginCapability::Agent));
    }

    #[tokio::test]
    async fn test_connect_disconnect() {
        let mut plugin = ExampleAgentPlugin::new();

        assert!(!plugin.is_connected());

        plugin.connect().await.unwrap();
        assert!(plugin.is_connected());

        plugin.disconnect().await.unwrap();
        assert!(!plugin.is_connected());
    }

    #[tokio::test]
    async fn test_get_sessions_when_connected() {
        let mut plugin = ExampleAgentPlugin::new();
        plugin.connect().await.unwrap();

        let sessions = plugin.get_sessions().await.unwrap();
        assert!(!sessions.is_empty());
    }

    #[tokio::test]
    async fn test_get_sessions_when_disconnected() {
        let plugin = ExampleAgentPlugin::new();

        let result = plugin.get_sessions().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_config_schema() {
        let plugin = ExampleAgentPlugin::new();
        let schema = plugin.config_schema();

        assert!(schema.is_some());
        let schema = schema.unwrap();
        assert_eq!(schema["type"], "object");
    }
}
