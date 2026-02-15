---
type: reference
title: Cost Tracking
created: 2026-02-05
tags:
  - costs
  - pricing
  - budgets
  - reporting
related:
  - "[[sessions]]"
  - "[[agents]]"
  - "[[cli-reference]]"
---

# Cost Tracking

Rimuru provides comprehensive cost tracking across all AI agents. Track spending, analyze usage patterns, and generate reports for budgeting.

## How Costs Are Calculated

Costs are calculated using the formula:

```
Total Cost = (Input Tokens × Input Price) + (Output Tokens × Output Price)
```

Model pricing is synchronized from provider APIs and updated regularly.

## CLI Commands

### Today's Costs

View costs for the current day:

```bash
rimuru costs today
```

Example output:

```
Today's Costs
Date: 2026-02-05

┌────────────────────┬─────────────┬──────────┬──────────────┬───────────────┬──────────┐
│ Agent              │ Type        │ Requests │ Input Tokens │ Output Tokens │ Cost     │
├────────────────────┼─────────────┼──────────┼──────────────┼───────────────┼──────────┤
│ ⟁ claude-code-main │ Claude Code │ 45       │ 125.3K       │ 48.2K         │ $0.4521  │
│ ◫ cursor-default   │ Cursor      │ 23       │ 62.1K        │ 21.5K         │ $0.1893  │
│ ◈ copilot-vscode   │ Copilot     │ 156      │ 31.2K        │ 12.8K         │ $0.0892  │
└────────────────────┴─────────────┴──────────┴──────────────┴───────────────┴──────────┘

  Total
    Requests:       224
    Input Tokens:   218.6K
    Output Tokens:  82.5K
    Total Cost:     $0.7306
```

### Cost Summary

View cost summary for a time period:

```bash
rimuru costs summary
rimuru costs summary --range 7d
rimuru costs summary --range 30d
rimuru costs summary --range all
```

Summary includes:
- Total requests
- Total input/output tokens
- Input and output costs separately
- Total cost
- Average cost per request
- Average tokens per request

### Costs by Model

Breakdown costs by AI model:

```bash
rimuru costs by-model
rimuru costs by-model --range 30d
```

Output shows which models are consuming the most resources:

```
Costs by Model
Time range: Last 30 days

┌──────────────────────────┬──────────┬──────────────┬───────────────┬────────────┬─────────────┬────────────┐
│ Model                    │ Requests │ Input Tokens │ Output Tokens │ Input Cost │ Output Cost │ Total Cost │
├──────────────────────────┼──────────┼──────────────┼───────────────┼────────────┼─────────────┼────────────┤
│ claude-opus-4-5-20251101 │ 234      │ 1.2M         │ 456.3K        │ $18.0000   │ $34.2225    │ $52.2225   │
│ claude-sonnet-4-5-2025…  │ 1,456    │ 3.8M         │ 892.1K        │ $11.4000   │ $13.3815    │ $24.7815   │
│ gpt-4-turbo              │ 892      │ 2.1M         │ 521.4K        │ $21.0000   │ $15.6420    │ $36.6420   │
│ gpt-4o                   │ 2,341    │ 5.2M         │ 1.1M          │ $13.0000   │ $16.5000    │ $29.5000   │
└──────────────────────────┴──────────┴──────────────┴───────────────┴────────────┴─────────────┴────────────┘

  Summary: 4 models, $143.1460 total cost
```

### Agent-Specific Costs

View costs for a specific agent:

```bash
rimuru costs agent claude-code-main
rimuru costs agent cursor-default --range 7d
```

Displays detailed breakdown for that agent including:
- Agent name and type
- Model used
- Request count
- Token breakdown
- Cost breakdown

### Time Ranges

All cost commands support these time ranges:

| Range | Description |
|-------|-------------|
| `today` | Current day |
| `yesterday` | Previous day |
| `7d` or `week` | Last 7 days |
| `30d` or `month` | Last 30 days |
| `thismonth` | Current calendar month |
| `lastmonth` | Previous calendar month |
| `all` | All time |

### JSON Output

For scripting, export, or integration:

```bash
rimuru costs today --format json
rimuru costs summary --range 30d --format json
rimuru costs by-model --format json
```

## Model Pricing Sync

Rimuru synchronizes model pricing from providers:

### Manual Sync

```bash
rimuru sync run
```

### Sync Status

```bash
rimuru sync status
```

### List Synced Models

```bash
rimuru models
rimuru models --provider anthropic
```

### Sync Configuration

Configure sync in `.env` or `config/local.toml`:

```ini
# Enable automatic sync
RIMURU_SYNC_ENABLED=true

# Sync interval in hours
RIMURU_SYNC_INTERVAL_HOURS=6
```

### API Keys for Sync

For accurate pricing, configure API keys:

```ini
# Anthropic (Claude models)
ANTHROPIC_API_KEY=sk-ant-...

# OpenAI (GPT, o1/o3 models)
OPENAI_API_KEY=sk-...

# Google (Gemini models)
GOOGLE_API_KEY=...

# OpenRouter (aggregated pricing)
OPENROUTER_API_KEY=sk-or-...
```

Without API keys, Rimuru uses known model lists with default pricing.

## TUI Cost View

In the TUI (`rimuru-tui`), press `4` to access the Costs view:

```
┌─ Costs (Last 7 Days) ───────────────────────────────────────┐
│                                                              │
│  Total Cost: $24.56                                          │
│  ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓░░░░░░░░░░░░░░░░░░░░  Budget: $50.00    │
│                                                              │
│  By Agent Type:                                              │
│    ⟁ Claude Code   ████████████████     $15.23  (62%)       │
│    ◫ Cursor        ██████               $5.89   (24%)       │
│    ◈ Copilot       ███                  $2.34   (10%)       │
│    ◎ Codex         █                    $1.10   (4%)        │
│                                                              │
│  Daily Trend:                                                │
│  $8 ┤      ▄▄                                                │
│  $6 ┤   ▄▄▀██▄                                               │
│  $4 ┤  ▀█████▀▄                                              │
│  $2 ┤ ▀███████▀▄                                             │
│  $0 └──────────────                                          │
│      Mon Tue Wed Thu Fri Sat Sun                             │
└──────────────────────────────────────────────────────────────┘
```

Navigation:
- `Tab`: Cycle between summary, by-agent, by-model views
- `r`: Change time range
- `Enter`: View detailed breakdown
- `/`: Search agents/models

## Cost Reports

### Export to CSV

Use the CSV exporter plugin:

```bash
rimuru plugins enable csv-exporter
rimuru costs summary --format json | rimuru-export csv > costs.csv
```

### Export to JSON

```bash
rimuru costs summary --range 30d --format json > costs-report.json
```

## Cost Data Structure

Rimuru stores costs in PostgreSQL:

```sql
CREATE TABLE cost_records (
    id UUID PRIMARY KEY,
    session_id UUID REFERENCES sessions(id),
    agent_id UUID REFERENCES agents(id),
    recorded_at TIMESTAMPTZ NOT NULL,
    model_name VARCHAR(255) NOT NULL,
    input_tokens BIGINT NOT NULL,
    output_tokens BIGINT NOT NULL,
    input_cost DECIMAL(10, 8) NOT NULL,
    output_cost DECIMAL(10, 8) NOT NULL,
    total_cost DECIMAL(10, 6) NOT NULL
);
```

## Best Practices

### Monitor Daily Costs

Set up a daily check:

```bash
rimuru costs today --format json | jq '.total_cost'
```

### Track by Agent Type

Identify which agent types consume the most:

```bash
rimuru costs summary --range 7d
```

### Optimize Model Usage

Review model costs to identify optimization opportunities:

```bash
rimuru costs by-model --range 30d
```

Consider:
- Using smaller models for simple tasks
- Reducing context size where possible
- Batching requests when appropriate

## Related Topics

- [[sessions]] - Session tracking and history
- [[agents]] - Agent management
- [[plugins]] - Export and notification plugins
- [[cli-reference]] - Complete CLI reference
