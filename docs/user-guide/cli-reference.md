---
type: reference
title: CLI Reference
created: 2026-02-05
tags:
  - cli
  - commands
  - reference
related:
  - "[[getting-started]]"
  - "[[agents]]"
  - "[[sessions]]"
  - "[[costs]]"
---

# CLI Reference

Complete reference for all Rimuru CLI commands.

## Global Options

```bash
rimuru [OPTIONS] <COMMAND>
```

| Option | Short | Description |
|--------|-------|-------------|
| `--verbose` | `-v` | Enable verbose/debug output |
| `--help` | `-h` | Print help information |
| `--version` | `-V` | Print version information |

## Commands Overview

| Command | Description |
|---------|-------------|
| `init` | Initialize the database |
| `status` | Show system metrics |
| `agents` | Manage agents |
| `sessions` | Manage sessions |
| `costs` | Track costs |
| `sync` | Sync model pricing |
| `models` | List models |
| `skills` | Manage SkillKit skills |
| `plugins` | Manage plugins |
| `hooks` | Manage hooks |
| `version` | Show version info |

---

## init

Initialize the database and run migrations.

```bash
rimuru init [OPTIONS]
```

| Option | Short | Description |
|--------|-------|-------------|
| `--force` | `-f` | Force re-initialization |

**Examples:**

```bash
rimuru init
rimuru init --force
```

---

## status

Show current system metrics.

```bash
rimuru status [OPTIONS]
```

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--detailed` | `-d` | Show extended metrics | false |
| `--format` | `-f` | Output format (text, json) | text |

**Examples:**

```bash
rimuru status
rimuru status --detailed
rimuru status --format json
```

---

## agents

Manage registered AI coding agents.

```bash
rimuru agents [SUBCOMMAND]
```

### agents (list)

List all agents (default when no subcommand).

```bash
rimuru agents [OPTIONS]
```

| Option | Short | Description |
|--------|-------|-------------|
| `--type` | `-t` | Filter by agent type |
| `--format` | `-f` | Output format (text, json) |

### agents show

Show details for a specific agent.

```bash
rimuru agents show <AGENT_ID>
```

### agents count

Count agents by type.

```bash
rimuru agents count
```

**Examples:**

```bash
rimuru agents
rimuru agents --type claude-code
rimuru agents show a1b2c3d4-e5f6-7890-abcd-ef1234567890
rimuru agents count
rimuru agents --format json
```

---

## sessions

Manage and view sessions across all agents.

```bash
rimuru sessions [SUBCOMMAND]
```

### sessions (list)

List sessions (default).

```bash
rimuru sessions [OPTIONS]
```

| Option | Short | Description |
|--------|-------|-------------|
| `--agent-type` | `-t` | Filter by agent type |
| `--agent` | `-a` | Filter by agent name |
| `--format` | `-f` | Output format (text, json) |
| `--active` | | Show only active sessions |

### sessions history

Show completed sessions.

```bash
rimuru sessions history [OPTIONS]
```

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--limit` | `-l` | Number of sessions | 10 |
| `--range` | `-r` | Time range | 7d |
| `--agent-type` | `-t` | Filter by agent type | |
| `--format` | `-f` | Output format | text |

### sessions stats

Show session statistics.

```bash
rimuru sessions stats [OPTIONS]
```

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--range` | `-r` | Time range | 7d |
| `--format` | `-f` | Output format | text |

### sessions show

Show details of a specific session.

```bash
rimuru sessions show <SESSION_ID>
```

**Examples:**

```bash
rimuru sessions
rimuru sessions --active
rimuru sessions --agent-type claude-code
rimuru sessions history --limit 20 --range 30d
rimuru sessions stats --range today
rimuru sessions show a1b2c3d4-e5f6-7890-abcd-ef1234567890
```

---

## costs

Track and analyze costs across all agents.

```bash
rimuru costs [SUBCOMMAND]
```

### costs today

Show today's costs (default).

```bash
rimuru costs today [OPTIONS]
```

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--format` | `-f` | Output format | text |

### costs summary

Show cost summary over a time period.

```bash
rimuru costs summary [OPTIONS]
```

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--range` | `-r` | Time range | 7d |
| `--format` | `-f` | Output format | text |

### costs by-model

Breakdown costs by model.

```bash
rimuru costs by-model [OPTIONS]
```

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--range` | `-r` | Time range | 7d |
| `--format` | `-f` | Output format | text |

### costs agent

Show costs for a specific agent.

```bash
rimuru costs agent <AGENT_NAME> [OPTIONS]
```

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--range` | `-r` | Time range | 7d |
| `--format` | `-f` | Output format | text |

**Examples:**

```bash
rimuru costs
rimuru costs today
rimuru costs summary --range 30d
rimuru costs by-model --range all
rimuru costs agent claude-code-main --range 7d
```

---

## sync

Sync model pricing from providers.

```bash
rimuru sync [SUBCOMMAND]
```

### sync run

Run model pricing sync.

```bash
rimuru sync run [OPTIONS]
```

| Option | Short | Description |
|--------|-------|-------------|
| `--provider` | `-p` | Sync specific provider only |
| `--force` | `-f` | Force sync even if recent |

### sync status

Show sync status.

```bash
rimuru sync status
```

**Examples:**

```bash
rimuru sync run
rimuru sync run --provider anthropic
rimuru sync status
```

---

## models

List and search models with pricing.

```bash
rimuru models [SUBCOMMAND]
```

### models (list)

List all models (default).

```bash
rimuru models [OPTIONS]
```

| Option | Short | Description |
|--------|-------|-------------|
| `--provider` | `-p` | Filter by provider |
| `--format` | `-f` | Output format |

### models search

Search for models.

```bash
rimuru models search <QUERY>
```

### models show

Show model details.

```bash
rimuru models show <MODEL_NAME>
```

**Examples:**

```bash
rimuru models
rimuru models --provider openai
rimuru models search gpt-4
rimuru models show claude-opus-4-5-20251101
```

---

## skills

Manage skills from SkillKit marketplace.

```bash
rimuru skills [SUBCOMMAND]
```

### skills search

Search marketplace for skills.

```bash
rimuru skills search <QUERY> [OPTIONS]
```

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--agent` | `-a` | Filter by agent type | |
| `--tags` | `-t` | Filter by tags (comma-separated) | |
| `--format` | `-f` | Output format | text |
| `--limit` | `-l` | Number of results | 20 |

### skills install

Install a skill.

```bash
rimuru skills install <NAME> [OPTIONS]
```

| Option | Short | Description |
|--------|-------|-------------|
| `--agent` | `-a` | Target agent type |
| `--all` | | Install for all agents |
| `--format` | `-f` | Output format |

### skills uninstall

Uninstall a skill.

```bash
rimuru skills uninstall <NAME>
```

### skills translate

Translate a skill between agents.

```bash
rimuru skills translate <NAME> --from <AGENT> --to <AGENT>
```

### skills list

List installed skills.

```bash
rimuru skills list [OPTIONS]
```

| Option | Short | Description |
|--------|-------|-------------|
| `--agent` | `-a` | Filter by agent |
| `--format` | `-f` | Output format |
| `--enabled` | | Show only enabled |
| `--disabled` | | Show only disabled |

### skills recommend

Get skill recommendations.

```bash
rimuru skills recommend [OPTIONS]
```

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--workflow` | `-w` | Workflow description | |
| `--format` | `-f` | Output format | text |
| `--limit` | `-l` | Number of recommendations | 10 |

### skills publish

Publish a skill to marketplace.

```bash
rimuru skills publish <PATH> [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--format` | Output format |
| `--skip-validation` | Skip validation |

### skills show

Show skill details.

```bash
rimuru skills show <NAME> [OPTIONS]
```

### skills sync

Sync with marketplace.

```bash
rimuru skills sync
```

### skills stats

Show marketplace statistics.

```bash
rimuru skills stats [OPTIONS]
```

### skills agents

List supported agents.

```bash
rimuru skills agents [OPTIONS]
```

**Examples:**

```bash
rimuru skills search "react"
rimuru skills install react-useeffect --agent claude-code
rimuru skills list --enabled
rimuru skills translate pro-workflow --from claude-code --to cursor
rimuru skills recommend --workflow "building REST API"
```

---

## plugins

Manage plugins.

```bash
rimuru plugins [SUBCOMMAND]
```

### plugins list

List installed plugins (default).

```bash
rimuru plugins list [OPTIONS]
```

| Option | Short | Description |
|--------|-------|-------------|
| `--all` | `-a` | Include disabled plugins |
| `--format` | `-f` | Output format |
| `--capability` | `-c` | Filter by capability |

### plugins install

Install a plugin.

```bash
rimuru plugins install <SOURCE> [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--enable` | Enable after install |

### plugins enable

Enable a plugin.

```bash
rimuru plugins enable <NAME>
```

### plugins disable

Disable a plugin.

```bash
rimuru plugins disable <NAME>
```

### plugins uninstall

Uninstall a plugin.

```bash
rimuru plugins uninstall <NAME> [OPTIONS]
```

| Option | Description |
|--------|-------------|
| `--force` | Skip confirmation |

### plugins info

Show plugin information.

```bash
rimuru plugins info <NAME>
```

### plugins config

Configure plugin settings.

```bash
rimuru plugins config <NAME> [OPTIONS]
```

| Option | Short | Description |
|--------|-------|-------------|
| `--key` | `-k` | Setting key |
| `--value` | `-v` | Setting value |
| `--show` | | Show current config |

### plugins builtin

Show built-in plugins.

```bash
rimuru plugins builtin
```

**Examples:**

```bash
rimuru plugins
rimuru plugins list --all
rimuru plugins install ./my-plugin --enable
rimuru plugins enable slack-notifier
rimuru plugins config slack-notifier --key webhook_url --value "https://..."
rimuru plugins info csv-exporter
rimuru plugins builtin
```

---

## hooks

Manage hooks and view execution history.

```bash
rimuru hooks [SUBCOMMAND]
```

### hooks list

List registered hooks.

```bash
rimuru hooks list [OPTIONS]
```

### hooks history

Show hook execution history.

```bash
rimuru hooks history [OPTIONS]
```

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--limit` | `-l` | Number of entries | 20 |

---

## version

Show version information.

```bash
rimuru version [OPTIONS]
```

| Option | Short | Description |
|--------|-------|-------------|
| `--detailed` | `-d` | Show build info and supported agents |

**Examples:**

```bash
rimuru version
rimuru version --detailed
```

---

## Time Range Values

Commands that accept `--range` support these values:

| Value | Description |
|-------|-------------|
| `today` | Current day |
| `yesterday` | Previous day |
| `7d`, `week` | Last 7 days |
| `30d`, `month` | Last 30 days |
| `thismonth` | Current calendar month |
| `lastmonth` | Previous calendar month |
| `all`, `alltime` | All time |

---

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection URL | Required |
| `RIMURU_LOG_LEVEL` | Log level | `info` |
| `RIMURU_METRICS_INTERVAL` | Metrics interval (seconds) | `5` |
| `RIMURU_METRICS_STORE_TO_DB` | Store metrics to database | `true` |
| `RIMURU_METRICS_RETENTION_DAYS` | Metrics retention | `30` |
| `RIMURU_AGENT_TIMEOUT` | Agent connection timeout | `30` |
| `RIMURU_AGENT_AUTO_DISCOVER` | Auto-discover agents | `true` |
| `RIMURU_SYNC_ENABLED` | Enable model sync | `true` |
| `RIMURU_SYNC_INTERVAL_HOURS` | Sync interval | `6` |
| `RIMURU_COLOR` | Enable colored output | `true` |
| `RIMURU_DATETIME_FORMAT` | Date/time format | `%Y-%m-%d %H:%M:%S` |
| `RIMURU_CURRENCY` | Currency for costs | `USD` |
| `ANTHROPIC_API_KEY` | Anthropic API key | |
| `OPENAI_API_KEY` | OpenAI API key | |
| `GOOGLE_API_KEY` | Google AI API key | |
| `OPENROUTER_API_KEY` | OpenRouter API key | |

---

## Exit Codes

| Code | Description |
|------|-------------|
| `0` | Success |
| `1` | General error |
| `2` | Command line usage error |

---

## Related Topics

- [[getting-started]] - Initial setup
- [[keyboard-shortcuts]] - TUI keybindings
