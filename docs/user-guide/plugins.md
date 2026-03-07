# Plugins

Rimuru supports plugins for extending functionality.

## Managing Plugins

### CLI

```bash
rimuru plugins list              # List all plugins
rimuru plugins install <id>      # Install a plugin
rimuru plugins uninstall <id>    # Uninstall a plugin
```

### API

```bash
curl http://localhost:3100/api/plugins
curl -X POST http://localhost:3100/api/plugins/install -d '{"plugin_id": "my-plugin"}'
curl -X DELETE http://localhost:3100/api/plugins/my-plugin
```

### Web UI

The Plugins page shows installed plugins with toggle controls.

### TUI

Press `7` to switch to the Plugins tab.

## Plugin Data

Each plugin stores:

| Field | Description |
|-------|-------------|
| `id` | Plugin identifier |
| `name` | Display name |
| `version` | Plugin version |
| `description` | What the plugin does |
| `author` | Plugin author |
| `installed` | Whether it's installed |
| `config` | Plugin configuration |

## Hooks

Hooks trigger actions in response to events. See [Hooks](../plugins/hooks.md) for the full reference.

### Quick Example

```bash
# Register a hook
rimuru hooks register --event agent_connected --function my-handler

# List hooks
rimuru hooks list

# Dispatch an event
rimuru hooks dispatch --event agent_connected --payload '{}'
```
