---
type: reference
title: Plugin Development Guide
created: 2026-02-05
tags:
  - plugins
  - development
  - developer-guide
related:
  - "[[architecture]]"
  - "[[api-reference]]"
  - "[[creating-adapters]]"
---

# Plugin Development Guide

This guide covers how to develop plugins for Rimuru using the Plugin SDK.

## Overview

Rimuru's plugin system allows extending functionality through four plugin types:

| Type | Purpose | Example |
|------|---------|---------|
| `Exporter` | Export sessions/costs to formats | CSV, JSON, XML |
| `Notifier` | Send notifications | Slack, Discord, Webhook |
| `Agent` | Add agent support | Custom AI agents |
| `View` | Custom UI views | TUI/Desktop widgets |

## Getting Started

### Create a New Plugin Project

```bash
cargo new my-rimuru-plugin --lib
cd my-rimuru-plugin
```

### Add Dependencies

```toml
# Cargo.toml
[package]
name = "my-rimuru-plugin"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
rimuru-plugin-sdk = "0.1"
async-trait = "0.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }
```

## Exporter Plugin

Export sessions and costs to custom formats.

### Using Macros (Recommended)

```rust
use rimuru_plugin_sdk::*;

// Define the plugin with the macro
define_exporter!(
    MyExporterPlugin,
    name: "my-exporter",
    version: "0.1.0",
    format: "myformat",
    extension: "mf",
    author: "Your Name",
    description: "Export to MyFormat"
);

// Implement base plugin trait
impl_plugin_base!(MyExporterPlugin);

#[async_trait]
impl ExporterPlugin for MyExporterPlugin {
    fn format(&self) -> &str {
        self.format_name()
    }

    fn file_extension(&self) -> &str {
        self.extension()
    }

    async fn export_sessions(
        &self,
        sessions: &[Session],
        options: ExportOptions,
    ) -> RimuruResult<Vec<u8>> {
        let mut output = String::new();

        // Build your format
        output.push_str("# Sessions\n");
        for session in sessions {
            output.push_str(&format!(
                "- {} | {} | {:?}\n",
                session.id,
                session.agent_id,
                session.status
            ));
        }

        Ok(output.into_bytes())
    }

    async fn export_costs(
        &self,
        costs: &[CostRecord],
        options: ExportOptions,
    ) -> RimuruResult<Vec<u8>> {
        let mut output = String::new();

        output.push_str("# Costs\n");
        for cost in costs {
            output.push_str(&format!(
                "- {} | {} tokens | ${:.4}\n",
                cost.model_name,
                cost.total_tokens(),
                cost.cost_usd
            ));
        }

        Ok(output.into_bytes())
    }
}

// Factory function for dynamic loading
pub fn create_plugin() -> Box<dyn Plugin> {
    Box::new(MyExporterPlugin::new())
}

pub fn create_exporter_plugin() -> Box<dyn ExporterPlugin> {
    Box::new(MyExporterPlugin::new())
}
```

### Manual Implementation

```rust
use rimuru_plugin_sdk::*;

pub struct MarkdownExporter {
    info: PluginInfo,
}

impl MarkdownExporter {
    pub fn new() -> Self {
        Self {
            info: PluginInfo {
                name: "markdown-exporter".to_string(),
                version: "0.1.0".to_string(),
                author: Some("Your Name".to_string()),
                description: Some("Export to Markdown".to_string()),
                capabilities: vec![PluginCapability::Exporter],
                permissions: vec![],
                dependencies: vec![],
            },
        }
    }
}

#[async_trait]
impl Plugin for MarkdownExporter {
    fn info(&self) -> &PluginInfo {
        &self.info
    }

    async fn initialize(&mut self, _ctx: &PluginContext) -> RimuruResult<()> {
        Ok(())
    }

    async fn shutdown(&mut self) -> RimuruResult<()> {
        Ok(())
    }

    fn state(&self) -> PluginState {
        PluginState::Running
    }
}

#[async_trait]
impl ExporterPlugin for MarkdownExporter {
    fn format(&self) -> &str {
        "markdown"
    }

    fn file_extension(&self) -> &str {
        "md"
    }

    async fn export_sessions(
        &self,
        sessions: &[Session],
        options: ExportOptions,
    ) -> RimuruResult<Vec<u8>> {
        let mut md = String::new();

        md.push_str("# Session Report\n\n");
        md.push_str("| ID | Agent | Status | Duration |\n");
        md.push_str("|---|---|---|---|\n");

        for session in sessions {
            let duration = session.ended_at
                .map(|e| e.signed_duration_since(session.started_at))
                .map(|d| format!("{}m", d.num_minutes()))
                .unwrap_or_else(|| "ongoing".to_string());

            md.push_str(&format!(
                "| {} | {} | {:?} | {} |\n",
                session.id,
                session.agent_id,
                session.status,
                duration
            ));
        }

        Ok(md.into_bytes())
    }

    async fn export_costs(
        &self,
        costs: &[CostRecord],
        options: ExportOptions,
    ) -> RimuruResult<Vec<u8>> {
        let mut md = String::new();

        md.push_str("# Cost Report\n\n");
        md.push_str("| Model | Input | Output | Cost |\n");
        md.push_str("|---|---|---|---|\n");

        let total: f64 = costs.iter().map(|c| c.cost_usd).sum();

        for cost in costs {
            md.push_str(&format!(
                "| {} | {} | {} | ${:.4} |\n",
                cost.model_name,
                cost.input_tokens,
                cost.output_tokens,
                cost.cost_usd
            ));
        }

        md.push_str(&format!("\n**Total: ${:.2}**\n", total));

        Ok(md.into_bytes())
    }
}
```

## Notifier Plugin

Send notifications for events.

```rust
use rimuru_plugin_sdk::*;

define_notifier!(
    MyNotifierPlugin,
    name: "my-notifier",
    version: "0.1.0",
    author: "Your Name",
    description: "Send notifications to MyService"
);

impl_plugin_base!(MyNotifierPlugin);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyNotifierConfig {
    pub endpoint: String,
    pub api_key: String,
    pub channel: Option<String>,
}

impl MyNotifierPlugin {
    pub fn with_config(config: MyNotifierConfig) -> Self {
        let mut plugin = Self::new();
        plugin.config = Some(config);
        plugin
    }
}

#[async_trait]
impl NotifierPlugin for MyNotifierPlugin {
    async fn send(&self, notification: Notification) -> RimuruResult<()> {
        let config = self.config.as_ref()
            .ok_or_else(|| RimuruError::Plugin("Not configured".to_string()))?;

        // Build your notification payload
        let payload = serde_json::json!({
            "title": notification.title,
            "message": notification.message,
            "level": format!("{:?}", notification.level),
            "channel": config.channel,
        });

        // Send the notification
        let client = reqwest::Client::new();
        let response = client
            .post(&config.endpoint)
            .header("Authorization", format!("Bearer {}", config.api_key))
            .json(&payload)
            .send()
            .await
            .map_err(|e| RimuruError::Plugin(format!("Send failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(RimuruError::Plugin(
                format!("Notification failed: {}", response.status())
            ));
        }

        Ok(())
    }

    fn supported_levels(&self) -> Vec<NotificationLevel> {
        vec![
            NotificationLevel::Info,
            NotificationLevel::Warning,
            NotificationLevel::Error,
            NotificationLevel::Critical,
        ]
    }
}
```

## View Plugin

Create custom UI views for TUI or desktop.

```rust
use rimuru_plugin_sdk::*;

pub struct DashboardViewPlugin {
    info: PluginInfo,
}

impl DashboardViewPlugin {
    pub fn new() -> Self {
        Self {
            info: PluginInfo {
                name: "custom-dashboard".to_string(),
                version: "0.1.0".to_string(),
                author: Some("Your Name".to_string()),
                description: Some("Custom dashboard view".to_string()),
                capabilities: vec![PluginCapability::View],
                permissions: vec![],
                dependencies: vec![],
            },
        }
    }
}

#[async_trait]
impl Plugin for DashboardViewPlugin {
    fn info(&self) -> &PluginInfo {
        &self.info
    }

    async fn initialize(&mut self, _ctx: &PluginContext) -> RimuruResult<()> {
        Ok(())
    }

    async fn shutdown(&mut self) -> RimuruResult<()> {
        Ok(())
    }

    fn state(&self) -> PluginState {
        PluginState::Running
    }
}

#[async_trait]
impl ViewPlugin for DashboardViewPlugin {
    fn view_name(&self) -> &str {
        "custom-dashboard"
    }

    fn view_title(&self) -> &str {
        "Custom Dashboard"
    }

    async fn render(&self, ctx: &ViewContext) -> RimuruResult<ViewOutput> {
        // Build widget data
        let widgets = vec![
            WidgetData {
                widget_type: "stat".to_string(),
                title: "Total Sessions".to_string(),
                data: serde_json::json!({
                    "value": 42,
                    "trend": "+5%"
                }),
            },
            WidgetData {
                widget_type: "chart".to_string(),
                title: "Cost Trend".to_string(),
                data: serde_json::json!({
                    "type": "line",
                    "data": [10, 20, 15, 25, 30]
                }),
            },
        ];

        Ok(ViewOutput {
            view_type: "dashboard".to_string(),
            widgets,
            actions: vec![
                ViewAction {
                    id: "refresh".to_string(),
                    label: "Refresh".to_string(),
                    shortcut: Some("r".to_string()),
                },
            ],
        })
    }

    async fn handle_action(
        &self,
        action: &str,
        input: ViewInput,
    ) -> RimuruResult<ViewOutput> {
        match action {
            "refresh" => {
                // Re-render with fresh data
                self.render(&ViewContext::default()).await
            }
            _ => Err(RimuruError::Plugin(
                format!("Unknown action: {}", action)
            )),
        }
    }
}
```

## Plugin Manifest

Create a `plugin.toml` manifest for your plugin:

```toml
[plugin]
name = "my-plugin"
version = "0.1.0"
description = "My awesome Rimuru plugin"
author = "Your Name"
license = "MIT"
repository = "https://github.com/you/my-plugin"

[capabilities]
type = "exporter"  # exporter, notifier, agent, view
format = "myformat"
extension = "mf"

[config]
schema = """
{
  "type": "object",
  "properties": {
    "option1": { "type": "string", "default": "value" },
    "option2": { "type": "boolean", "default": true }
  }
}
"""

[dependencies]
rimuru = ">=0.1.0"

[permissions]
filesystem = ["read", "write:~/.rimuru/exports"]
network = ["https://api.example.com"]
```

## Configuration Schema

Use the helpers module to define configuration schemas:

```rust
use rimuru_plugin_sdk::helpers;

impl MyPlugin {
    pub fn config_schema(&self) -> Option<serde_json::Value> {
        Some(helpers::create_config_schema(json!({
            "api_key": helpers::string_property(
                "API key for authentication",
                None
            ),
            "timeout_seconds": helpers::integer_property(
                "Request timeout in seconds",
                Some(30)
            ),
            "retry_count": helpers::integer_property(
                "Number of retries on failure",
                Some(3)
            ),
            "enabled": helpers::boolean_property(
                "Enable the plugin",
                Some(true)
            ),
            "log_level": helpers::enum_property(
                "Logging level",
                &["debug", "info", "warn", "error"],
                Some("info")
            ),
            "tags": helpers::array_property(
                "Tags to apply to exports",
                "string"
            ),
        })))
    }
}
```

## Hook Integration

Register hooks to respond to events:

```rust
impl MyPlugin {
    fn register_hooks(&self, manager: &mut HookManager) {
        // Called when a session starts
        manager.register(
            Hook::SessionStart,
            HookConfig::default(),
            Box::new(|data| {
                if let HookData::Session(session) = data {
                    println!("Session started: {}", session.id);
                }
                HookResult::Continue
            }),
        );

        // Called when cost exceeds threshold
        manager.register(
            Hook::CostThreshold,
            HookConfig {
                threshold: Some(10.0),
                ..Default::default()
            },
            Box::new(|data| {
                if let HookData::Cost(cost) = data {
                    println!("Cost alert: ${:.2}", cost.cost_usd);
                }
                HookResult::Continue
            }),
        );
    }
}
```

## Testing Your Plugin

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_plugin_info() {
        let plugin = MyPlugin::new();
        let info = plugin.info();

        assert_eq!(info.name, "my-plugin");
        assert!(info.capabilities.contains(&PluginCapability::Exporter));
    }

    #[tokio::test]
    async fn test_export_sessions() {
        let plugin = MyExporterPlugin::new();
        let sessions = vec![
            create_test_session(),
        ];

        let result = plugin.export_sessions(
            &sessions,
            ExportOptions::default(),
        ).await;

        assert!(result.is_ok());
        let output = String::from_utf8(result.unwrap()).unwrap();
        assert!(output.contains("Sessions"));
    }

    #[tokio::test]
    async fn test_notification() {
        let plugin = MyNotifierPlugin::with_config(MyNotifierConfig {
            endpoint: "https://httpbin.org/post".to_string(),
            api_key: "test".to_string(),
            channel: None,
        });

        let notification = Notification {
            title: "Test".to_string(),
            message: "Test message".to_string(),
            level: NotificationLevel::Info,
            metadata: Default::default(),
        };

        let result = plugin.send(notification).await;
        // In real tests, use wiremock for HTTP mocking
    }

    fn create_test_session() -> Session {
        Session {
            id: Uuid::new_v4(),
            agent_id: Uuid::new_v4(),
            status: SessionStatus::Completed,
            started_at: Utc::now(),
            ended_at: Some(Utc::now()),
            metadata: serde_json::json!({}),
        }
    }
}
```

## Building and Distribution

### Build the Plugin

```bash
cargo build --release
```

The plugin library will be at `target/release/libmy_plugin.so` (Linux), `.dylib` (macOS), or `.dll` (Windows).

### Package for Distribution

1. Create a release directory:
   ```bash
   mkdir -p dist
   cp target/release/libmy_plugin.so dist/
   cp plugin.toml dist/
   ```

2. Create a tarball:
   ```bash
   tar -czvf my-plugin-0.1.0.tar.gz dist/
   ```

### Installation

Users install plugins via:

```bash
# From file
rimuru plugins install ./my-plugin-0.1.0.tar.gz

# From URL
rimuru plugins install https://example.com/my-plugin-0.1.0.tar.gz
```

## Security Considerations

Plugins run in a sandbox with restricted capabilities:

### Filesystem Access

```rust
// Declare required paths in manifest
[permissions]
filesystem = [
    "read:~/.config/myapp",
    "write:~/.rimuru/exports"
]
```

### Network Access

```rust
// Declare required endpoints
[permissions]
network = [
    "https://api.slack.com",
    "https://api.myservice.com"
]
```

### Resource Limits

Plugins have default limits:
- Memory: 256MB
- CPU: 10% of one core
- File handles: 100

## Best Practices

1. **Error Handling**: Return meaningful errors with context
2. **Logging**: Use `tracing` for structured logging
3. **Configuration**: Validate config on initialization
4. **Async**: Use async for I/O operations
5. **Testing**: Write comprehensive tests
6. **Documentation**: Document config options and behavior

## Example Plugins

See `examples/plugins/` for complete examples:

- `example_exporter/` - XML exporter plugin
- `example_notifier/` - Custom webhook notifier
- `example_agent/` - Custom agent adapter

## See Also

- [[architecture]] - System architecture
- [[api-reference]] - API documentation
- [[creating-adapters]] - Agent adapter development
