# Agents

Rimuru auto-discovers and monitors AI coding agents installed on your system.

## Supported Agents

| Agent | Type ID | Discovery Path | Provider |
|-------|---------|----------------|----------|
| Claude Code | `claude_code` | `~/.claude/` | Anthropic |
| Cursor | `cursor` | `~/Library/Application Support/Cursor/` | OpenAI |
| GitHub Copilot | `copilot` | VS Code extension storage | OpenAI |
| Codex | `codex` | `~/.config/codex/` | OpenAI |
| Goose | `goose` | `~/.config/goose/` | Various |
| OpenCode | `opencode` | `~/.opencode/` | Various |

## Agent Discovery

On startup, the worker scans for installed agents and stores them in KV state. Discovery runs automatically on a schedule.

### Manual Sync

```bash
rimuru agents sync
```

Or via API:

```bash
curl -X POST http://localhost:3100/api/agents/sync
```

## Viewing Agents

### CLI

```bash
rimuru agents list
rimuru agents get <agent-id>
```

### Web UI

The Agents page shows cards for each discovered agent with status indicators, session counts, and cost totals.

### TUI

Press `2` to switch to the Agents tab.

## Agent Status

| Status | Description |
|--------|-------------|
| `active` | Agent is running with active sessions |
| `idle` | Agent is installed but no active sessions |
| `disconnected` | Agent was previously seen but is now unavailable |
| `error` | Agent discovery encountered an error |

## Tempest City

The Web UI includes a pixel-art city view where each agent is represented as an anime character from Tensura:

| Agent | Character |
|-------|-----------|
| Claude Code | Rimuru |
| Cursor | Shion |
| Codex | Benimaru |
| Gemini CLI | Shuna |
| OpenCode | Souei |
| Other | Gabiru |

Characters walk between districts based on their status (busy agents go to Great Hall, idle agents to Scholar's Quarter).
