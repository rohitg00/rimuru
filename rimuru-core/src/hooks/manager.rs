use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{timeout, Duration};
use tracing::{debug, error, info, warn};

use crate::error::{RimuruError, RimuruResult};

use super::types::{
    Hook, HookConfig, HookContext, HookData, HookExecution, HookHandlerInfo, HookResult,
};

#[async_trait]
pub trait HookHandler: Send + Sync {
    fn name(&self) -> &str;

    fn hook(&self) -> Hook;

    fn priority(&self) -> i32 {
        0
    }

    fn description(&self) -> Option<&str> {
        None
    }

    async fn handle(&self, ctx: &HookContext) -> RimuruResult<HookResult>;

    fn info(&self) -> HookHandlerInfo {
        HookHandlerInfo {
            name: self.name().to_string(),
            hook: self.hook(),
            priority: self.priority(),
            enabled: true,
            plugin_id: None,
            description: self.description().map(|s| s.to_string()),
        }
    }
}

pub type DynHookHandler = Arc<dyn HookHandler>;

struct RegisteredHandler {
    handler: DynHookHandler,
    enabled: bool,
    plugin_id: Option<String>,
}

pub struct HookManager {
    handlers: RwLock<HashMap<Hook, Vec<RegisteredHandler>>>,
    configs: RwLock<HashMap<Hook, HookConfig>>,
    global_config: RwLock<HookConfig>,
    executions: RwLock<Vec<HookExecution>>,
    max_execution_history: usize,
    event_handlers: RwLock<Vec<Arc<dyn Fn(HookExecution) + Send + Sync>>>,
}

impl Default for HookManager {
    fn default() -> Self {
        Self::new()
    }
}

impl HookManager {
    pub fn new() -> Self {
        Self {
            handlers: RwLock::new(HashMap::new()),
            configs: RwLock::new(HashMap::new()),
            global_config: RwLock::new(HookConfig::default()),
            executions: RwLock::new(Vec::new()),
            max_execution_history: 1000,
            event_handlers: RwLock::new(Vec::new()),
        }
    }

    pub fn with_max_history(mut self, max: usize) -> Self {
        self.max_execution_history = max;
        self
    }

    pub async fn register(&self, handler: DynHookHandler) -> RimuruResult<()> {
        self.register_with_plugin(handler, None).await
    }

    pub async fn register_with_plugin(
        &self,
        handler: DynHookHandler,
        plugin_id: Option<String>,
    ) -> RimuruResult<()> {
        let hook = handler.hook();
        let name = handler.name().to_string();

        let mut handlers = self.handlers.write().await;
        let hook_handlers = handlers.entry(hook.clone()).or_insert_with(Vec::new);

        let config = self.get_config(&hook).await;
        if hook_handlers.len() >= config.max_handlers {
            return Err(RimuruError::HookError(format!(
                "Maximum handlers ({}) reached for hook '{}'",
                config.max_handlers,
                hook.name()
            )));
        }

        if hook_handlers
            .iter()
            .any(|h| h.handler.name() == handler.name())
        {
            return Err(RimuruError::HookError(format!(
                "Handler '{}' already registered for hook '{}'",
                name,
                hook.name()
            )));
        }

        let registered = RegisteredHandler {
            handler,
            enabled: true,
            plugin_id,
        };

        hook_handlers.push(registered);
        hook_handlers.sort_by(|a, b| b.handler.priority().cmp(&a.handler.priority()));

        info!(
            hook = %hook.name(),
            handler = %name,
            "Registered hook handler"
        );

        Ok(())
    }

    pub async fn unregister(&self, hook: &Hook, handler_name: &str) -> RimuruResult<()> {
        let mut handlers = self.handlers.write().await;

        if let Some(hook_handlers) = handlers.get_mut(hook) {
            let initial_len = hook_handlers.len();
            hook_handlers.retain(|h| h.handler.name() != handler_name);

            if hook_handlers.len() < initial_len {
                info!(
                    hook = %hook.name(),
                    handler = %handler_name,
                    "Unregistered hook handler"
                );
                return Ok(());
            }
        }

        Err(RimuruError::HookHandlerNotFound(format!(
            "{}/{}",
            hook.name(),
            handler_name
        )))
    }

    pub async fn unregister_plugin_handlers(&self, plugin_id: &str) -> usize {
        let mut handlers = self.handlers.write().await;
        let mut removed = 0;

        for hook_handlers in handlers.values_mut() {
            let initial_len = hook_handlers.len();
            hook_handlers.retain(|h| h.plugin_id.as_deref() != Some(plugin_id));
            removed += initial_len - hook_handlers.len();
        }

        if removed > 0 {
            info!(
                plugin_id = %plugin_id,
                count = removed,
                "Unregistered plugin hook handlers"
            );
        }

        removed
    }

    pub async fn enable_handler(&self, hook: &Hook, handler_name: &str) -> RimuruResult<()> {
        self.set_handler_enabled(hook, handler_name, true).await
    }

    pub async fn disable_handler(&self, hook: &Hook, handler_name: &str) -> RimuruResult<()> {
        self.set_handler_enabled(hook, handler_name, false).await
    }

    async fn set_handler_enabled(
        &self,
        hook: &Hook,
        handler_name: &str,
        enabled: bool,
    ) -> RimuruResult<()> {
        let mut handlers = self.handlers.write().await;

        if let Some(hook_handlers) = handlers.get_mut(hook) {
            for h in hook_handlers.iter_mut() {
                if h.handler.name() == handler_name {
                    h.enabled = enabled;
                    debug!(
                        hook = %hook.name(),
                        handler = %handler_name,
                        enabled = enabled,
                        "Handler enabled state changed"
                    );
                    return Ok(());
                }
            }
        }

        Err(RimuruError::HookHandlerNotFound(format!(
            "{}/{}",
            hook.name(),
            handler_name
        )))
    }

    pub async fn set_config(&self, hook: Hook, config: HookConfig) {
        let mut configs = self.configs.write().await;
        configs.insert(hook, config);
    }

    pub async fn set_global_config(&self, config: HookConfig) {
        let mut global = self.global_config.write().await;
        *global = config;
    }

    pub async fn get_config(&self, hook: &Hook) -> HookConfig {
        let configs = self.configs.read().await;
        if let Some(config) = configs.get(hook) {
            return *config;
        }
        *self.global_config.read().await
    }

    pub async fn execute(&self, ctx: HookContext) -> RimuruResult<HookResult> {
        let config = self.get_config(&ctx.hook).await;

        if !config.enabled {
            debug!(hook = %ctx.hook.name(), "Hook disabled, skipping execution");
            return Ok(HookResult::Skip);
        }

        let handlers = self.handlers.read().await;
        let hook_handlers = match handlers.get(&ctx.hook) {
            Some(h) if !h.is_empty() => h,
            _ => {
                debug!(hook = %ctx.hook.name(), "No handlers registered for hook");
                return Ok(HookResult::Continue);
            }
        };

        let enabled_handlers: Vec<_> = hook_handlers.iter().filter(|h| h.enabled).collect();

        if enabled_handlers.is_empty() {
            debug!(hook = %ctx.hook.name(), "All handlers disabled for hook");
            return Ok(HookResult::Continue);
        }

        let timeout_duration = Duration::from_millis(config.timeout_ms);
        let mut current_ctx = ctx.clone();
        let mut final_result = HookResult::Continue;

        for registered in enabled_handlers {
            let handler = &registered.handler;
            let handler_name = handler.name().to_string();
            let mut execution = HookExecution::new(ctx.hook.clone(), &handler_name);

            debug!(
                hook = %ctx.hook.name(),
                handler = %handler_name,
                priority = handler.priority(),
                "Executing hook handler"
            );

            let result = match timeout(timeout_duration, handler.handle(&current_ctx)).await {
                Ok(Ok(result)) => {
                    execution = execution.complete(result.clone());
                    result
                }
                Ok(Err(e)) => {
                    error!(
                        hook = %ctx.hook.name(),
                        handler = %handler_name,
                        error = %e,
                        "Hook handler execution failed"
                    );
                    execution = execution.fail(e.to_string());
                    self.record_execution(execution.clone()).await;
                    self.notify_execution(execution).await;
                    return Err(RimuruError::HookExecutionFailed {
                        hook: ctx.hook.name().to_string(),
                        message: e.to_string(),
                    });
                }
                Err(_) => {
                    warn!(
                        hook = %ctx.hook.name(),
                        handler = %handler_name,
                        timeout_ms = config.timeout_ms,
                        "Hook handler timed out"
                    );
                    execution = execution.fail("Timeout");
                    self.record_execution(execution.clone()).await;
                    self.notify_execution(execution).await;
                    return Err(RimuruError::HookTimeout(
                        ctx.hook.name().to_string(),
                        config.timeout_ms / 1000,
                    ));
                }
            };

            self.record_execution(execution.clone()).await;
            self.notify_execution(execution).await;

            match &result {
                HookResult::Abort { reason } => {
                    info!(
                        hook = %ctx.hook.name(),
                        handler = %handler_name,
                        reason = %reason,
                        "Hook chain aborted"
                    );
                    return Err(RimuruError::HookAborted(
                        ctx.hook.name().to_string(),
                        reason.clone(),
                    ));
                }
                HookResult::Modified { data, message } => {
                    if let Some(msg) = message {
                        debug!(
                            hook = %ctx.hook.name(),
                            handler = %handler_name,
                            message = %msg,
                            "Hook data modified"
                        );
                    }
                    current_ctx = HookContext {
                        data: data.clone(),
                        ..current_ctx
                    };
                    final_result = result;
                }
                HookResult::Skip => {
                    debug!(
                        hook = %ctx.hook.name(),
                        handler = %handler_name,
                        "Handler skipped"
                    );
                    continue;
                }
                HookResult::Continue => {
                    final_result = result;
                }
            }
        }

        Ok(final_result)
    }

    pub async fn execute_with_chaining(
        &self,
        ctx: HookContext,
    ) -> RimuruResult<(HookResult, HookData)> {
        let result = self.execute(ctx.clone()).await?;
        let final_data = match &result {
            HookResult::Modified { data, .. } => data.clone(),
            _ => ctx.data,
        };
        Ok((result, final_data))
    }

    async fn record_execution(&self, execution: HookExecution) {
        let mut executions = self.executions.write().await;
        executions.push(execution);

        if executions.len() > self.max_execution_history {
            let drain_count = executions.len() - self.max_execution_history;
            executions.drain(0..drain_count);
        }
    }

    async fn notify_execution(&self, execution: HookExecution) {
        let handlers = self.event_handlers.read().await;
        for handler in handlers.iter() {
            handler(execution.clone());
        }
    }

    pub async fn on_execution<F>(&self, handler: F)
    where
        F: Fn(HookExecution) + Send + Sync + 'static,
    {
        let mut handlers = self.event_handlers.write().await;
        handlers.push(Arc::new(handler));
    }

    pub async fn get_handlers(&self, hook: &Hook) -> Vec<HookHandlerInfo> {
        let handlers = self.handlers.read().await;
        handlers
            .get(hook)
            .map(|h| {
                h.iter()
                    .map(|r| {
                        let mut info = r.handler.info();
                        info.enabled = r.enabled;
                        info.plugin_id = r.plugin_id.clone();
                        info
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    pub async fn get_all_handlers(&self) -> HashMap<Hook, Vec<HookHandlerInfo>> {
        let handlers = self.handlers.read().await;
        handlers
            .iter()
            .map(|(hook, h)| {
                (
                    hook.clone(),
                    h.iter()
                        .map(|r| {
                            let mut info = r.handler.info();
                            info.enabled = r.enabled;
                            info.plugin_id = r.plugin_id.clone();
                            info
                        })
                        .collect(),
                )
            })
            .collect()
    }

    pub async fn get_recent_executions(&self, limit: usize) -> Vec<HookExecution> {
        let executions = self.executions.read().await;
        executions.iter().rev().take(limit).cloned().collect()
    }

    pub async fn get_executions_for_hook(&self, hook: &Hook, limit: usize) -> Vec<HookExecution> {
        let executions = self.executions.read().await;
        executions
            .iter()
            .rev()
            .filter(|e| &e.hook == hook)
            .take(limit)
            .cloned()
            .collect()
    }

    pub async fn clear_execution_history(&self) {
        let mut executions = self.executions.write().await;
        executions.clear();
    }

    pub async fn handler_count(&self) -> usize {
        let handlers = self.handlers.read().await;
        handlers.values().map(|h| h.len()).sum()
    }

    pub async fn has_handlers(&self, hook: &Hook) -> bool {
        let handlers = self.handlers.read().await;
        handlers.get(hook).map(|h| !h.is_empty()).unwrap_or(false)
    }
}

pub async fn trigger_hook(manager: &HookManager, ctx: HookContext) -> RimuruResult<()> {
    match manager.execute(ctx).await {
        Ok(_) => Ok(()),
        Err(RimuruError::HookAborted(_, _)) => Ok(()),
        Err(e) => Err(e),
    }
}

pub async fn trigger_hook_with_data<T: Into<HookData>>(
    manager: &HookManager,
    hook: Hook,
    data: T,
    source: impl Into<String>,
) -> RimuruResult<HookResult> {
    let ctx = HookContext::new(hook, data.into()).with_source(source);
    manager.execute(ctx).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct TestHandler {
        name: String,
        hook: Hook,
        priority: i32,
        call_count: Arc<AtomicUsize>,
        result: HookResult,
    }

    impl TestHandler {
        fn new(name: &str, hook: Hook) -> Self {
            Self {
                name: name.to_string(),
                hook,
                priority: 0,
                call_count: Arc::new(AtomicUsize::new(0)),
                result: HookResult::Continue,
            }
        }

        fn with_priority(mut self, priority: i32) -> Self {
            self.priority = priority;
            self
        }

        fn with_result(mut self, result: HookResult) -> Self {
            self.result = result;
            self
        }

        fn calls(&self) -> usize {
            self.call_count.load(Ordering::SeqCst)
        }
    }

    #[async_trait]
    impl HookHandler for TestHandler {
        fn name(&self) -> &str {
            &self.name
        }

        fn hook(&self) -> Hook {
            self.hook.clone()
        }

        fn priority(&self) -> i32 {
            self.priority
        }

        async fn handle(&self, _ctx: &HookContext) -> RimuruResult<HookResult> {
            self.call_count.fetch_add(1, Ordering::SeqCst);
            Ok(self.result.clone())
        }
    }

    #[tokio::test]
    async fn test_register_handler() {
        let manager = HookManager::new();
        let handler = Arc::new(TestHandler::new("test", Hook::PreSessionStart));

        manager.register(handler).await.unwrap();

        assert!(manager.has_handlers(&Hook::PreSessionStart).await);
        assert_eq!(manager.handler_count().await, 1);
    }

    #[tokio::test]
    async fn test_register_duplicate_handler() {
        let manager = HookManager::new();
        let handler1 = Arc::new(TestHandler::new("test", Hook::PreSessionStart));
        let handler2 = Arc::new(TestHandler::new("test", Hook::PreSessionStart));

        manager.register(handler1).await.unwrap();
        let result = manager.register(handler2).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_unregister_handler() {
        let manager = HookManager::new();
        let handler = Arc::new(TestHandler::new("test", Hook::PreSessionStart));

        manager.register(handler).await.unwrap();
        manager
            .unregister(&Hook::PreSessionStart, "test")
            .await
            .unwrap();

        assert!(!manager.has_handlers(&Hook::PreSessionStart).await);
    }

    #[tokio::test]
    async fn test_execute_handler() {
        let manager = HookManager::new();
        let handler = Arc::new(TestHandler::new("test", Hook::PreSessionStart));
        let handler_clone = handler.clone();

        manager.register(handler).await.unwrap();

        let ctx = HookContext::new(Hook::PreSessionStart, HookData::None);
        let result = manager.execute(ctx).await.unwrap();

        assert!(result.is_continue());
        assert_eq!(handler_clone.calls(), 1);
    }

    #[tokio::test]
    async fn test_handler_priority_order() {
        let manager = HookManager::new();
        let order = Arc::new(RwLock::new(Vec::new()));

        struct OrderTracker {
            name: String,
            hook: Hook,
            priority: i32,
            order: Arc<RwLock<Vec<String>>>,
        }

        #[async_trait]
        impl HookHandler for OrderTracker {
            fn name(&self) -> &str {
                &self.name
            }
            fn hook(&self) -> Hook {
                self.hook.clone()
            }
            fn priority(&self) -> i32 {
                self.priority
            }
            async fn handle(&self, _ctx: &HookContext) -> RimuruResult<HookResult> {
                self.order.write().await.push(self.name.clone());
                Ok(HookResult::Continue)
            }
        }

        let h1 = Arc::new(OrderTracker {
            name: "low".to_string(),
            hook: Hook::PreSessionStart,
            priority: 1,
            order: order.clone(),
        });
        let h2 = Arc::new(OrderTracker {
            name: "high".to_string(),
            hook: Hook::PreSessionStart,
            priority: 10,
            order: order.clone(),
        });
        let h3 = Arc::new(OrderTracker {
            name: "medium".to_string(),
            hook: Hook::PreSessionStart,
            priority: 5,
            order: order.clone(),
        });

        manager.register(h1).await.unwrap();
        manager.register(h2).await.unwrap();
        manager.register(h3).await.unwrap();

        let ctx = HookContext::new(Hook::PreSessionStart, HookData::None);
        manager.execute(ctx).await.unwrap();

        let execution_order = order.read().await;
        assert_eq!(*execution_order, vec!["high", "medium", "low"]);
    }

    #[tokio::test]
    async fn test_abort_stops_chain() {
        let manager = HookManager::new();
        let handler1 = Arc::new(
            TestHandler::new("first", Hook::PreSessionStart)
                .with_priority(10)
                .with_result(HookResult::abort("stopped")),
        );
        let handler2 = Arc::new(TestHandler::new("second", Hook::PreSessionStart).with_priority(5));
        let handler2_clone = handler2.clone();

        manager.register(handler1).await.unwrap();
        manager.register(handler2).await.unwrap();

        let ctx = HookContext::new(Hook::PreSessionStart, HookData::None);
        let result = manager.execute(ctx).await;

        assert!(result.is_err());
        assert_eq!(handler2_clone.calls(), 0);
    }

    #[tokio::test]
    async fn test_disable_handler() {
        let manager = HookManager::new();
        let handler = Arc::new(TestHandler::new("test", Hook::PreSessionStart));
        let handler_clone = handler.clone();

        manager.register(handler).await.unwrap();
        manager
            .disable_handler(&Hook::PreSessionStart, "test")
            .await
            .unwrap();

        let ctx = HookContext::new(Hook::PreSessionStart, HookData::None);
        manager.execute(ctx).await.unwrap();

        assert_eq!(handler_clone.calls(), 0);
    }

    #[tokio::test]
    async fn test_execution_history() {
        let manager = HookManager::new().with_max_history(10);
        let handler = Arc::new(TestHandler::new("test", Hook::PreSessionStart));

        manager.register(handler).await.unwrap();

        for _ in 0..5 {
            let ctx = HookContext::new(Hook::PreSessionStart, HookData::None);
            manager.execute(ctx).await.unwrap();
        }

        let executions = manager.get_recent_executions(10).await;
        assert_eq!(executions.len(), 5);
    }

    #[tokio::test]
    async fn test_no_handlers_returns_continue() {
        let manager = HookManager::new();

        let ctx = HookContext::new(Hook::PreSessionStart, HookData::None);
        let result = manager.execute(ctx).await.unwrap();

        assert!(result.is_continue());
    }

    #[tokio::test]
    async fn test_disabled_hook_skips_execution() {
        let manager = HookManager::new();
        let handler = Arc::new(TestHandler::new("test", Hook::PreSessionStart));
        let handler_clone = handler.clone();

        manager.register(handler).await.unwrap();
        manager
            .set_config(Hook::PreSessionStart, HookConfig::default().disabled())
            .await;

        let ctx = HookContext::new(Hook::PreSessionStart, HookData::None);
        let result = manager.execute(ctx).await.unwrap();

        assert!(result.is_skip());
        assert_eq!(handler_clone.calls(), 0);
    }
}
