# Plugin Development

Rimuru plugins extend functionality through iii-engine functions and KV state.

## Plugin Architecture

Plugins are managed via iii functions in `crates/rimuru-core/src/functions/plugins.rs`. Plugin metadata is stored in KV state under the `plugins` scope.

## Creating a Plugin

### 1. Define Plugin Metadata

```json
{
  "id": "my-plugin",
  "name": "My Plugin",
  "version": "1.0.0",
  "description": "Extends Rimuru with custom functionality",
  "author": "Your Name"
}
```

### 2. Register via API

```bash
curl -X POST http://localhost:3100/api/plugins/install \
  -H 'Content-Type: application/json' \
  -d '{"plugin_id": "my-plugin", "name": "My Plugin", "version": "1.0.0"}'
```

## Plugin Management

```bash
rimuru plugins list
rimuru plugins install <id>
rimuru plugins uninstall <id>
```

## Hook Integration

Plugins can register hooks to respond to system events. See [hooks.md](hooks.md) for the event reference.

## MCP Server Plugins

Add MCP (Model Context Protocol) servers:

```bash
rimuru mcp add --command "npx" --args "my-mcp-server"
rimuru mcp list
rimuru mcp remove <id>
```
