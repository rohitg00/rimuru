# Plugin System

Rimuru's plugin system uses iii-engine functions for registration and management.

## How Plugins Work

Plugins are managed through iii functions in `crates/rimuru-core/src/functions/plugins.rs`. Plugin metadata is stored in KV state under the `plugins` scope.

### Plugin Structure

Each plugin is a JSON object stored in KV:

```json
{
  "id": "my-plugin",
  "name": "My Plugin",
  "version": "1.0.0",
  "description": "Does something useful",
  "author": "Your Name",
  "installed": true,
  "config": {}
}
```

### Managing Plugins

**CLI:**
```bash
rimuru plugins list
rimuru plugins install <plugin-id>
rimuru plugins uninstall <plugin-id>
```

**API:**
```bash
curl http://localhost:3100/api/plugins
curl -X POST http://localhost:3100/api/plugins/install -d '{"plugin_id": "my-plugin"}'
curl -X DELETE http://localhost:3100/api/plugins/my-plugin
```

## Hook System

Hooks trigger functions in response to events (agent connect, session start, cost threshold, etc.).

### Registering Hooks

**CLI:**
```bash
rimuru hooks register --event agent_connected --function my-handler
rimuru hooks dispatch --event agent_connected --payload '{}'
```

**API:**
```bash
curl -X POST http://localhost:3100/api/hooks/register \
  -H 'Content-Type: application/json' \
  -d '{"event_type": "agent_connected", "function_id": "my-handler"}'
```

### Built-in Events

| Event | Description |
|-------|-------------|
| `agent_connected` | Agent discovered and registered |
| `agent_disconnected` | Agent went offline |
| `agent_status_changed` | Agent status update |
| `session_started` | New coding session |
| `session_ended` | Session completed |
| `cost_recorded` | Cost entry recorded |
| `cost_threshold` | Cost exceeded threshold |

## MCP Server Management

Manage MCP (Model Context Protocol) servers:

```bash
rimuru mcp list
rimuru mcp add --command "npx" --args "my-mcp-server"
rimuru mcp remove <server-id>
```
