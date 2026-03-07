# Sessions

Sessions represent individual coding interactions with AI agents.

## Viewing Sessions

### CLI

```bash
rimuru sessions list
rimuru sessions list --active
rimuru sessions get <session-id>
```

### API

```bash
curl http://localhost:3100/api/sessions
curl http://localhost:3100/api/sessions/active
curl http://localhost:3100/api/sessions/<id>
```

### Web UI

The Sessions page shows a table of all sessions with agent, duration, token usage, and cost.

### TUI

Press `3` to switch to the Sessions tab.

## Session Data

Each session includes:

| Field | Description |
|-------|-------------|
| `id` | Unique session identifier |
| `agent_id` | Which agent this session belongs to |
| `status` | active, completed, cancelled, failed |
| `started_at` | Session start time |
| `ended_at` | Session end time (if completed) |
| `input_tokens` | Total input tokens used |
| `output_tokens` | Total output tokens used |
| `total_cost` | Calculated cost in USD |
| `model` | Model used during the session |

## How Sessions Are Tracked

Sessions are discovered by reading each agent's local storage:

- **Claude Code**: Reads from `~/.claude/` project directories
- **Cursor**: Reads from Cursor's application support directory
- **Copilot**: Reads from VS Code extension data
- **Codex**: Reads from `~/.config/codex/`

The worker syncs sessions periodically and calculates costs using the model pricing catalog.

## Cost Records

Each session generates a cost record with idempotent UUIDs. Re-syncing the same session overwrites the existing record instead of creating duplicates.
