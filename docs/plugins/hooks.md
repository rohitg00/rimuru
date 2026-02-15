# Rimuru Hooks Reference

This document provides a comprehensive reference for the Rimuru hooks system.

## Overview

Hooks allow plugins to react to events in the Rimuru system. When an event occurs (like a session starting or cost being recorded), Rimuru executes all registered handlers in priority order.

## Hook Types

### PreSessionStart

Fired before a new session starts.

```rust
hook_handler!(
    SessionStartValidator,
    name: "session-start-validator",
    hook: Hook::PreSessionStart,
    priority: 100,
    description: "Validate session before start"
);

impl_hook_handler!(SessionStartValidator, |ctx| {
    if let HookData::Session(session) = &ctx.data {
        // Validate or modify session
        if session.agent_name.is_empty() {
            return Ok(HookResult::abort("Session must have agent name"));
        }
    }
    Ok(HookResult::ok())
});
```

**Data Type**: `HookData::Session(Session)`

### PostSessionEnd

Fired after a session ends.

```rust
hook_handler!(
    SessionEndLogger,
    name: "session-end-logger",
    hook: Hook::PostSessionEnd,
    priority: 50,
    description: "Log session completion"
);

impl_hook_handler!(SessionEndLogger, |ctx| {
    if let HookData::Session(session) = &ctx.data {
        info!(
            "Session {} ended with status {:?}",
            session.id,
            session.status
        );
    }
    Ok(HookResult::ok())
});
```

**Data Type**: `HookData::Session(Session)`

### OnCostRecorded

Fired when a cost record is created.

```rust
hook_handler!(
    BudgetChecker,
    name: "budget-checker",
    hook: Hook::OnCostRecorded,
    priority: 100,
    description: "Check if cost exceeds budget"
);

impl_hook_handler!(BudgetChecker, |ctx| {
    if let HookData::Cost(cost) = &ctx.data {
        if cost.cost_usd > 10.0 {
            // Send alert or abort
            return Ok(HookResult::abort("Cost exceeds daily budget"));
        }
    }
    Ok(HookResult::ok())
});
```

**Data Type**: `HookData::Cost(CostRecord)`

### OnMetricsCollected

Fired when system metrics are collected.

```rust
hook_handler!(
    MetricsExporter,
    name: "metrics-exporter",
    hook: Hook::OnMetricsCollected,
    priority: 50,
    description: "Export metrics to external service"
);

impl_hook_handler!(MetricsExporter, |ctx| {
    if let HookData::Metrics(metrics) = &ctx.data {
        // Export metrics to Prometheus, Datadog, etc.
        export_metrics(metrics).await?;
    }
    Ok(HookResult::ok())
});
```

**Data Type**: `HookData::Metrics(MetricsSnapshot)`

### OnAgentConnect

Fired when an agent connects.

```rust
hook_handler!(
    AgentConnectNotifier,
    name: "agent-connect-notifier",
    hook: Hook::OnAgentConnect,
    priority: 50,
    description: "Notify on agent connection"
);

impl_hook_handler!(AgentConnectNotifier, |ctx| {
    if let HookData::Agent { agent_id, agent_name, agent_type } = &ctx.data {
        info!("Agent connected: {} ({})", agent_name, agent_type);
    }
    Ok(HookResult::ok())
});
```

**Data Type**: `HookData::Agent { agent_id: Uuid, agent_name: String, agent_type: String }`

### OnAgentDisconnect

Fired when an agent disconnects.

**Data Type**: `HookData::Agent { agent_id: Uuid, agent_name: String, agent_type: String }`

### OnSyncComplete

Fired after data synchronization completes.

```rust
hook_handler!(
    SyncCompleteHandler,
    name: "sync-complete-handler",
    hook: Hook::OnSyncComplete,
    priority: 50,
    description: "Handle sync completion"
);

impl_hook_handler!(SyncCompleteHandler, |ctx| {
    if let HookData::Sync { provider, models_synced, duration_ms } = &ctx.data {
        info!(
            "Sync complete: {} synced {} models in {}ms",
            provider, models_synced, duration_ms
        );
    }
    Ok(HookResult::ok())
});
```

**Data Type**: `HookData::Sync { provider: String, models_synced: usize, duration_ms: u64 }`

### OnPluginLoaded

Fired when a plugin is loaded.

**Data Type**: `HookData::Plugin { plugin_id: String, plugin_name: String }`

### OnPluginUnloaded

Fired when a plugin is unloaded.

**Data Type**: `HookData::Plugin { plugin_id: String, plugin_name: String }`

### OnConfigChanged

Fired when configuration changes.

**Data Type**: `HookData::Config { changed_keys: Vec<String> }`

### OnError

Fired when an error occurs.

```rust
hook_handler!(
    ErrorReporter,
    name: "error-reporter",
    hook: Hook::OnError,
    priority: 100,
    description: "Report errors to external service"
);

impl_hook_handler!(ErrorReporter, |ctx| {
    if let HookData::Error { error_code, error_message, source } = &ctx.data {
        error!(
            "Error {}: {} (source: {:?})",
            error_code, error_message, source
        );
        // Report to Sentry, etc.
    }
    Ok(HookResult::ok())
});
```

**Data Type**: `HookData::Error { error_code: String, error_message: String, source: Option<String> }`

### Custom Hooks

You can define custom hooks for plugin-to-plugin communication:

```rust
// Define custom hook
let hook = Hook::Custom("my-custom-event".to_string());

// Register handler
hook_handler!(
    CustomEventHandler,
    name: "custom-event-handler",
    hook: Hook::Custom("my-custom-event".to_string()),
    priority: 50,
    description: "Handle custom events"
);

impl_hook_handler!(CustomEventHandler, |ctx| {
    if let HookData::Custom(value) = &ctx.data {
        // Handle custom data
    }
    Ok(HookResult::ok())
});

// Trigger custom hook
let ctx = HookContext::new(
    Hook::Custom("my-custom-event".to_string()),
    HookData::Custom(json!({"key": "value"}))
);
hook_manager.execute(ctx).await?;
```

**Data Type**: `HookData::Custom(serde_json::Value)`

## Hook Context

Every hook handler receives a `HookContext` with:

```rust
pub struct HookContext {
    pub hook: Hook,              // The hook type
    pub data: HookData,          // Hook-specific data
    pub timestamp: DateTime<Utc>, // When the event occurred
    pub source: String,          // Source component
    pub correlation_id: Uuid,    // For tracing
    pub metadata: HashMap<String, Value>, // Additional context
}
```

### Creating Context

```rust
// Using constructors
let ctx = HookContext::session_start(session);
let ctx = HookContext::cost_recorded(cost);
let ctx = HookContext::error("E001", "Something went wrong", Some("api"));

// Manual creation with builder
let ctx = HookContext::new(Hook::OnCostRecorded, HookData::Cost(cost))
    .with_source("cost_tracker")
    .with_metadata("user_id", "123")
    .with_correlation_id(correlation_id);
```

### Accessing Metadata

```rust
impl_hook_handler!(MyHandler, |ctx| {
    // Get typed metadata
    if let Some(user_id) = ctx.get_metadata::<String>("user_id") {
        info!("Processing for user: {}", user_id);
    }
    Ok(HookResult::ok())
});
```

## Hook Results

### Continue

Continue to the next handler in the chain:

```rust
Ok(HookResult::Continue)
// or
Ok(HookResult::ok())
```

### Abort

Stop the hook chain and return an error:

```rust
Ok(HookResult::Abort { reason: "Budget exceeded".to_string() })
// or
Ok(HookResult::abort("Budget exceeded"))
```

The abort reason is returned to the caller as a `RimuruError::HookAborted`.

### Modified

Pass modified data to the next handler:

```rust
// Modify the data
let mut modified_session = session.clone();
modified_session.metadata["processed"] = json!(true);

Ok(HookResult::Modified {
    data: HookData::Session(modified_session),
    message: Some("Added processed flag".to_string())
})
// or
Ok(HookResult::modified(HookData::Session(modified_session)))
Ok(HookResult::modified_with_message(
    HookData::Session(modified_session),
    "Added processed flag"
))
```

### Skip

Skip this handler but continue the chain:

```rust
// Conditionally skip
if !should_process(&ctx) {
    return Ok(HookResult::Skip);
    // or
    return Ok(HookResult::skip());
}
```

## Priority System

Handlers are executed in priority order (highest first):

| Priority Range | Typical Use |
|---------------|-------------|
| 100+ | Validators, security checks |
| 50-99 | Standard processing |
| 10-49 | Logging, metrics |
| 0-9 | Cleanup, notifications |

```rust
hook_handler!(
    HighPriorityValidator,
    name: "validator",
    hook: Hook::PreSessionStart,
    priority: 100  // Runs first
);

hook_handler!(
    LowPriorityLogger,
    name: "logger",
    hook: Hook::PreSessionStart,
    priority: 10   // Runs last
);
```

## Built-in Handlers

Rimuru provides several built-in handlers:

### CostAlertHandler

Sends alerts when costs exceed thresholds:

```rust
let handler = CostAlertHandler::new(CostAlertConfig {
    threshold: 100.0,
    daily_budget: Some(500.0),
    weekly_budget: Some(2500.0),
    monthly_budget: Some(10000.0),
    alert_interval_seconds: 3600,
});
```

### SessionLogHandler

Logs sessions to file:

```rust
let handler = SessionLogHandler::new(SessionLogConfig {
    format: SessionLogFormat::Json,
    include_metadata: true,
    log_path: PathBuf::from("/var/log/rimuru/sessions.log"),
});
```

### MetricsExportHandler

Exports metrics to external services:

```rust
let handler = MetricsExportHandler::new(MetricsExportConfig {
    endpoint: "https://metrics.example.com/v1/metrics".to_string(),
    api_key: Some("key".to_string()),
    batch_size: 100,
    flush_interval_seconds: 60,
});
```

### WebhookHandler

Sends events to webhook URLs:

```rust
let handler = WebhookHandler::new(WebhookConfig {
    url: "https://example.com/webhook".to_string(),
    headers: HashMap::from([
        ("Authorization".to_string(), "Bearer token".to_string())
    ]),
    events: vec![Hook::OnCostRecorded, Hook::OnError],
});
```

## Hook Configuration

Configure hooks globally or per-hook:

```rust
// Global config
let config = HookConfig::new()
    .with_timeout(10000)     // 10 second timeout
    .with_max_handlers(50)   // Max 50 handlers per hook
    .parallel();             // Enable parallel execution

hook_manager.set_global_config(config).await;

// Per-hook config
hook_manager.set_config(
    Hook::OnCostRecorded,
    HookConfig::new()
        .with_timeout(5000)
        .with_max_handlers(10)
).await;
```

## Error Handling

Hooks can fail in several ways:

```rust
// Handler returns error
async fn handle(&self, ctx: &HookContext) -> RimuruResult<HookResult> {
    Err(RimuruError::plugin("Something went wrong"))
}

// Handler times out
// Automatic if handler exceeds timeout_ms

// Handler aborts
Ok(HookResult::abort("Validation failed"))
```

All failures are recorded in the execution history:

```rust
// Get recent executions
let executions = hook_manager.get_recent_executions(100).await;

for exec in executions {
    if let Some(error) = &exec.error {
        println!("Handler {} failed: {}", exec.handler_name, error);
    }
}
```

## Best Practices

1. **Keep handlers fast**: Avoid blocking operations
2. **Use appropriate priorities**: Validators first, loggers last
3. **Handle errors gracefully**: Don't crash on expected failures
4. **Use correlation IDs**: For tracing across handlers
5. **Log important events**: Use tracing macros
6. **Test thoroughly**: Unit test each handler
7. **Document side effects**: Especially for Modified results
