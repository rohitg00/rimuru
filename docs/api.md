# HTTP API Reference

Endpoints are served by `rimuru-worker` via `iii-http` on `:3111`. Routes are registered from the central table in [`crates/rimuru-core/src/triggers/api.rs`](../crates/rimuru-core/src/triggers/api.rs) -- treat that file as the source of truth, this document as a navigable summary.

## Versioning

Every route is registered twice:

- **Canonical:** `/api/v1/<path>` — the versioned route. Use this for any new integration. The `v1` prefix is locked: schemas under `/api/v1/` will only break on a future major bump (and that bump will land at `/api/v2/` first, with `/api/v1/` kept as an alias for at least one minor release).
- **Legacy alias:** `/api/<path>` — same handler, no version prefix. Preserved for the CLI, the bundled web UI, and any pre-v0.4 external client. New external integrations should prefer `/api/v1/`; the unversioned alias may be dropped on a future major bump after a deprecation window.

The two routes resolve to the same iii function, so they always return identical bodies. `API_VERSION` in `crates/rimuru-core/src/triggers/api.rs` is the single source of truth for the version segment — change it there to bump.

The tables below list each route under its unversioned form for brevity. Every entry is also reachable at `/api/v1/<path>`.

## Response shape

Every handler returns `api_response(data)` which wraps the payload as `{"status_code": 200, "body": <data>}`. The CLI's `unwrap_body()` unpacks this for 2xx responses. The `extract_input()` helper merges path params, query params, and JSON body into one `Value` before dispatch, so the handler treats all three as a single input map.

## HTTP vs iii trigger

The HTTP layer exposes the **public surface**: agents, sessions, costs, context, models, advisor, metrics, health, MCP proxy, hooks, plugins, config, **budget**, **runaway**, and **guard**. The CLI calls all of these via `iii.trigger()` directly to skip the HTTP hop, but external clients can use the HTTP routes documented below. See [function namespaces in the README](../README.md#api).

## Agents

| Method | Path | Function |
|--------|------|----------|
| GET    | `/api/agents`                  | `rimuru.agents.list`        |
| GET    | `/api/agents/:id`              | `rimuru.agents.get`         |
| POST   | `/api/agents`                  | `rimuru.agents.create`      |
| POST   | `/api/agents/connect`          | `rimuru.agents.connect`     |
| POST   | `/api/agents/:id/disconnect`   | `rimuru.agents.disconnect`  |
| GET    | `/api/agents/detect`           | `rimuru.agents.detect`      |

## Sessions

| Method | Path | Function |
|--------|------|----------|
| GET    | `/api/sessions`           | `rimuru.sessions.list`    |
| GET    | `/api/sessions/:id`       | `rimuru.sessions.get`     |
| GET    | `/api/sessions/active`    | `rimuru.sessions.active`  |
| GET    | `/api/sessions/history`   | `rimuru.sessions.history` |

## Costs

| Method | Path | Function |
|--------|------|----------|
| GET    | `/api/costs`                   | `rimuru.costs.summary`  (alias)  |
| GET    | `/api/costs/summary`           | `rimuru.costs.summary`  |
| GET    | `/api/costs/daily`             | `rimuru.costs.daily`    |
| GET    | `/api/costs/agent/:id`         | `rimuru.costs.by_agent` |
| POST   | `/api/costs`                   | `rimuru.costs.record`   |

## Hardware

| Method | Path | Function |
|--------|------|----------|
| GET    | `/api/system`          | `rimuru.hardware.get`    |
| POST   | `/api/system/detect`   | `rimuru.hardware.detect` |

## Models and Advisor

| Method | Path | Function |
|--------|------|----------|
| GET    | `/api/models`                    | `rimuru.models.list`      |
| GET    | `/api/models/:id`                | `rimuru.models.get`       |
| POST   | `/api/models/sync`               | `rimuru.models.sync`      |
| GET    | `/api/models/advisor`            | `rimuru.advisor.assess`   |
| GET    | `/api/models/catalog`            | `rimuru.advisor.catalog`  |
| GET    | `/api/models/catalog/runnable`   | `rimuru.advisor.catalog`  (alias, filters runnable in handler via query params) |

## Metrics

| Method | Path | Function |
|--------|------|----------|
| GET    | `/api/metrics`           | `rimuru.metrics.current` |
| GET    | `/api/metrics/history`   | `rimuru.metrics.history` |

## Context

| Method | Path | Function |
|--------|------|----------|
| GET    | `/api/context/breakdown/:session_id`  | `rimuru.context.breakdown`              |
| GET    | `/api/context/breakdowns`             | `rimuru.context.breakdown_by_session`   |
| GET    | `/api/context/utilization`            | `rimuru.context.utilization`            |
| GET    | `/api/context/waste`                  | `rimuru.context.waste`                  |

## MCP Proxy

| Method | Path | Function |
|--------|------|----------|
| POST   | `/api/mcp/proxy/connect`       | `rimuru.mcp.proxy.connect`    |
| GET    | `/api/mcp/proxy/tools`         | `rimuru.mcp.proxy.tools`      |
| POST   | `/api/mcp/proxy/call`          | `rimuru.mcp.proxy.call`       |
| GET    | `/api/mcp/proxy/search`        | `rimuru.mcp.proxy.search`     |
| GET    | `/api/mcp/proxy/stats`         | `rimuru.mcp.proxy.stats`      |
| POST   | `/api/mcp/proxy/disconnect`    | `rimuru.mcp.proxy.disconnect` |

## Hooks

| Method | Path | Function |
|--------|------|----------|
| POST   | `/api/hooks/register`   | `rimuru.hooks.register` |
| POST   | `/api/hooks/dispatch`   | `rimuru.hooks.dispatch` |

## Plugins

| Method | Path | Function |
|--------|------|----------|
| POST   | `/api/plugins/install`        | `rimuru.plugins.install`   |
| DELETE | `/api/plugins/:id`            | `rimuru.plugins.uninstall` |
| POST   | `/api/plugins/:id/enable`     | `rimuru.plugins.start`     |
| POST   | `/api/plugins/:id/disable`    | `rimuru.plugins.stop`      |

## Config

| Method | Path | Function |
|--------|------|----------|
| GET    | `/api/config`   | `rimuru.config.get` |
| PUT    | `/api/config`   | `rimuru.config.set` |
| POST   | `/api/config`   | `rimuru.config.set` (alias) |

## Health

| Method | Path | Function |
|--------|------|----------|
| GET    | `/api/health`   | `rimuru.health.check` |

## Budget (v0.4.0)

| Method | Path | Function |
|--------|------|----------|
| POST   | `/api/budget/check`    | `rimuru.budget.check`  |
| GET    | `/api/budget/status`   | `rimuru.budget.status` |
| POST   | `/api/budget/set`      | `rimuru.budget.set`    |
| GET    | `/api/budget/alerts`   | `rimuru.budget.alerts` |

## Runaway (v0.4.0)

| Method | Path | Function |
|--------|------|----------|
| POST   | `/api/runaway/analyze`     | `rimuru.runaway.analyze`   |
| GET    | `/api/runaway/scan`        | `rimuru.runaway.scan`      |
| GET    | `/api/runaway/configure`   | `rimuru.runaway.configure` (read) |
| POST   | `/api/runaway/configure`   | `rimuru.runaway.configure` (write) |

## Guard (v0.4.0)

| Method | Path | Function |
|--------|------|----------|
| GET    | `/api/guard`            | `rimuru.guard.list`     |
| POST   | `/api/guard/register`   | `rimuru.guard.register` |
| POST   | `/api/guard/complete`   | `rimuru.guard.complete` |
| GET    | `/api/guard/history`    | `rimuru.guard.history`  |

## iii-trigger-only functions (no HTTP route)

A handful of registered iii functions are not exposed over HTTP because they're internal:

```text
rimuru.costs.*        daily_rollup
rimuru.agents.*       update, delete, status, sync
rimuru.sessions.*     cleanup
```

(`rimuru.costs.record` is available at `POST /api/costs` -- see the Costs table above.)

To expose any of these over HTTP, add a `Route` entry in `crates/rimuru-core/src/triggers/api.rs` -- they are already registered as iii functions, so the route just needs the HTTP binding.
