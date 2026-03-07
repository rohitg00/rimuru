# Cost Tracking

Rimuru tracks token usage and calculates costs across all your AI agents.

## Viewing Costs

### CLI

```bash
rimuru costs summary          # Overall cost summary
rimuru costs daily            # Daily cost breakdown
rimuru costs records          # Raw cost records
```

### API

```bash
curl http://localhost:3100/api/costs/summary
curl http://localhost:3100/api/costs/daily
curl http://localhost:3100/api/costs/records
```

### Web UI

The Costs page shows:
- Total spend with period selection
- Cost breakdown by agent and model
- Daily cost chart
- Individual cost records

### TUI

Press `4` to switch to the Costs tab.

## Cost Summary

The summary breaks down costs by:

- **By Agent**: How much each agent (Claude Code, Cursor, etc.) has cost
- **By Model**: Cost per model (claude-opus-4-6, gpt-4o, etc.)

## Daily Rollups

Costs are automatically rolled up into daily summaries. Each daily record includes:

| Field | Description |
|-------|-------------|
| `date` | The day |
| `total_cost` | Total cost for that day |
| `total_input_tokens` | Input tokens used |
| `total_output_tokens` | Output tokens used |
| `record_count` | Number of sessions |
| `by_agent` | Breakdown per agent |

## Model Pricing

Rimuru includes pricing for 8 models across 5 providers:

| Model | Provider | Input (per 1M) | Output (per 1M) |
|-------|----------|----------------|------------------|
| claude-opus-4-6 | Anthropic | $15.00 | $75.00 |
| claude-sonnet-4-6 | Anthropic | $3.00 | $15.00 |
| claude-haiku-3-5 | Anthropic | $0.80 | $4.00 |
| gpt-4o | OpenAI | $2.50 | $10.00 |
| gpt-4o-mini | OpenAI | $0.15 | $0.60 |
| o3 | OpenAI | $10.00 | $40.00 |
| gemini-2.5-pro | Google | $1.25 | $10.00 |
| gemini-2.5-flash | Google | $0.15 | $0.60 |

Sync pricing updates:

```bash
rimuru models sync
```

## Hardware Advisor

The advisor tells you which API models can run locally on your hardware to save money:

```bash
curl http://localhost:3100/api/models/advisor
```

Each assessment includes fit level (Perfect/Good/Marginal/TooTight), estimated tok/s, and potential savings based on your actual spend.

## Idempotent Records

Cost records use deterministic UUIDs derived from session IDs. Re-syncing the same session overwrites the existing cost record instead of creating duplicates.
