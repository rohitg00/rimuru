//! Slack Notifier Plugin
//!
//! Sends notifications to Slack channels using webhooks or the Slack API.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::error::{RimuruError, RimuruResult};
use crate::plugins::traits::{Notification, NotificationLevel, NotifierPlugin, Plugin};
use crate::plugins::types::{PluginCapability, PluginConfig, PluginContext, PluginInfo};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackNotifierConfig {
    pub webhook_url: String,
    pub channel: Option<String>,
    pub username: String,
    pub icon_emoji: String,
    pub timeout_secs: u64,
    pub include_timestamp: bool,
    pub mention_on_critical: bool,
    pub mention_users: Vec<String>,
}

impl Default for SlackNotifierConfig {
    fn default() -> Self {
        Self {
            webhook_url: String::new(),
            channel: None,
            username: "Rimuru".to_string(),
            icon_emoji: ":robot_face:".to_string(),
            timeout_secs: 30,
            include_timestamp: true,
            mention_on_critical: true,
            mention_users: vec!["here".to_string()],
        }
    }
}

#[derive(Debug, Serialize)]
struct SlackMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    channel: Option<String>,
    username: String,
    icon_emoji: String,
    attachments: Vec<SlackAttachment>,
}

#[derive(Debug, Serialize)]
struct SlackAttachment {
    color: String,
    title: String,
    text: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    fields: Vec<SlackField>,
    #[serde(skip_serializing_if = "Option::is_none")]
    footer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ts: Option<i64>,
}

#[derive(Debug, Serialize)]
struct SlackField {
    title: String,
    value: String,
    short: bool,
}

pub struct SlackNotifierPlugin {
    info: PluginInfo,
    config: SlackNotifierConfig,
    initialized: bool,
    client: Option<reqwest::Client>,
}

impl SlackNotifierPlugin {
    pub fn new() -> Self {
        Self {
            info: PluginInfo::new("slack-notifier", "1.0.0")
                .with_author("Rimuru Team")
                .with_description("Send notifications to Slack channels")
                .with_capability(PluginCapability::Notifier),
            config: SlackNotifierConfig::default(),
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

    fn level_to_color(level: NotificationLevel) -> &'static str {
        match level {
            NotificationLevel::Debug => "#808080",
            NotificationLevel::Info => "#36a64f",
            NotificationLevel::Warning => "#daa520",
            NotificationLevel::Error => "#dc3545",
            NotificationLevel::Critical => "#8b0000",
        }
    }

    fn level_to_emoji(level: NotificationLevel) -> &'static str {
        match level {
            NotificationLevel::Debug => ":bug:",
            NotificationLevel::Info => ":information_source:",
            NotificationLevel::Warning => ":warning:",
            NotificationLevel::Error => ":x:",
            NotificationLevel::Critical => ":rotating_light:",
        }
    }

    fn build_message(&self, notification: &Notification) -> SlackMessage {
        let mut text = notification.message.clone();

        if notification.level == NotificationLevel::Critical && self.config.mention_on_critical {
            let mentions: String = self
                .config
                .mention_users
                .iter()
                .map(|u| {
                    if u == "here" || u == "channel" || u == "everyone" {
                        format!("<!{}>", u)
                    } else {
                        format!("<@{}>", u)
                    }
                })
                .collect::<Vec<_>>()
                .join(" ");
            text = format!("{} {}", mentions, text);
        }

        let mut fields = Vec::new();

        if let Some(data) = &notification.data {
            if let Some(obj) = data.as_object() {
                for (key, value) in obj.iter().take(10) {
                    fields.push(SlackField {
                        title: key.clone(),
                        value: match value {
                            serde_json::Value::String(s) => s.clone(),
                            _ => value.to_string(),
                        },
                        short: true,
                    });
                }
            }
        }

        let ts = if self.config.include_timestamp {
            Some(notification.timestamp.timestamp())
        } else {
            None
        };

        let title = format!(
            "{} {}",
            Self::level_to_emoji(notification.level),
            notification.title
        );

        let attachment = SlackAttachment {
            color: Self::level_to_color(notification.level).to_string(),
            title,
            text,
            fields,
            footer: Some("Rimuru AI Agent Monitor".to_string()),
            ts,
        };

        SlackMessage {
            channel: self.config.channel.clone(),
            username: self.config.username.clone(),
            icon_emoji: self.config.icon_emoji.clone(),
            attachments: vec![attachment],
        }
    }

    async fn send_to_slack(&self, message: &SlackMessage) -> RimuruResult<()> {
        let client = self.client.as_ref().ok_or_else(|| {
            RimuruError::PluginError("Slack notifier not initialized".to_string())
        })?;

        if self.config.webhook_url.is_empty() {
            return Err(RimuruError::PluginConfigError {
                name: "slack-notifier".to_string(),
                message: "Webhook URL is not configured".to_string(),
            });
        }

        let response = client
            .post(&self.config.webhook_url)
            .header("Content-Type", "application/json")
            .json(message)
            .send()
            .await
            .map_err(|e| {
                RimuruError::PluginError(format!("Failed to send Slack message: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(RimuruError::PluginError(format!(
                "Slack API error {}: {}",
                status, body
            )));
        }

        Ok(())
    }
}

impl Default for SlackNotifierPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for SlackNotifierPlugin {
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
        if let Some(webhook_url) = config.get_setting::<String>("webhook_url") {
            self.config.webhook_url = webhook_url;
        }
        if let Some(channel) = config.get_setting::<String>("channel") {
            self.config.channel = if channel.is_empty() {
                None
            } else {
                Some(channel)
            };
        }
        if let Some(username) = config.get_setting::<String>("username") {
            self.config.username = username;
        }
        if let Some(icon_emoji) = config.get_setting::<String>("icon_emoji") {
            self.config.icon_emoji = icon_emoji;
        }
        if let Some(timeout_secs) = config.get_setting::<u64>("timeout_secs") {
            self.config.timeout_secs = timeout_secs.max(1).min(300);
        }
        if let Some(include_timestamp) = config.get_setting::<bool>("include_timestamp") {
            self.config.include_timestamp = include_timestamp;
        }
        if let Some(mention_on_critical) = config.get_setting::<bool>("mention_on_critical") {
            self.config.mention_on_critical = mention_on_critical;
        }
        if let Some(mention_users) = config.get_setting::<Vec<String>>("mention_users") {
            self.config.mention_users = mention_users;
        }

        if self.initialized {
            self.client = Some(self.create_client()?);
        }

        Ok(())
    }

    fn config_schema(&self) -> Option<serde_json::Value> {
        Some(serde_json::json!({
            "type": "object",
            "required": ["webhook_url"],
            "properties": {
                "webhook_url": {
                    "type": "string",
                    "description": "Slack Incoming Webhook URL",
                    "format": "uri"
                },
                "channel": {
                    "type": "string",
                    "description": "Override the default channel (optional)"
                },
                "username": {
                    "type": "string",
                    "description": "Bot username to display",
                    "default": "Rimuru"
                },
                "icon_emoji": {
                    "type": "string",
                    "description": "Emoji to use as bot icon",
                    "default": ":robot_face:"
                },
                "timeout_secs": {
                    "type": "integer",
                    "description": "Request timeout in seconds",
                    "default": 30,
                    "minimum": 1,
                    "maximum": 300
                },
                "include_timestamp": {
                    "type": "boolean",
                    "description": "Include timestamp in message",
                    "default": true
                },
                "mention_on_critical": {
                    "type": "boolean",
                    "description": "Mention users on critical notifications",
                    "default": true
                },
                "mention_users": {
                    "type": "array",
                    "description": "Users/groups to mention (use 'here', 'channel', or user IDs)",
                    "items": { "type": "string" },
                    "default": ["here"]
                }
            }
        }))
    }
}

#[async_trait]
impl NotifierPlugin for SlackNotifierPlugin {
    fn notification_type(&self) -> &str {
        "slack"
    }

    async fn send(&self, notification: Notification) -> RimuruResult<()> {
        let message = self.build_message(&notification);
        self.send_to_slack(&message).await
    }

    async fn test_connection(&self) -> RimuruResult<bool> {
        let test_notification = Notification::info(
            "Connection Test",
            "This is a test notification from Rimuru AI Agent Monitor",
        );
        let message = self.build_message(&test_notification);

        match self.send_to_slack(&message).await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_notification() -> Notification {
        Notification::info("Test Alert", "This is a test message")
    }

    #[test]
    fn test_slack_notifier_new() {
        let plugin = SlackNotifierPlugin::new();
        assert_eq!(plugin.info().name, "slack-notifier");
        assert_eq!(plugin.info().version, "1.0.0");
        assert!(!plugin.is_initialized());
    }

    #[test]
    fn test_level_to_color() {
        assert_eq!(
            SlackNotifierPlugin::level_to_color(NotificationLevel::Debug),
            "#808080"
        );
        assert_eq!(
            SlackNotifierPlugin::level_to_color(NotificationLevel::Info),
            "#36a64f"
        );
        assert_eq!(
            SlackNotifierPlugin::level_to_color(NotificationLevel::Warning),
            "#daa520"
        );
        assert_eq!(
            SlackNotifierPlugin::level_to_color(NotificationLevel::Error),
            "#dc3545"
        );
        assert_eq!(
            SlackNotifierPlugin::level_to_color(NotificationLevel::Critical),
            "#8b0000"
        );
    }

    #[test]
    fn test_level_to_emoji() {
        assert_eq!(
            SlackNotifierPlugin::level_to_emoji(NotificationLevel::Debug),
            ":bug:"
        );
        assert_eq!(
            SlackNotifierPlugin::level_to_emoji(NotificationLevel::Info),
            ":information_source:"
        );
        assert_eq!(
            SlackNotifierPlugin::level_to_emoji(NotificationLevel::Warning),
            ":warning:"
        );
        assert_eq!(
            SlackNotifierPlugin::level_to_emoji(NotificationLevel::Error),
            ":x:"
        );
        assert_eq!(
            SlackNotifierPlugin::level_to_emoji(NotificationLevel::Critical),
            ":rotating_light:"
        );
    }

    #[test]
    fn test_build_message_basic() {
        let plugin = SlackNotifierPlugin::new();
        let notification = create_test_notification();
        let message = plugin.build_message(&notification);

        assert_eq!(message.username, "Rimuru");
        assert_eq!(message.icon_emoji, ":robot_face:");
        assert_eq!(message.attachments.len(), 1);

        let attachment = &message.attachments[0];
        assert!(attachment.title.contains("Test Alert"));
        assert_eq!(attachment.text, "This is a test message");
        assert_eq!(attachment.color, "#36a64f");
    }

    #[test]
    fn test_build_message_critical_with_mention() {
        let plugin = SlackNotifierPlugin::new();
        let notification = Notification::critical("Critical Alert", "System is down");
        let message = plugin.build_message(&notification);

        let attachment = &message.attachments[0];
        assert!(attachment.text.contains("<!here>"));
        assert!(attachment.text.contains("System is down"));
        assert_eq!(attachment.color, "#8b0000");
    }

    #[test]
    fn test_build_message_critical_without_mention() {
        let mut plugin = SlackNotifierPlugin::new();
        plugin.config.mention_on_critical = false;

        let notification = Notification::critical("Critical Alert", "System is down");
        let message = plugin.build_message(&notification);

        let attachment = &message.attachments[0];
        assert!(!attachment.text.contains("<!here>"));
        assert_eq!(attachment.text, "System is down");
    }

    #[test]
    fn test_build_message_with_custom_mentions() {
        let mut plugin = SlackNotifierPlugin::new();
        plugin.config.mention_users = vec!["U12345".to_string(), "channel".to_string()];

        let notification = Notification::critical("Alert", "msg");
        let message = plugin.build_message(&notification);

        let attachment = &message.attachments[0];
        assert!(attachment.text.contains("<@U12345>"));
        assert!(attachment.text.contains("<!channel>"));
    }

    #[test]
    fn test_build_message_with_data() {
        let plugin = SlackNotifierPlugin::new();
        let notification = Notification::info("Test", "msg").with_data(serde_json::json!({
            "agent": "claude-code",
            "cost": 1.23
        }));
        let message = plugin.build_message(&notification);

        let attachment = &message.attachments[0];
        assert_eq!(attachment.fields.len(), 2);
    }

    #[test]
    fn test_build_message_without_timestamp() {
        let mut plugin = SlackNotifierPlugin::new();
        plugin.config.include_timestamp = false;

        let notification = create_test_notification();
        let message = plugin.build_message(&notification);

        let attachment = &message.attachments[0];
        assert!(attachment.ts.is_none());
    }

    #[test]
    fn test_config_schema() {
        let plugin = SlackNotifierPlugin::new();
        let schema = plugin.config_schema().unwrap();

        assert_eq!(schema["type"], "object");
        assert!(schema["required"]
            .as_array()
            .unwrap()
            .contains(&serde_json::json!("webhook_url")));
        assert!(schema["properties"]["webhook_url"].is_object());
        assert!(schema["properties"]["channel"].is_object());
        assert!(schema["properties"]["username"].is_object());
        assert!(schema["properties"]["mention_users"].is_object());
    }

    #[test]
    fn test_configure() {
        let mut plugin = SlackNotifierPlugin::new();
        let config = PluginConfig::new()
            .with_setting("webhook_url", "https://hooks.slack.com/services/xxx")
            .with_setting("channel", "#alerts")
            .with_setting("username", "RimuruBot")
            .with_setting("icon_emoji", ":slime:")
            .with_setting("mention_on_critical", false)
            .with_setting("mention_users", vec!["U123"]);

        plugin.configure(config).unwrap();

        assert_eq!(
            plugin.config.webhook_url,
            "https://hooks.slack.com/services/xxx"
        );
        assert_eq!(plugin.config.channel, Some("#alerts".to_string()));
        assert_eq!(plugin.config.username, "RimuruBot");
        assert_eq!(plugin.config.icon_emoji, ":slime:");
        assert!(!plugin.config.mention_on_critical);
        assert_eq!(plugin.config.mention_users, vec!["U123"]);
    }

    #[tokio::test]
    async fn test_init_and_shutdown() {
        let mut plugin = SlackNotifierPlugin::new();
        let ctx = PluginContext::new("slack-notifier", "/tmp");

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
        let plugin = SlackNotifierPlugin::new();
        assert_eq!(plugin.notification_type(), "slack");
    }

    #[tokio::test]
    async fn test_send_without_webhook_url() {
        let mut plugin = SlackNotifierPlugin::new();
        let ctx = PluginContext::new("slack-notifier", "/tmp");
        plugin.init(&ctx).await.unwrap();

        let notification = create_test_notification();
        let result = plugin.send(notification).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not configured"));
    }

    #[test]
    fn test_slack_message_serialization() {
        let plugin = SlackNotifierPlugin::new();
        let notification = create_test_notification();
        let message = plugin.build_message(&notification);

        let json = serde_json::to_string(&message).unwrap();
        assert!(json.contains("username"));
        assert!(json.contains("attachments"));
        assert!(json.contains("Rimuru"));
    }

    #[test]
    fn test_custom_channel() {
        let mut plugin = SlackNotifierPlugin::new();
        plugin.config.channel = Some("#monitoring".to_string());

        let notification = create_test_notification();
        let message = plugin.build_message(&notification);

        assert_eq!(message.channel, Some("#monitoring".to_string()));
    }

    #[test]
    fn test_configure_empty_channel() {
        let mut plugin = SlackNotifierPlugin::new();
        plugin.config.channel = Some("#test".to_string());

        let config = PluginConfig::new().with_setting("channel", "");
        plugin.configure(config).unwrap();

        assert!(plugin.config.channel.is_none());
    }
}
