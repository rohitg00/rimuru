# CLI Reference

## Global Options

```bash
rimuru [OPTIONS] <COMMAND>
```

| Option | Description | Default |
|--------|-------------|---------|
| `--engine-url` | iii engine WebSocket URL | `ws://127.0.0.1:49134` |
| `--format` | Output format: `table`, `json` | `table` |
| `--help` / `-h` | Print help |  |
| `--version` / `-V` | Print version |  |

## Commands

### agents

```bash
rimuru agents list              # List all agents
rimuru agents sync              # Trigger agent discovery
rimuru agents get <id>          # Get agent details
```

### sessions

```bash
rimuru sessions list            # List all sessions
rimuru sessions list --active   # List active sessions
rimuru sessions get <id>        # Get session details
```

### costs

```bash
rimuru costs summary            # Cost summary with breakdowns
rimuru costs daily              # Daily cost rollups
rimuru costs records            # Raw cost records
rimuru costs record             # Record a cost entry manually
```

### models

```bash
rimuru models list              # List all models with pricing
rimuru models sync              # Sync model pricing
rimuru models get <id>          # Get model details
```

### metrics

```bash
rimuru metrics current          # Current system metrics
rimuru metrics history          # Metrics history
```

### plugins

```bash
rimuru plugins list             # List plugins
rimuru plugins install <id>     # Install a plugin
rimuru plugins uninstall <id>   # Uninstall a plugin
```

### hooks

```bash
rimuru hooks list               # List registered hooks
rimuru hooks register           # Register a hook
  --event <type>                # Event type to listen for
  --function <id>               # Function to call
  --priority <n>                # Priority (optional)
rimuru hooks dispatch           # Dispatch an event
  --event <type>                # Event type
  --payload <json>              # Event payload
```

### mcp

```bash
rimuru mcp list                 # List MCP servers
rimuru mcp add                  # Add MCP server
  --command <cmd>               # Server command
  --args <args>                 # Command arguments
rimuru mcp remove <id>          # Remove MCP server
```

### config

```bash
rimuru config get               # Get current configuration
rimuru config set <key> <val>   # Set configuration value
```

### health

```bash
rimuru health                   # Health check
```

### ui

```bash
rimuru ui                       # Open Web UI in browser
  --port <port>                 # Port (default: 3100)
```

## Output Formats

Use `--format json` for machine-readable output:

```bash
rimuru agents list --format json
rimuru costs summary --format json
```
