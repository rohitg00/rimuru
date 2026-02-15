//! Webhook Notifier Plugin
//!
//! Sends notifications to webhook URLs via HTTP POST requests.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

use crate::error::{RimuruError, RimuruResult};
use crate::plugins::traits::{Notification, NotificationLevel, NotifierPlugin, Plugin};
use crate::plugins::types::{PluginCapability, PluginConfig, PluginContext, PluginInfo};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookNotifierConfig {
    pub url: String,
    pub method: HttpMethod,
    pub headers: HashMap<String, String>,
    pub timeout_secs: u64,
    pub retry_count: u32,
    pub retry_delay_ms: u64,
    pub include_timestamp: bool,
    pub include_level: bool,
    pub custom_payload_template: Option<String>,
}

impl Default for WebhookNotifierConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            method: HttpMethod::Post,
            headers: HashMap::new(),
            timeout_secs: 30,
            retry_count: 3,
            retry_delay_ms: 1000,
            include_timestamp: true,
            include_level: true,
            custom_payload_template: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    Post,
    Put,
    Patch,
}

impl HttpMethod {
    fn as_str(&self) -> &'static str {
        match self {
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Patch => "PATCH",
        }
    }
}

#[derive(Debug, Serialize)]
struct WebhookPayload<'a> {
    title: &'a str,
    message: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    level: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    timestamp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<&'a serde_json::Value>,
    source: &'static str,
}

pub struct WebhookNotifierPlugin {
    info: PluginInfo,
    config: WebhookNotifierConfig,
    initialized: bool,
    client: Option<reqwest::Client>,
}

impl WebhookNotifierPlugin {
    pub fn new() -> Self {
        Self {
            info: PluginInfo::new("webhook-notifier", "1.0.0")
                .with_author("Rimuru Team")
                .with_description("Send notifications to webhook URLs")
                .with_capability(PluginCapability::Notifier),
            config: WebhookNotifierConfig::default(),
            initialized: false,
            client: None,
        }
    }

    fn create_client(&self) -> RimuruResult<reqwest::Client> {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(self.config.timeout_secs))
            .build()
            .map_err(|e| RimuruError::PluginError(format!("Failed to create HTTP client: {}", e)))
    }

    fn build_payload<'a>(&self, notification: &'a Notification) -> WebhookPayload<'a> {
        let level = if self.config.include_level {
            Some(match notification.level {
                NotificationLevel::Debug => "debug",
                NotificationLevel::Info => "info",
                NotificationLevel::Warning => "warning",
                NotificationLevel::Error => "error",
                NotificationLevel::Critical => "critical",
            })
        } else {
            None
        };

        let timestamp = if self.config.include_timestamp {
            Some(notification.timestamp.to_rfc3339())
        } else {
            None
        };

        WebhookPayload {
            title: &notification.title,
            message: &notification.message,
            level,
            timestamp,
            data: notification.data.as_ref(),
            source: "rimuru",
        }
    }

    async fn send_request(&self, payload: &WebhookPayload<'_>) -> RimuruResult<()> {
        let client = self.client.as_ref().ok_or_else(|| {
            RimuruError::PluginError("Webhook notifier not initialized".to_string())
        })?;

        if self.config.url.is_empty() {
            return Err(RimuruError::PluginConfigError {
                name: "webhook-notifier".to_string(),
                message: "Webhook URL is not configured".to_string(),
            });
        }

        let body = serde_json::to_string(payload)?;

        let mut request = match self.config.method {
            HttpMethod::Post => client.post(&self.config.url),
            HttpMethod::Put => client.put(&self.config.url),
            HttpMethod::Patch => client.patch(&self.config.url),
        };

        request = request.header("Content-Type", "application/json");

        for (key, value) in &self.config.headers {
            request = request.header(key, value);
        }

        request = request.body(body);

        let mut last_error = None;
        for attempt in 0..=self.config.retry_count {
            if attempt > 0 {
                tokio::time::sleep(Duration::from_millis(
                    self.config.retry_delay_ms * (attempt as u64),
                ))
                .await;
            }

            match request.try_clone() {
                Some(req) => match req.send().await {
                    Ok(response) => {
                        if response.status().is_success() {
                            return Ok(());
                        }
                        let status = response.status();
                        let body = response.text().await.unwrap_or_default();
                        last_error = Some(format!(
                            "HTTP {}: {}",
                            status,
                            body.chars().take(200).collect::<String>()
                        ));
                    }
                    Err(e) => {
                        last_error = Some(e.to_string());
                    }
                },
                None => {
                    return Err(RimuruError::PluginError(
                        "Failed to clone request for retry".to_string(),
                    ));
                }
            }
        }

        Err(RimuruError::PluginError(format!(
            "Webhook request failed after {} retries: {}",
            self.config.retry_count,
            last_error.unwrap_or_else(|| "Unknown error".to_string())
        )))
    }
}

impl Default for WebhookNotifierPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for WebhookNotifierPlugin {
    fn info(&self) -> &PluginInfo {
        &self.info
    }

    async fn init(&mut self, _ctx: &PluginContext) -> RimuruResult<()> {
        self.client = Some(self.create_client()?);
        self.initialized = true;
        Ok(())
    }

    async fn shutdown(&mut self) -> RimuruResult<()> {
        self.client = None;
        self.initialized = false;
        Ok(())
    }

    fn is_initialized(&self) -> bool {
        self.initialized
    }

    fn configure(&mut self, config: PluginConfig) -> RimuruResult<()> {
        if let Some(url) = config.get_setting::<String>("url") {
            self.config.url = url;
        }
        if let Some(method) = config.get_setting::<String>("method") {
            self.config.method = match method.to_uppercase().as_str() {
                "PUT" => HttpMethod::Put,
                "PATCH" => HttpMethod::Patch,
                _ => HttpMethod::Post,
            };
        }
        if let Some(headers) = config.get_setting::<HashMap<String, String>>("headers") {
            self.config.headers = headers;
        }
        if let Some(timeout_secs) = config.get_setting::<u64>("timeout_secs") {
            self.config.timeout_secs = timeout_secs.max(1).min(300);
        }
        if let Some(retry_count) = config.get_setting::<u32>("retry_count") {
            self.config.retry_count = retry_count.min(10);
        }
        if let Some(retry_delay_ms) = config.get_setting::<u64>("retry_delay_ms") {
            self.config.retry_delay_ms = retry_delay_ms.max(100).min(60000);
        }
        if let Some(include_timestamp) = config.get_setting::<bool>("include_timestamp") {
            self.config.include_timestamp = include_timestamp;
        }
        if let Some(include_level) = config.get_setting::<bool>("include_level") {
            self.config.include_level = include_level;
        }

        if self.initialized {
            self.client = Some(self.create_client()?);
        }

        Ok(())
    }

    fn config_schema(&self) -> Option<serde_json::Value> {
        Some(serde_json::json!({
            "type": "object",
            "required": ["url"],
            "properties": {
                "url": {
                    "type": "string",
                    "description": "Webhook URL to send notifications to",
                    "format": "uri"
                },
                "method": {
                    "type": "string",
                    "description": "HTTP method to use",
                    "enum": ["POST", "PUT", "PATCH"],
                    "default": "POST"
                },
                "headers": {
                    "type": "object",
                    "description": "Additional HTTP headers to include",
                    "additionalProperties": { "type": "string" }
                },
                "timeout_secs": {
                    "type": "integer",
                    "description": "Request timeout in seconds",
                    "default": 30,
                    "minimum": 1,
                    "maximum": 300
                },
                "retry_count": {
                    "type": "integer",
                    "description": "Number of retry attempts on failure",
                    "default": 3,
                    "minimum": 0,
                    "maximum": 10
                },
                "retry_delay_ms": {
                    "type": "integer",
                    "description": "Base delay between retries in milliseconds",
                    "default": 1000,
                    "minimum": 100,
                    "maximum": 60000
                },
                "include_timestamp": {
                    "type": "boolean",
                    "description": "Include timestamp in payload",
                    "default": true
                },
                "include_level": {
                    "type": "boolean",
                    "description": "Include notification level in payload",
                    "default": true
                }
            }
        }))
    }
}

#[async_trait]
impl NotifierPlugin for WebhookNotifierPlugin {
    fn notification_type(&self) -> &str {
        "webhook"
    }

    async fn send(&self, notification: Notification) -> RimuruResult<()> {
        let payload = self.build_payload(&notification);
        self.send_request(&payload).await
    }

    async fn test_connection(&self) -> RimuruResult<bool> {
        let test_notification =
            Notification::info("Connection Test", "This is a test notification from Rimuru");
        let payload = self.build_payload(&test_notification);

        match self.send_request(&payload).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_notification() -> Notification {
        Notification::info("Test Title", "Test message content")
    }

    #[test]
    fn test_webhook_notifier_new() {
        let plugin = WebhookNotifierPlugin::new();
        assert_eq!(plugin.info().name, "webhook-notifier");
        assert_eq!(plugin.info().version, "1.0.0");
        assert!(!plugin.is_initialized());
    }

    #[test]
    fn test_build_payload() {
        let plugin = WebhookNotifierPlugin::new();
        let notification = create_test_notification();
        let payload = plugin.build_payload(&notification);

        assert_eq!(payload.title, "Test Title");
        assert_eq!(payload.message, "Test message content");
        assert_eq!(payload.level, Some("info"));
        assert!(payload.timestamp.is_some());
        assert_eq!(payload.source, "rimuru");
    }

    #[test]
    fn test_build_payload_without_level() {
        let mut plugin = WebhookNotifierPlugin::new();
        plugin.config.include_level = false;

        let notification = create_test_notification();
        let payload = plugin.build_payload(&notification);

        assert!(payload.level.is_none());
    }

    #[test]
    fn test_build_payload_without_timestamp() {
        let mut plugin = WebhookNotifierPlugin::new();
        plugin.config.include_timestamp = false;

        let notification = create_test_notification();
        let payload = plugin.build_payload(&notification);

        assert!(payload.timestamp.is_none());
    }

    #[test]
    fn test_config_schema() {
        let plugin = WebhookNotifierPlugin::new();
        let schema = plugin.config_schema().unwrap();

        assert_eq!(schema["type"], "object");
        assert!(schema["required"]
            .as_array()
            .unwrap()
            .contains(&serde_json::json!("url")));
        assert!(schema["properties"]["url"].is_object());
        assert!(schema["properties"]["method"].is_object());
        assert!(schema["properties"]["headers"].is_object());
        assert!(schema["properties"]["timeout_secs"].is_object());
        assert!(schema["properties"]["retry_count"].is_object());
    }

    #[test]
    fn test_configure() {
        let mut plugin = WebhookNotifierPlugin::new();
        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), "Bearer token".to_string());

        let config = PluginConfig::new()
            .with_setting("url", "https://example.com/webhook")
            .with_setting("method", "PUT")
            .with_setting("headers", headers)
            .with_setting("timeout_secs", 60u64)
            .with_setting("retry_count", 5u32);

        plugin.configure(config).unwrap();

        assert_eq!(plugin.config.url, "https://example.com/webhook");
        assert!(matches!(plugin.config.method, HttpMethod::Put));
        assert_eq!(plugin.config.timeout_secs, 60);
        assert_eq!(plugin.config.retry_count, 5);
        assert!(plugin.config.headers.contains_key("Authorization"));
    }

    #[test]
    fn test_configure_clamps_values() {
        let mut plugin = WebhookNotifierPlugin::new();
        let config = PluginConfig::new()
            .with_setting("timeout_secs", 1000u64)
            .with_setting("retry_count", 100u32)
            .with_setting("retry_delay_ms", 1u64);

        plugin.configure(config).unwrap();

        assert_eq!(plugin.config.timeout_secs, 300);
        assert_eq!(plugin.config.retry_count, 10);
        assert_eq!(plugin.config.retry_delay_ms, 100);
    }

    #[tokio::test]
    async fn test_init_and_shutdown() {
        let mut plugin = WebhookNotifierPlugin::new();
        let ctx = PluginContext::new("webhook-notifier", "/tmp");

        assert!(!plugin.is_initialized());
        assert!(plugin.client.is_none());

        plugin.init(&ctx).await.unwrap();
        assert!(plugin.is_initialized());
        assert!(plugin.client.is_some());

        plugin.shutdown().await.unwrap();
        assert!(!plugin.is_initialized());
        assert!(plugin.client.is_none());
    }

    #[test]
    fn test_notification_type() {
        let plugin = WebhookNotifierPlugin::new();
        assert_eq!(plugin.notification_type(), "webhook");
    }

    #[test]
    fn test_http_method_as_str() {
        assert_eq!(HttpMethod::Post.as_str(), "POST");
        assert_eq!(HttpMethod::Put.as_str(), "PUT");
        assert_eq!(HttpMethod::Patch.as_str(), "PATCH");
    }

    #[tokio::test]
    async fn test_send_without_url() {
        let mut plugin = WebhookNotifierPlugin::new();
        let ctx = PluginContext::new("webhook-notifier", "/tmp");
        plugin.init(&ctx).await.unwrap();

        let notification = create_test_notification();
        let result = plugin.send(notification).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not configured"));
    }

    #[test]
    fn test_notification_levels() {
        let plugin = WebhookNotifierPlugin::new();

        let debug = Notification::new("Debug", "msg").with_level(NotificationLevel::Debug);
        assert_eq!(plugin.build_payload(&debug).level, Some("debug"));

        let warning = Notification::warning("Warning", "msg");
        assert_eq!(plugin.build_payload(&warning).level, Some("warning"));

        let error = Notification::error("Error", "msg");
        assert_eq!(plugin.build_payload(&error).level, Some("error"));

        let critical = Notification::critical("Critical", "msg");
        assert_eq!(plugin.build_payload(&critical).level, Some("critical"));
    }

    #[test]
    fn test_notification_with_data() {
        let plugin = WebhookNotifierPlugin::new();
        let notification =
            Notification::info("Test", "msg").with_data(serde_json::json!({"key": "value"}));
        let payload = plugin.build_payload(&notification);

        assert!(payload.data.is_some());
        assert_eq!(payload.data.unwrap()["key"], "value");
    }
}
