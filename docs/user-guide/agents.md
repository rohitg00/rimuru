---
type: reference
title: Managing Agents
created: 2026-02-05
tags:
  - agents
  - configuration
  - adapters
related:
  - "[[getting-started]]"
  - "[[sessions]]"
  - "[[costs]]"
---

# Managing Agents

Rimuru manages multiple AI coding agents from a single interface. This guide covers supported agents, configuration, and management commands.

## Supported Agents

| Agent | Icon | Type ID | Description |
|-------|------|---------|-------------|
| Claude Code | ⟁ | `claude-code` | Anthropic's Claude-powered coding assistant |
| Codex | ◎ | `codex` | OpenAI's Codex models (GPT-4, o1, o3) |
| GitHub Copilot | ◈ | `copilot` | GitHub's AI pair programmer |
| Goose | ⬡ | `goose` | Block's autonomous AI agent |
| OpenCode | ◇ | `opencode` | Open-source terminal coding assistant |
| Cursor | ◫ | `cursor` | AI-first code editor |

## Agent Discovery

Rimuru automatically discovers running agents when started with auto-discovery enabled:

```ini
RIMURU_AGENT_AUTO_DISCOVER=true
```

Discovery checks:
- Running processes matching known agent patterns
- Configuration files in standard locations
- Active sessions from agent data stores

## CLI Commands

### List Agents

View all registered agents:

```bash
rimuru agents
```

Example output:

```
Registered Agents
┌────────────────────┬─────────────┬──────────┬────────────────────┐
│ Name               │ Type        │ Status   │ Last Active        │
├────────────────────┼─────────────┼──────────┼────────────────────┤
│ claude-code-main   │ ⟁ Claude    │ Active   │ 2026-02-05 14:30   │
│ cursor-default     │ ◫ Cursor    │ Inactive │ 2026-02-04 18:15   │
│ copilot-vscode     │ ◈ Copilot   │ Active   │ 2026-02-05 14:28   │
└────────────────────┴─────────────┴──────────┴────────────────────┘
```

### Show Agent Details

View detailed information for a specific agent:

```bash
rimuru agents show <agent-id>
```

Displays:
- Agent name and type
- Configuration path
- Connection status
- Active sessions
- Total usage statistics
- Cost summary

### Count Agents by Type

Get agent counts grouped by type:

```bash
rimuru agents count
```

### Filter by Type

List only agents of a specific type:

```bash
rimuru agents --type claude-code
rimuru agents --type copilot
```

### JSON Output

For scripting and automation:

```bash
rimuru agents --format json
```

## Agent Configuration

Each agent type has specific configuration requirements:

### Claude Code

Configuration location: `~/.claude/`

```toml
[claude_code]
config_path = "~/.claude"
session_store = "~/.claude/sessions"
cost_tracking = true
```

Environment variables:
- `ANTHROPIC_API_KEY` - Required for cost calculation
- `CLAUDE_CODE_SESSION_PATH` - Custom session storage path

### Codex (OpenAI)

Configuration location: `~/.codex/`

```toml
[codex]
config_path = "~/.codex"
model_preference = "gpt-4"
```

Environment variables:
- `OPENAI_API_KEY` - Required for cost calculation

### GitHub Copilot

Configuration location: Varies by IDE

VS Code: `~/.config/Code/User/globalStorage/github.copilot`
JetBrains: `~/.config/JetBrains/<IDE>/copilot`

```toml
[copilot]
vscode_path = "~/.config/Code/User/globalStorage/github.copilot"
jetbrains_path = "~/.config/JetBrains"
```

### Goose

Configuration location: `~/.config/goose/`

```toml
[goose]
config_path = "~/.config/goose"
session_store = "~/.config/goose/sessions"
```

### OpenCode

Configuration location: `~/.opencode/`

```toml
[opencode]
config_path = "~/.opencode"
state_file = "~/.opencode/state.json"
```

### Cursor

Configuration location: `~/.cursor/`

```toml
[cursor]
config_path = "~/.cursor"
session_store = "~/.cursor/sessions"
```

## Configuration File

Create `config/local.toml` for custom agent settings:

```toml
[agents]
timeout_secs = 30
auto_discover = true
health_check_interval = 60

[agents.claude_code]
enabled = true
config_path = "~/.claude"

[agents.codex]
enabled = true
config_path = "~/.codex"

[agents.copilot]
enabled = true

[agents.goose]
enabled = false

[agents.opencode]
enabled = true

[agents.cursor]
enabled = true
```

## Agent Adapters

Rimuru uses adapters to communicate with each agent type. Adapters handle:

- **Discovery**: Finding running agent instances
- **Session Monitoring**: Tracking active sessions
- **Cost Tracking**: Calculating usage costs
- **Health Checks**: Verifying agent availability

### Adapter Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Adapter Manager                          │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │ Claude Code │  │   Codex     │  │   Copilot   │  ...    │
│  │   Adapter   │  │   Adapter   │  │   Adapter   │         │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘         │
│         │                │                │                 │
│         └────────────────┼────────────────┘                 │
│                          ▼                                  │
│              ┌─────────────────────┐                        │
│              │  Agent Repository   │                        │
│              └─────────────────────┘                        │
└─────────────────────────────────────────────────────────────┘
```

### Adapter Traits

Each adapter implements:

```rust
pub trait AgentAdapter: Send + Sync {
    fn name(&self) -> &str;
    fn agent_type(&self) -> AgentType;
    async fn connect(&self) -> Result<(), AdapterError>;
    async fn disconnect(&self) -> Result<(), AdapterError>;
    async fn is_connected(&self) -> bool;
    async fn health_check(&self) -> Result<(), AdapterError>;
}

pub trait CostTracker: Send + Sync {
    async fn get_current_cost(&self) -> Result<CostRecord, AdapterError>;
    async fn get_cost_history(&self, range: TimeRange) -> Result<Vec<CostRecord>, AdapterError>;
}

pub trait SessionMonitor: Send + Sync {
    async fn get_active_sessions(&self) -> Result<Vec<Session>, AdapterError>;
    async fn get_session_history(&self, limit: Option<usize>) -> Result<Vec<Session>, AdapterError>;
}
```

## Troubleshooting

### Agent Not Discovered

1. Verify the agent is running
2. Check configuration paths are correct
3. Ensure auto-discovery is enabled
4. Try manual registration

### Connection Failures

1. Check agent process is running
2. Verify configuration file permissions
3. Review logs: `RIMURU_LOG_LEVEL=debug rimuru agents`

### Missing Cost Data

1. Ensure API keys are configured
2. Check model sync is enabled
3. Verify agent is reporting usage

## Related Topics

- [[sessions]] - View and manage agent sessions
- [[costs]] - Track costs across agents
- [[cli-reference]] - Complete CLI command reference
