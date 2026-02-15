use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::error::RimuruResult;
use crate::plugins::{Notification, NotificationLevel};

use super::manager::HookHandler;
use super::types::{Hook, HookContext, HookData, HookResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostAlertConfig {
    pub threshold_usd: f64,
    pub alert_interval_seconds: u64,
    pub daily_budget_usd: Option<f64>,
    pub weekly_budget_usd: Option<f64>,
    pub monthly_budget_usd: Option<f64>,
}

impl Default for CostAlertConfig {
    fn default() -> Self {
        Self {
            threshold_usd: 1.0,
            alert_interval_seconds: 3600,
            daily_budget_usd: None,
            weekly_budget_usd: None,
            monthly_budget_usd: None,
        }
    }
}

pub struct CostAlertHandler {
    config: RwLock<CostAlertConfig>,
    last_alert: RwLock<Option<DateTime<Utc>>>,
    daily_total: RwLock<f64>,
    weekly_total: RwLock<f64>,
    monthly_total: RwLock<f64>,
    notifier: Option<Arc<dyn Fn(Notification) + Send + Sync>>,
}

impl CostAlertHandler {
    pub fn new(config: CostAlertConfig) -> Self {
        Self {
            config: RwLock::new(config),
            last_alert: RwLock::new(None),
            daily_total: RwLock::new(0.0),
            weekly_total: RwLock::new(0.0),
            monthly_total: RwLock::new(0.0),
            notifier: None,
        }
    }

    pub fn with_notifier<F>(mut self, notifier: F) -> Self
    where
        F: Fn(Notification) + Send + Sync + 'static,
    {
        self.notifier = Some(Arc::new(notifier));
        self
    }

    pub async fn update_config(&self, config: CostAlertConfig) {
        let mut cfg = self.config.write().await;
        *cfg = config;
    }

    pub async fn reset_daily(&self) {
        let mut daily = self.daily_total.write().await;
        *daily = 0.0;
    }

    pub async fn reset_weekly(&self) {
        let mut weekly = self.weekly_total.write().await;
        *weekly = 0.0;
    }

    pub async fn reset_monthly(&self) {
        let mut monthly = self.monthly_total.write().await;
        *monthly = 0.0;
    }

    fn send_notification(&self, notification: Notification) {
        if let Some(ref notifier) = self.notifier {
            notifier(notification);
        } else {
            match notification.level {
                NotificationLevel::Critical | NotificationLevel::Error => {
                    error!(
                        title = %notification.title,
                        message = %notification.message,
                        "Cost alert"
                    );
                }
                NotificationLevel::Warning => {
                    warn!(
                        title = %notification.title,
                        message = %notification.message,
                        "Cost alert"
                    );
                }
                _ => {
                    info!(
                        title = %notification.title,
                        message = %notification.message,
                        "Cost alert"
                    );
                }
            }
        }
    }

    async fn should_alert(&self) -> bool {
        let config = self.config.read().await;
        let last = self.last_alert.read().await;

        if let Some(last_time) = *last {
            let elapsed = (Utc::now() - last_time).num_seconds() as u64;
            elapsed >= config.alert_interval_seconds
        } else {
            true
        }
    }

    async fn record_alert(&self) {
        let mut last = self.last_alert.write().await;
        *last = Some(Utc::now());
    }
}

#[async_trait]
impl HookHandler for CostAlertHandler {
    fn name(&self) -> &str {
        "cost_alert"
    }

    fn hook(&self) -> Hook {
        Hook::OnCostRecorded
    }

    fn priority(&self) -> i32 {
        100
    }

    fn description(&self) -> Option<&str> {
        Some("Alerts when cost exceeds configured thresholds")
    }

    async fn handle(&self, ctx: &HookContext) -> RimuruResult<HookResult> {
        let cost = match &ctx.data {
            HookData::Cost(record) => record.cost_usd,
            _ => return Ok(HookResult::Continue),
        };

        let config = self.config.read().await;

        {
            let mut daily = self.daily_total.write().await;
            *daily += cost;
        }
        {
            let mut weekly = self.weekly_total.write().await;
            *weekly += cost;
        }
        {
            let mut monthly = self.monthly_total.write().await;
            *monthly += cost;
        }

        if cost >= config.threshold_usd && self.should_alert().await {
            self.record_alert().await;
            let notification = Notification::warning(
                "High Cost Alert",
                format!(
                    "Single request cost ${:.4} exceeds threshold ${:.4}",
                    cost, config.threshold_usd
                ),
            )
            .with_data(serde_json::json!({
                "cost": cost,
                "threshold": config.threshold_usd,
            }));
            self.send_notification(notification);
        }

        let daily_total = *self.daily_total.read().await;
        if let Some(daily_budget) = config.daily_budget_usd {
            if daily_total >= daily_budget && self.should_alert().await {
                self.record_alert().await;
                let notification = Notification::critical(
                    "Daily Budget Exceeded",
                    format!(
                        "Daily spending ${:.2} has exceeded budget ${:.2}",
                        daily_total, daily_budget
                    ),
                );
                self.send_notification(notification);
            }
        }

        let weekly_total = *self.weekly_total.read().await;
        if let Some(weekly_budget) = config.weekly_budget_usd {
            if weekly_total >= weekly_budget && self.should_alert().await {
                self.record_alert().await;
                let notification = Notification::critical(
                    "Weekly Budget Exceeded",
                    format!(
                        "Weekly spending ${:.2} has exceeded budget ${:.2}",
                        weekly_total, weekly_budget
                    ),
                );
                self.send_notification(notification);
            }
        }

        let monthly_total = *self.monthly_total.read().await;
        if let Some(monthly_budget) = config.monthly_budget_usd {
            if monthly_total >= monthly_budget && self.should_alert().await {
                self.record_alert().await;
                let notification = Notification::critical(
                    "Monthly Budget Exceeded",
                    format!(
                        "Monthly spending ${:.2} has exceeded budget ${:.2}",
                        monthly_total, monthly_budget
                    ),
                );
                self.send_notification(notification);
            }
        }

        Ok(HookResult::Continue)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionLogConfig {
    pub log_path: PathBuf,
    pub format: SessionLogFormat,
    pub include_metadata: bool,
    pub max_file_size_mb: Option<u64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum SessionLogFormat {
    #[default]
    Json,
    Csv,
    Plain,
}

impl Default for SessionLogConfig {
    fn default() -> Self {
        Self {
            log_path: PathBuf::from("sessions.log"),
            format: SessionLogFormat::Json,
            include_metadata: true,
            max_file_size_mb: Some(100),
        }
    }
}

pub struct SessionLogHandler {
    config: RwLock<SessionLogConfig>,
}

impl SessionLogHandler {
    pub fn new(config: SessionLogConfig) -> Self {
        Self {
            config: RwLock::new(config),
        }
    }

    pub async fn update_config(&self, config: SessionLogConfig) {
        let mut cfg = self.config.write().await;
        *cfg = config;
    }

    async fn format_entry(&self, session: &crate::models::Session, event_type: &str) -> String {
        let config = self.config.read().await;

        match config.format {
            SessionLogFormat::Json => {
                let entry = if config.include_metadata {
                    serde_json::json!({
                        "timestamp": Utc::now().to_rfc3339(),
                        "event": event_type,
                        "session_id": session.id.to_string(),
                        "agent_id": session.agent_id.to_string(),
                        "status": session.status.to_string(),
                        "started_at": session.started_at.to_rfc3339(),
                        "ended_at": session.ended_at.map(|t| t.to_rfc3339()),
                        "metadata": session.metadata,
                    })
                } else {
                    serde_json::json!({
                        "timestamp": Utc::now().to_rfc3339(),
                        "event": event_type,
                        "session_id": session.id.to_string(),
                        "agent_id": session.agent_id.to_string(),
                        "status": session.status.to_string(),
                    })
                };
                format!("{}\n", serde_json::to_string(&entry).unwrap_or_default())
            }
            SessionLogFormat::Csv => {
                format!(
                    "{},{},{},{},{}\n",
                    Utc::now().to_rfc3339(),
                    event_type,
                    session.id,
                    session.agent_id,
                    session.status
                )
            }
            SessionLogFormat::Plain => {
                format!(
                    "[{}] {} session {} (agent: {}, status: {})\n",
                    Utc::now().format("%Y-%m-%d %H:%M:%S"),
                    event_type,
                    session.id,
                    session.agent_id,
                    session.status
                )
            }
        }
    }

    async fn write_entry(&self, entry: &str) -> RimuruResult<()> {
        let config = self.config.read().await;

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&config.log_path)
            .await?;

        file.write_all(entry.as_bytes()).await?;
        file.flush().await?;

        Ok(())
    }
}

#[async_trait]
impl HookHandler for SessionLogHandler {
    fn name(&self) -> &str {
        "session_log"
    }

    fn hook(&self) -> Hook {
        Hook::PostSessionEnd
    }

    fn priority(&self) -> i32 {
        50
    }

    fn description(&self) -> Option<&str> {
        Some("Logs session events to a file")
    }

    async fn handle(&self, ctx: &HookContext) -> RimuruResult<HookResult> {
        let session = match &ctx.data {
            HookData::Session(s) => s,
            _ => return Ok(HookResult::Continue),
        };

        let event_type = match ctx.hook {
            Hook::PreSessionStart => "START",
            Hook::PostSessionEnd => "END",
            _ => "EVENT",
        };

        let entry = self.format_entry(session, event_type).await;

        if let Err(e) = self.write_entry(&entry).await {
            error!(error = %e, "Failed to write session log entry");
        } else {
            debug!(
                session_id = %session.id,
                event_type = %event_type,
                "Session event logged"
            );
        }

        Ok(HookResult::Continue)
    }
}

pub struct SessionStartLogHandler {
    inner: SessionLogHandler,
}

impl SessionStartLogHandler {
    pub fn new(config: SessionLogConfig) -> Self {
        Self {
            inner: SessionLogHandler::new(config),
        }
    }
}

#[async_trait]
impl HookHandler for SessionStartLogHandler {
    fn name(&self) -> &str {
        "session_start_log"
    }

    fn hook(&self) -> Hook {
        Hook::PreSessionStart
    }

    fn priority(&self) -> i32 {
        50
    }

    fn description(&self) -> Option<&str> {
        Some("Logs session start events to a file")
    }

    async fn handle(&self, ctx: &HookContext) -> RimuruResult<HookResult> {
        self.inner.handle(ctx).await
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsExportConfig {
    pub endpoint: String,
    pub api_key: Option<String>,
    pub batch_size: usize,
    pub flush_interval_seconds: u64,
    pub include_system_metrics: bool,
    pub tags: std::collections::HashMap<String, String>,
}

impl Default for MetricsExportConfig {
    fn default() -> Self {
        Self {
            endpoint: String::new(),
            api_key: None,
            batch_size: 100,
            flush_interval_seconds: 60,
            include_system_metrics: true,
            tags: std::collections::HashMap::new(),
        }
    }
}

pub struct MetricsExportHandler {
    config: RwLock<MetricsExportConfig>,
    buffer: RwLock<Vec<serde_json::Value>>,
    last_flush: RwLock<DateTime<Utc>>,
    client: reqwest::Client,
}

impl MetricsExportHandler {
    pub fn new(config: MetricsExportConfig) -> Self {
        Self {
            config: RwLock::new(config),
            buffer: RwLock::new(Vec::new()),
            last_flush: RwLock::new(Utc::now()),
            client: reqwest::Client::new(),
        }
    }

    pub async fn update_config(&self, config: MetricsExportConfig) {
        let mut cfg = self.config.write().await;
        *cfg = config;
    }

    pub async fn flush(&self) -> RimuruResult<usize> {
        let config = self.config.read().await;
        let mut buffer = self.buffer.write().await;

        if buffer.is_empty() {
            return Ok(0);
        }

        if config.endpoint.is_empty() {
            buffer.clear();
            return Ok(0);
        }

        let metrics: Vec<_> = buffer.drain(..).collect();
        let count = metrics.len();

        drop(buffer);
        drop(config);

        let config = self.config.read().await;
        let payload = serde_json::json!({
            "metrics": metrics,
            "timestamp": Utc::now().to_rfc3339(),
            "tags": config.tags,
        });

        let mut request = self.client.post(&config.endpoint).json(&payload);

        if let Some(ref api_key) = config.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        match request.send().await {
            Ok(response) if response.status().is_success() => {
                debug!(count = count, "Metrics exported successfully");
                let mut last_flush = self.last_flush.write().await;
                *last_flush = Utc::now();
                Ok(count)
            }
            Ok(response) => {
                error!(
                    status = %response.status(),
                    "Failed to export metrics"
                );
                Ok(0)
            }
            Err(e) => {
                error!(error = %e, "Failed to send metrics");
                Ok(0)
            }
        }
    }

    async fn should_flush(&self) -> bool {
        let config = self.config.read().await;
        let buffer = self.buffer.read().await;
        let last_flush = self.last_flush.read().await;

        if buffer.len() >= config.batch_size {
            return true;
        }

        let elapsed = (Utc::now() - *last_flush).num_seconds() as u64;
        elapsed >= config.flush_interval_seconds && !buffer.is_empty()
    }
}

#[async_trait]
impl HookHandler for MetricsExportHandler {
    fn name(&self) -> &str {
        "metrics_export"
    }

    fn hook(&self) -> Hook {
        Hook::OnMetricsCollected
    }

    fn priority(&self) -> i32 {
        25
    }

    fn description(&self) -> Option<&str> {
        Some("Exports metrics to an external service")
    }

    async fn handle(&self, ctx: &HookContext) -> RimuruResult<HookResult> {
        let metrics = match &ctx.data {
            HookData::Metrics(m) => m,
            _ => return Ok(HookResult::Continue),
        };

        let config = self.config.read().await;
        if !config.include_system_metrics {
            return Ok(HookResult::Continue);
        }
        drop(config);

        let metric_data = serde_json::json!({
            "type": "system_metrics",
            "timestamp": Utc::now().to_rfc3339(),
            "cpu_percent": metrics.cpu_percent,
            "memory_used_mb": metrics.memory_used_mb,
            "memory_total_mb": metrics.memory_total_mb,
            "memory_percent": metrics.memory_percent,
            "active_sessions": metrics.active_sessions,
        });

        {
            let mut buffer = self.buffer.write().await;
            buffer.push(metric_data);
        }

        if self.should_flush().await {
            self.flush().await?;
        }

        Ok(HookResult::Continue)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    pub url: String,
    pub headers: std::collections::HashMap<String, String>,
    pub events: Vec<Hook>,
    pub timeout_seconds: u64,
}

impl Default for WebhookConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            headers: std::collections::HashMap::new(),
            events: Vec::new(),
            timeout_seconds: 10,
        }
    }
}

pub struct WebhookHandler {
    config: RwLock<WebhookConfig>,
    client: reqwest::Client,
}

impl WebhookHandler {
    pub fn new(config: WebhookConfig) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self {
            config: RwLock::new(config),
            client,
        }
    }

    pub async fn update_config(&self, config: WebhookConfig) {
        let mut cfg = self.config.write().await;
        *cfg = config;
    }
}

#[async_trait]
impl HookHandler for WebhookHandler {
    fn name(&self) -> &str {
        "webhook"
    }

    fn hook(&self) -> Hook {
        Hook::Custom("webhook_all".to_string())
    }

    fn priority(&self) -> i32 {
        10
    }

    fn description(&self) -> Option<&str> {
        Some("Sends hook events to a webhook URL")
    }

    async fn handle(&self, ctx: &HookContext) -> RimuruResult<HookResult> {
        let config = self.config.read().await;

        if config.url.is_empty() {
            return Ok(HookResult::Continue);
        }

        if !config.events.is_empty() && !config.events.contains(&ctx.hook) {
            return Ok(HookResult::Continue);
        }

        let payload = serde_json::json!({
            "hook": ctx.hook.name(),
            "timestamp": ctx.timestamp.to_rfc3339(),
            "source": ctx.source,
            "correlation_id": ctx.correlation_id.to_string(),
            "data": ctx.data,
            "metadata": ctx.metadata,
        });

        let mut request = self.client.post(&config.url).json(&payload);

        for (key, value) in &config.headers {
            request = request.header(key.as_str(), value.as_str());
        }

        match request.send().await {
            Ok(response) if response.status().is_success() => {
                debug!(
                    hook = %ctx.hook.name(),
                    url = %config.url,
                    "Webhook sent successfully"
                );
            }
            Ok(response) => {
                warn!(
                    hook = %ctx.hook.name(),
                    url = %config.url,
                    status = %response.status(),
                    "Webhook request failed"
                );
            }
            Err(e) => {
                error!(
                    hook = %ctx.hook.name(),
                    url = %config.url,
                    error = %e,
                    "Webhook request error"
                );
            }
        }

        Ok(HookResult::Continue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CostRecord, MetricsSnapshot, Session, SessionStatus};
    use uuid::Uuid;

    fn create_test_session() -> Session {
        Session {
            id: Uuid::new_v4(),
            agent_id: Uuid::new_v4(),
            status: SessionStatus::Completed,
            started_at: Utc::now(),
            ended_at: Some(Utc::now()),
            metadata: serde_json::json!({}),
        }
    }

    fn create_test_cost() -> CostRecord {
        CostRecord::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "claude-3-opus".to_string(),
            1000,
            500,
            0.05,
        )
    }

    fn create_test_metrics() -> MetricsSnapshot {
        MetricsSnapshot {
            cpu_percent: 50.0,
            memory_used_mb: 8192,
            memory_total_mb: 16384,
            memory_percent: 50.0,
            active_sessions: 2,
        }
    }

    #[tokio::test]
    async fn test_cost_alert_handler_below_threshold() {
        let config = CostAlertConfig {
            threshold_usd: 1.0,
            ..Default::default()
        };
        let handler = CostAlertHandler::new(config);

        let cost = create_test_cost();
        let ctx = HookContext::cost_recorded(cost);

        let result = handler.handle(&ctx).await.unwrap();
        assert!(result.is_continue());
    }

    #[tokio::test]
    async fn test_cost_alert_handler_above_threshold() {
        let config = CostAlertConfig {
            threshold_usd: 0.01,
            alert_interval_seconds: 0,
            ..Default::default()
        };

        let notifications = Arc::new(RwLock::new(Vec::new()));
        let notifications_clone = notifications.clone();

        let handler = CostAlertHandler::new(config).with_notifier(move |n| {
            let notifications = notifications_clone.clone();
            tokio::spawn(async move {
                notifications.write().await.push(n);
            });
        });

        let cost = create_test_cost();
        let ctx = HookContext::cost_recorded(cost);

        let result = handler.handle(&ctx).await.unwrap();
        assert!(result.is_continue());
    }

    #[tokio::test]
    async fn test_session_log_handler() {
        let temp_dir = std::env::temp_dir();
        let log_path = temp_dir.join(format!("test_session_{}.log", Uuid::new_v4()));

        let config = SessionLogConfig {
            log_path: log_path.clone(),
            format: SessionLogFormat::Json,
            include_metadata: true,
            max_file_size_mb: None,
        };

        let handler = SessionLogHandler::new(config);

        let session = create_test_session();
        let ctx = HookContext::session_end(session);

        let result = handler.handle(&ctx).await.unwrap();
        assert!(result.is_continue());

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        let content = tokio::fs::read_to_string(&log_path).await.unwrap();
        assert!(content.contains("END"));

        let _ = tokio::fs::remove_file(log_path).await;
    }

    #[tokio::test]
    async fn test_metrics_export_handler_buffering() {
        let config = MetricsExportConfig {
            endpoint: String::new(),
            batch_size: 10,
            ..Default::default()
        };

        let handler = MetricsExportHandler::new(config);

        let metrics = create_test_metrics();
        let ctx = HookContext::metrics_collected(metrics);

        let result = handler.handle(&ctx).await.unwrap();
        assert!(result.is_continue());

        let buffer = handler.buffer.read().await;
        assert_eq!(buffer.len(), 1);
    }

    #[tokio::test]
    async fn test_webhook_handler_empty_url() {
        let config = WebhookConfig::default();
        let handler = WebhookHandler::new(config);

        let ctx = HookContext::new(Hook::PreSessionStart, HookData::None);
        let result = handler.handle(&ctx).await.unwrap();

        assert!(result.is_continue());
    }

    #[test]
    fn test_session_log_format_default() {
        let format = SessionLogFormat::default();
        assert!(matches!(format, SessionLogFormat::Json));
    }

    #[test]
    fn test_cost_alert_config_default() {
        let config = CostAlertConfig::default();
        assert_eq!(config.threshold_usd, 1.0);
        assert_eq!(config.alert_interval_seconds, 3600);
    }
}
