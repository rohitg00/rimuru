# Rimuru Plugin Development Guide

This guide explains how to create plugins for Rimuru, the AI agent orchestration and cost tracking platform.

## Table of Contents

1. [Overview](#overview)
2. [Plugin Types](#plugin-types)
3. [Getting Started](#getting-started)
4. [Plugin SDK](#plugin-sdk)
5. [Plugin Manifest](#plugin-manifest)
6. [Creating Plugins](#creating-plugins)
7. [Hooks System](#hooks-system)
8. [Testing Plugins](#testing-plugins)
9. [Publishing Plugins](#publishing-plugins)

## Overview

Rimuru's plugin architecture allows you to extend functionality in several ways:

- **Agent Plugins**: Add support for new AI agents (e.g., custom LLM providers)
- **Exporter Plugins**: Create new data export formats (e.g., XML, Parquet)
- **Notifier Plugins**: Send alerts to new channels (e.g., Microsoft Teams, PagerDuty)
- **View Plugins**: Add custom views to the TUI or desktop app
- **Hook Handlers**: React to events in the system

## Plugin Types

### Agent Plugins

Agent plugins connect Rimuru to AI agent systems for session tracking and cost monitoring.

```rust
use rimuru_plugin_sdk::*;

#[async_trait]
impl AgentPlugin for MyAgentPlugin {
    fn agent_type(&self) -> &str { "my-agent" }
    async fn connect(&mut self) -> RimuruResult<()> { /* ... */ }
    async fn disconnect(&mut self) -> RimuruResult<()> { /* ... */ }
    fn is_connected(&self) -> bool { /* ... */ }
    async fn get_sessions(&self) -> RimuruResult<Vec<Session>> { /* ... */ }
    async fn get_costs(&self) -> RimuruResult<Vec<CostRecord>> { /* ... */ }
}
```

### Exporter Plugins

Exporter plugins convert session and cost data to different formats.

```rust
use rimuru_plugin_sdk::*;

#[async_trait]
impl ExporterPlugin for MyExporterPlugin {
    fn format(&self) -> &str { "my-format" }
    fn file_extension(&self) -> &str { "mf" }
    async fn export_sessions(&self, sessions: &[Session], options: ExportOptions) -> RimuruResult<Vec<u8>> { /* ... */ }
    async fn export_costs(&self, costs: &[CostRecord], options: ExportOptions) -> RimuruResult<Vec<u8>> { /* ... */ }
}
```

### Notifier Plugins

Notifier plugins send alerts and notifications to external services.

```rust
use rimuru_plugin_sdk::*;

#[async_trait]
impl NotifierPlugin for MyNotifierPlugin {
    fn notification_type(&self) -> &str { "my-notifier" }
    async fn send(&self, notification: Notification) -> RimuruResult<()> { /* ... */ }
    async fn test_connection(&self) -> RimuruResult<bool> { /* ... */ }
}
```

## Getting Started

### Prerequisites

- Rust 1.75 or later
- Basic understanding of async Rust
- Familiarity with Rimuru's data models

### Project Setup

1. Create a new Rust library:

```bash
cargo new --lib my-rimuru-plugin
cd my-rimuru-plugin
```

2. Add the SDK dependency to `Cargo.toml`:

```toml
[package]
name = "my-rimuru-plugin"
version = "0.1.0"
edition = "2021"

[dependencies]
rimuru-plugin-sdk = { path = "../rimuru-plugin-sdk" }
async-trait = "0.1"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.45", features = ["full"] }

[lib]
crate-type = ["cdylib", "rlib"]
```

3. Create a plugin manifest file `rimuru-plugin.toml`:

```toml
[plugin]
name = "my-plugin"
version = "0.1.0"
author = "Your Name"
description = "My awesome Rimuru plugin"
plugin_type = "native"
entry_point = "libmy_rimuru_plugin.so"

[capabilities.exporter]
format = "custom"
file_extension = "cst"
supports_sessions = true
supports_costs = true
```

## Plugin SDK

The `rimuru-plugin-sdk` crate provides everything you need to build plugins:

### Re-exported Types

```rust
use rimuru_plugin_sdk::{
    // Core plugin traits
    Plugin, AgentPlugin, ExporterPlugin, NotifierPlugin, ViewPlugin,

    // Data types
    Session, SessionStatus, CostRecord, AgentType, MetricsSnapshot,

    // Plugin types
    PluginInfo, PluginConfig, PluginContext, PluginCapability,
    PluginManifest, PluginEvent, PluginState,

    // Export types
    ExportData, ExportOptions,

    // Notification types
    Notification, NotificationLevel,

    // Hook types
    Hook, HookHandler, HookContext, HookData, HookResult,

    // Error handling
    RimuruError, RimuruResult,
};
```

### Macros

The SDK provides convenience macros to reduce boilerplate:

```rust
use rimuru_plugin_sdk::*;

// Define plugin metadata
rimuru_plugin!(
    name: "my-plugin",
    version: "0.1.0",
    author: "Your Name",
    description: "My plugin description",
    capabilities: [PluginCapability::Exporter],
    homepage: "https://example.com",
    license: "MIT"
);

// Quick plugin struct definition
define_exporter!(
    MyExporter,
    name: "my-exporter",
    version: "0.1.0",
    format: "custom",
    extension: "cst",
    author: "Your Name",
    description: "Custom export format"
);

// Implement base Plugin trait automatically
impl_plugin_base!(MyExporter);
```

### Config Schema Helpers

Build JSON Schema for your plugin configuration:

```rust
use rimuru_plugin_sdk::helpers::*;

let schema = create_config_schema(json!({
    "api_key": string_property("API key for authentication", None),
    "timeout": integer_property("Request timeout in seconds", Some(30)),
    "enabled": boolean_property("Enable this feature", Some(true)),
    "log_level": enum_property("Logging level", &["debug", "info", "warn", "error"], Some("info"))
}));
```

## Plugin Manifest

Every plugin must have a `rimuru-plugin.toml` manifest file:

```toml
[plugin]
name = "example-plugin"
version = "0.1.0"
author = "Your Name"
description = "Plugin description"
homepage = "https://example.com"
repository = "https://github.com/example/plugin"
license = "MIT"
plugin_type = "native"  # native, wasm, or script
entry_point = "libexample_plugin.so"

# Declare capabilities (at least one required)
[capabilities.agent]
agent_type = "my-agent"
supports_sessions = true
supports_costs = true
supports_streaming = false

[capabilities.exporter]
format = "custom"
file_extension = "cst"
supports_sessions = true
supports_costs = true

[capabilities.notifier]
notification_type = "my-notifier"
supports_batch = true
rate_limit = 60

[capabilities.view]
view_name = "my-view"
view_title = "My Custom View"
keybind = "v"

# Declare required permissions
[[permissions]]
name = "network"
description = "Required to call external API"
required = true

[[permissions]]
name = "filesystem"
description = "Required to cache data"
required = false

# Declare dependencies on other plugins
[[dependencies]]
name = "other-plugin"
version_requirement = ">=1.0.0"
optional = false

# Configuration schema and defaults
[config]
schema = { type = "object", properties = { api_key = { type = "string" } } }

[config.defaults]
timeout = 30
enabled = true

# Hook registrations
[[hooks]]
hook = "on_cost_recorded"
handler = "handle_cost"
priority = 10
```

## Creating Plugins

### Example: Agent Plugin

See `examples/plugins/example_agent/` for a complete example.

Key points:
- Implement both `Plugin` and `AgentPlugin` traits
- Handle connection state properly
- Return appropriate errors when not connected
- Support the `watch_sessions` callback for real-time updates

### Example: Exporter Plugin

See `examples/plugins/example_exporter/` for a complete example.

Key points:
- Implement both `Plugin` and `ExporterPlugin` traits
- Support both sessions and costs export
- Respect `ExportOptions` (date format, pretty print, etc.)
- Handle empty data gracefully

### Example: Notifier Plugin

See `examples/plugins/example_notifier/` for a complete example.

Key points:
- Implement both `Plugin` and `NotifierPlugin` traits
- Support different notification levels
- Implement rate limiting for batch sends
- Provide a `test_connection` implementation

## Hooks System

Plugins can register handlers for system events:

```rust
use rimuru_plugin_sdk::*;

// Define a hook handler
hook_handler!(
    MyCostHandler,
    name: "my-cost-handler",
    hook: Hook::OnCostRecorded,
    priority: 10,
    description: "Handle cost recording events"
);

// Implement the handler
impl_hook_handler!(MyCostHandler, |ctx| {
    if let HookData::Cost(cost) = &ctx.data {
        info!("Cost recorded: ${:.4}", cost.cost_usd);

        if cost.cost_usd > 1.0 {
            return Ok(HookResult::abort("Cost exceeds budget"));
        }
    }
    Ok(HookResult::ok())
});
```

### Available Hooks

| Hook | Description | Data Type |
|------|-------------|-----------|
| `PreSessionStart` | Before a session starts | `Session` |
| `PostSessionEnd` | After a session ends | `Session` |
| `OnCostRecorded` | When cost is recorded | `CostRecord` |
| `OnMetricsCollected` | When metrics are collected | `MetricsSnapshot` |
| `OnAgentConnect` | When an agent connects | `Agent` |
| `OnAgentDisconnect` | When an agent disconnects | `Agent` |
| `OnSyncComplete` | After data sync completes | `Sync` |
| `OnPluginLoaded` | When a plugin loads | `Plugin` |
| `OnPluginUnloaded` | When a plugin unloads | `Plugin` |
| `OnConfigChanged` | When config changes | `Config` |
| `OnError` | When an error occurs | `Error` |
| `Custom(String)` | Custom hooks | `Custom` |

### Hook Results

- `HookResult::Continue` - Continue to next handler
- `HookResult::Abort { reason }` - Stop the hook chain with error
- `HookResult::Modified { data, message }` - Pass modified data to next handler
- `HookResult::Skip` - Skip this handler, continue chain

## Testing Plugins

### Unit Tests

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
        let plugin = MyExporter::new();
        let sessions = vec![/* test data */];

        let result = plugin.export_sessions(&sessions, ExportOptions::default()).await;
        assert!(result.is_ok());
    }
}
```

### Integration Tests

```bash
# Build your plugin
cargo build --release

# Install in Rimuru
rimuru-cli plugins install ./target/release/libmy_plugin.so

# Enable the plugin
rimuru-cli plugins enable my-plugin

# Test functionality
rimuru-cli export --format my-format sessions.cst
```

## Publishing Plugins

### Plugin Registry (Coming Soon)

Plugins will be publishable to a central registry:

```bash
# Package your plugin
cargo build --release

# Publish (future)
rimuru-cli plugins publish
```

### Manual Distribution

1. Build your plugin:
   ```bash
   cargo build --release
   ```

2. Package the following files:
   - `target/release/libmy_plugin.so` (or .dylib/.dll)
   - `rimuru-plugin.toml`
   - `README.md`

3. Users install with:
   ```bash
   rimuru-cli plugins install /path/to/plugin
   ```

## Best Practices

1. **Error Handling**: Use `RimuruError` for consistent error types
2. **Logging**: Use `tracing` macros (info!, warn!, error!) for logging
3. **Configuration**: Provide sensible defaults and validate config
4. **Testing**: Write comprehensive unit and integration tests
5. **Documentation**: Document your plugin's capabilities and configuration
6. **Security**: Request only necessary permissions
7. **Versioning**: Follow semver for plugin versions

## Resources

- [Example Plugins](../../examples/plugins/)
- [Plugin SDK Documentation](../rimuru-plugin-sdk/)
- [Hooks Reference](./hooks.md)
- [Rimuru Core Types](../rimuru-core/)

## Getting Help

- Open an issue on GitHub
- Join the Rimuru Discord community
- Check the FAQ in the wiki
