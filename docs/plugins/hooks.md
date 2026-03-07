# Hooks Reference

Hooks trigger functions in response to system events.

## Managing Hooks

### CLI

```bash
rimuru hooks list
rimuru hooks register --event <type> --function <id> [--priority <n>]
rimuru hooks dispatch --event <type> --payload <json>
```

### API

```bash
# List hooks
curl http://localhost:3100/api/hooks

# Register a hook
curl -X POST http://localhost:3100/api/hooks/register \
  -H 'Content-Type: application/json' \
  -d '{"event_type": "agent_connected", "function_id": "my-handler", "priority": 10}'

# Dispatch an event
curl -X POST http://localhost:3100/api/hooks/dispatch \
  -H 'Content-Type: application/json' \
  -d '{"event_type": "agent_connected", "payload": {"agent_id": "..."}}'
```

## Event Types

| Event | Description | Payload |
|-------|-------------|---------|
| `agent_connected` | Agent discovered | `{ agent_id, agent_type, name }` |
| `agent_disconnected` | Agent went offline | `{ agent_id }` |
| `agent_status_changed` | Agent status update | `{ agent_id, status }` |
| `session_started` | New coding session | `{ session_id, agent_id }` |
| `session_ended` | Session completed | `{ session_id, agent_id, cost }` |
| `cost_recorded` | Cost entry recorded | `{ record_id, agent_id, cost }` |
| `cost_threshold` | Cost exceeded threshold | `{ total_cost, threshold }` |
| `metrics_collected` | System metrics updated | `{ cpu, memory }` |

## Hook Properties

| Field | Description |
|-------|-------------|
| `id` | Unique hook identifier |
| `name` | Display name |
| `event_type` | Event to listen for |
| `function_id` | iii function to call |
| `priority` | Execution order (lower = first) |
| `plugin_id` | Associated plugin (optional) |
| `matcher` | Event filter pattern (optional) |

## Web UI

The Hooks page shows registered hooks with event types and function bindings.

## TUI

Press `8` to switch to the Hooks tab.
