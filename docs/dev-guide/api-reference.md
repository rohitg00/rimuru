# API Reference

The rimuru-worker serves a REST API at `http://localhost:3100`.

## Agents

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/agents` | List all agents |
| GET | `/api/agents/:id` | Get agent by ID |
| POST | `/api/agents/sync` | Trigger agent discovery |
| GET | `/api/agents/:id/sessions` | Get agent sessions |

## Sessions

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/sessions` | List all sessions |
| GET | `/api/sessions/:id` | Get session by ID |
| GET | `/api/sessions/active` | List active sessions |

## Costs

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/costs/summary` | Cost summary with breakdowns |
| GET | `/api/costs/daily` | Daily cost rollups |
| GET | `/api/costs/records` | Raw cost records |
| POST | `/api/costs/record` | Record a cost entry |

## Models

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/models` | List all models with pricing |
| GET | `/api/models/:id` | Get model by ID |
| POST | `/api/models/sync` | Sync model pricing |
| GET | `/api/models/advisor` | Hardware advisor assessments |

## Metrics

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/metrics` | Current system metrics |
| GET | `/api/metrics/history` | Metrics history |

## Plugins

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/plugins` | List plugins |
| POST | `/api/plugins/install` | Install a plugin |
| DELETE | `/api/plugins/:id` | Uninstall a plugin |

## Hooks

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/hooks` | List hooks |
| POST | `/api/hooks/register` | Register a hook |
| POST | `/api/hooks/dispatch` | Dispatch an event |

## MCP

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/mcp/servers` | List MCP servers |
| POST | `/api/mcp/servers` | Add MCP server |
| DELETE | `/api/mcp/servers/:id` | Remove MCP server |

## System

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/health` | Health check |
| GET | `/api/config` | Get configuration |
| PUT | `/api/config` | Update configuration |
| GET | `/api/system` | Hardware info |
| POST | `/api/system/detect` | Detect hardware |
| GET | `/api/activity` | Activity feed |

## Streaming

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/stream` | SSE event stream (real-time updates) |

## Response Format

All endpoints return JSON. Example:

```json
{
  "agents": [
    {
      "id": "uuid",
      "name": "Claude Code",
      "agent_type": "claude_code",
      "status": "active",
      "session_count": 5,
      "total_cost": 1.23
    }
  ]
}
```

## iii Functions

Each API endpoint maps to an iii function (e.g., `GET /api/agents` calls `rimuru.agents.list`). The CLI connects directly to the iii engine and calls these functions via WebSocket.
