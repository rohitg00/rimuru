---
type: reference
title: Installing and Configuring Plugins
created: 2026-02-05
tags:
  - plugins
  - extensions
  - exporters
  - notifiers
related:
  - "[[skills]]"
  - "[[hooks]]"
  - "[[cli-reference]]"
---

# Installing and Configuring Plugins

Rimuru's plugin system extends functionality through modular plugins. Add custom agents, exporters, notifiers, and more.

## Plugin Types

| Type | Description | Examples |
|------|-------------|----------|
| **Agent** | Add support for new AI agents | Custom agent adapters |
| **Exporter** | Export data to various formats | CSV, JSON, Prometheus |
| **Notifier** | Send notifications | Slack, Discord, Webhook |
| **View** | Add TUI views | Custom dashboards |
| **Hook** | React to events | Auto-save, logging |
| **Custom** | Other extensions | Analytics, integrations |

## Built-in Plugins

Rimuru includes several built-in plugins:

```bash
rimuru plugins builtin
```

Output:

```
Built-in Plugins

┌────────────────────┬──────────┬─────────────────────────────────────────────┐
│ Name               │ Type     │ Description                                 │
├────────────────────┼──────────┼─────────────────────────────────────────────┤
│ csv-exporter       │ Exporter │ Export cost and session data to CSV format  │
│ json-exporter      │ Exporter │ Export data to JSON format                  │
│ webhook-notifier   │ Notifier │ Send notifications via webhooks             │
│ slack-notifier     │ Notifier │ Send notifications to Slack channels        │
│ discord-notifier   │ Notifier │ Send notifications to Discord channels      │
└────────────────────┴──────────┴─────────────────────────────────────────────┘

  Built-in plugins are included with Rimuru and don't need installation.
  Configure them using the hooks system or plugin configuration.
```

## CLI Commands

### List Plugins

View installed plugins:

```bash
rimuru plugins list
```

Show all plugins including disabled:

```bash
rimuru plugins list --all
```

Filter by capability:

```bash
rimuru plugins list --capability exporter
rimuru plugins list --capability notifier
```

### Install Plugins

Install from local path:

```bash
rimuru plugins install ./my-plugin
```

Install and enable immediately:

```bash
rimuru plugins install ./my-plugin --enable
```

### Enable/Disable Plugins

Enable a plugin:

```bash
rimuru plugins enable <plugin-name>
```

Disable a plugin:

```bash
rimuru plugins disable <plugin-name>
```

### Uninstall Plugins

Remove a plugin:

```bash
rimuru plugins uninstall <plugin-name>
```

Force uninstall without confirmation:

```bash
rimuru plugins uninstall <plugin-name> --force
```

### View Plugin Info

Show detailed plugin information:

```bash
rimuru plugins info <plugin-name>
```

Output:

```
Plugin Information
══════════════════════════════════════════════════════

  Name:           slack-notifier
  Version:        1.0.0
  Author:         Rimuru Team
  Description:    Send cost alerts and session notifications to Slack

  Status
  Status:         Enabled
  Loaded At:      2026-02-05 14:30:00 UTC

  Capabilities
    • Notifier

  Metadata
  Homepage:       https://github.com/rohitg00/rimuru
  Repository:     https://github.com/rohitg00/rimuru
  License:        Apache-2.0

  Files
  Plugin Dir:     ~/.config/rimuru/plugins/slack-notifier
  Manifest:       slack-notifier
```

### Configure Plugins

View current configuration:

```bash
rimuru plugins config <plugin-name> --show
```

Set a configuration value:

```bash
rimuru plugins config <plugin-name> --key webhook_url --value "https://hooks.slack.com/..."
```

Example for Slack notifier:

```bash
rimuru plugins config slack-notifier --key webhook_url --value "https://hooks.slack.com/services/T00/B00/xxx"
rimuru plugins config slack-notifier --key channel --value "#costs"
rimuru plugins config slack-notifier --key notify_on --value '["cost_threshold", "session_end"]'
```

## Plugin Manifest

Plugins are defined by a `rimuru-plugin.toml` manifest:

```toml
[plugin]
name = "my-plugin"
version = "1.0.0"
author = "Your Name"
description = "A custom Rimuru plugin"
homepage = "https://example.com"
repository = "https://github.com/user/my-plugin"
license = "MIT"

[plugin.capabilities]
types = ["notifier"]

[dependencies]
# Optional plugin dependencies

[[config]]
key = "webhook_url"
type = "string"
required = true
description = "URL to send notifications"

[[config]]
key = "enabled_events"
type = "array"
required = false
default = ["cost_alert", "session_end"]
description = "Events that trigger notifications"
```

## Configuring Built-in Plugins

### CSV Exporter

Export data to CSV format:

```bash
rimuru plugins config csv-exporter --key output_dir --value "./exports"
rimuru plugins config csv-exporter --key include_headers --value true
```

Configuration options:
- `output_dir`: Directory for exported files
- `include_headers`: Include column headers (default: true)
- `date_format`: Date format string (default: %Y-%m-%d)

### JSON Exporter

Export data to JSON format:

```bash
rimuru plugins config json-exporter --key output_dir --value "./exports"
rimuru plugins config json-exporter --key pretty_print --value true
```

Configuration options:
- `output_dir`: Directory for exported files
- `pretty_print`: Format JSON with indentation (default: true)

### Webhook Notifier

Send notifications via HTTP webhooks:

```bash
rimuru plugins config webhook-notifier --key url --value "https://api.example.com/webhook"
rimuru plugins config webhook-notifier --key method --value "POST"
rimuru plugins config webhook-notifier --key headers --value '{"Authorization": "Bearer token"}'
```

Configuration options:
- `url`: Webhook URL (required)
- `method`: HTTP method (default: POST)
- `headers`: Custom headers as JSON object
- `events`: Events to notify on

### Slack Notifier

Send notifications to Slack:

```bash
rimuru plugins config slack-notifier --key webhook_url --value "https://hooks.slack.com/services/..."
rimuru plugins config slack-notifier --key channel --value "#rimuru-alerts"
rimuru plugins config slack-notifier --key cost_threshold --value 10.0
```

Configuration options:
- `webhook_url`: Slack webhook URL (required)
- `channel`: Target channel (optional, uses webhook default)
- `username`: Bot username (default: Rimuru)
- `icon_emoji`: Bot icon (default: :robot_face:)
- `cost_threshold`: Cost threshold for alerts
- `notify_on`: Array of event types

### Discord Notifier

Send notifications to Discord:

```bash
rimuru plugins config discord-notifier --key webhook_url --value "https://discord.com/api/webhooks/..."
rimuru plugins config discord-notifier --key username --value "Rimuru Bot"
```

Configuration options:
- `webhook_url`: Discord webhook URL (required)
- `username`: Bot username (default: Rimuru)
- `avatar_url`: Bot avatar URL
- `embed_color`: Embed color (hex)
- `notify_on`: Array of event types

## Plugin Directory Structure

Plugins are stored in:
- Linux/macOS: `~/.config/rimuru/plugins/`
- Windows: `%APPDATA%\rimuru\plugins\`

Structure:

```
plugins/
├── my-plugin/
│   ├── rimuru-plugin.toml    # Manifest
│   ├── config.json           # Runtime configuration
│   └── src/                  # Plugin source/binary
└── another-plugin/
    ├── rimuru-plugin.toml
    └── ...
```

## Plugin Events

Plugins can subscribe to these events:

| Event | Description |
|-------|-------------|
| `session_start` | Session begins |
| `session_end` | Session completes |
| `cost_recorded` | Cost record created |
| `cost_threshold` | Cost exceeds threshold |
| `agent_connected` | Agent connects |
| `agent_disconnected` | Agent disconnects |
| `sync_complete` | Model sync completes |
| `error` | Error occurs |

## Creating Custom Plugins

See the [[creating-plugins]] developer guide for detailed instructions on creating plugins.

Basic plugin structure:

```rust
use rimuru_plugin_sdk::prelude::*;

pub struct MyPlugin {
    config: PluginConfig,
}

#[async_trait]
impl Plugin for MyPlugin {
    fn info(&self) -> PluginInfo {
        PluginInfo::builder()
            .name("my-plugin")
            .version("1.0.0")
            .capabilities(vec![PluginCapability::Notifier])
            .build()
    }

    async fn on_load(&mut self, config: PluginConfig) -> Result<()> {
        self.config = config;
        Ok(())
    }

    async fn on_event(&self, event: PluginEvent) -> Result<()> {
        match event {
            PluginEvent::CostRecorded(cost) => {
                // Handle cost event
            }
            _ => {}
        }
        Ok(())
    }
}
```

## JSON Output

For scripting:

```bash
rimuru plugins list --format json
```

## Related Topics

- [[skills]] - SkillKit integration
- [[hooks]] - Hook system
- [[cli-reference]] - Complete CLI reference
