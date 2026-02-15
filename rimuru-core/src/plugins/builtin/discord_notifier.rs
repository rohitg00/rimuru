//! Discord Notifier Plugin
//!
//! Sends notifications to Discord channels using webhooks.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::error::{RimuruError, RimuruResult};
use crate::plugins::traits::{Notification, NotificationLevel, NotifierPlugin, Plugin};
use crate::plugins::types::{PluginCapability, PluginConfig, PluginContext, PluginInfo};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordNotifierConfig {
    pub webhook_url: String,
    pub username: String,
    pub avatar_url: Option<String>,
    pub timeout_secs: u64,
    pub mention_on_critical: bool,
    pub mention_role_id: Option<String>,
    pub thread_id: Option<String>,
}

impl Default for DiscordNotifierConfig {
    fn default() -> Self {
        Self {
            webhook_url: String::new(),
            username: "Rimuru".to_string(),
            avatar_url: None,
            timeout_secs: 30,
            mention_on_critical: true,
            mention_role_id: None,
            thread_id: None,
        }
    }
}

#[derive(Debug, Serialize)]
struct DiscordWebhookPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    avatar_url: Option<String>,
    embeds: Vec<DiscordEmbed>,
}

#[derive(Debug, Serialize)]
struct DiscordEmbed {
    title: String,
    description: String,
    color: u32,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    fields: Vec<DiscordField>,
    #[serde(skip_serializing_if = "Option::is_none")]
    footer: Option<DiscordFooter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    timestamp: Option<String>,
}

#[derive(Debug, Serialize)]
struct DiscordField {
    name: String,
    value: String,
    inline: bool,
}

#[derive(Debug, Serialize)]
struct DiscordFooter {
    text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    icon_url: Option<String>,
}

pub struct DiscordNotifierPlugin {
    info: PluginInfo,
    config: DiscordNotifierConfig,
    initialized: bool,
    client: Option<reqwest::Client>,
}

impl DiscordNotifierPlugin {
    pub fn new() -> Self {
        Self {
            info: PluginInfo::new("discord-notifier", "1.0.0")
                .with_author("Rimuru Team")
                .with_description("Send notifications to Discord channels")
                .with_capability(PluginCapability::Notifier),
            config: DiscordNotifierConfig::default(),
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

    fn level_to_color(level: NotificationLevel) -> u32 {
        match level {
            NotificationLevel::Debug => 0x808080,
            NotificationLevel::Info => 0x3498db,
            NotificationLevel::Warning => 0xf39c12,
            NotificationLevel::Error => 0xe74c3c,
            NotificationLevel::Critical => 0x9b59b6,
        }
    }

    fn level_to_emoji(level: NotificationLevel) -> &'static str {
        match level {
            NotificationLevel::Debug => "🐛",
            NotificationLevel::Info => "ℹ️",
            NotificationLevel::Warning => "⚠️",
            NotificationLevel::Error => "❌",
            NotificationLevel::Critical => "🚨",
        }
    }

    fn build_payload(&self, notification: &Notification) -> DiscordWebhookPayload {
        let content = if notification.level == NotificationLevel::Critical
            && self.config.mention_on_critical
        {
            if let Some(role_id) = &self.config.mention_role_id {
                Some(format!("<@&{}>", role_id))
            } else {
                Some("@here".to_string())
            }
        } else {
            None
        };

        let mut fields = Vec::new();

        if let Some(data) = &notification.data {
            if let Some(obj) = data.as_object() {
                for (key, value) in obj.iter().take(25) {
                    let value_str = match value {
                        serde_json::Value::String(s) => s.clone(),
                        _ => value.to_string(),
                    };
                    if value_str.len() <= 1024 {
                        fields.push(DiscordField {
                            name: key.chars().take(256).collect(),
                            value: value_str,
                            inline: true,
                        });
                    }
                }
            }
        }

        let title = format!(
            "{} {}",
            Self::level_to_emoji(notification.level),
            notification.title
        );

        let embed = DiscordEmbed {
            title,
            description: notification.message.chars().take(4096).collect(),
            color: Self::level_to_color(notification.level),
            fields,
            footer: Some(DiscordFooter {
                text: "Rimuru AI Agent Monitor".to_string(),
                icon_url: None,
            }),
            timestamp: Some(notification.timestamp.to_rfc3339()),
        };

        DiscordWebhookPayload {
            content,
            username: self.config.username.clone(),
            avatar_url: self.config.avatar_url.clone(),
            embeds: vec![embed],
        }
    }

    async fn send_to_discord(&self, payload: &DiscordWebhookPayload) -> RimuruResult<()> {
        let client = self.client.as_ref().ok_or_else(|| {
            RimuruError::PluginError("Discord notifier not initialized".to_string())
        })?;

        if self.config.webhook_url.is_empty() {
            return Err(RimuruError::PluginConfigError {
                name: "discord-notifier".to_string(),
                message: "Webhook URL is not configured".to_string(),
            });
        }

        let mut url = self.config.webhook_url.clone();
        if let Some(thread_id) = &self.config.thread_id {
            url = format!("{}?thread_id={}", url, thread_id);
        }

        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .json(payload)
            .send()
            .await
            .map_err(|e| {
                RimuruError::PluginError(format!("Failed to send Discord message: {}", e))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(RimuruError::PluginError(format!(
                "Discord API error {}: {}",
                status, body
            )));
        }

        Ok(())
    }
}

impl Default for DiscordNotifierPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for DiscordNotifierPlugin {
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
        if let Some(username) = config.get_setting::<String>("username") {
            self.config.username = username;
        }
        if let Some(avatar_url) = config.get_setting::<String>("avatar_url") {
            self.config.avatar_url = if avatar_url.is_empty() {
                None
            } else {
                Some(avatar_url)
            };
        }
        if let Some(timeout_secs) = config.get_setting::<u64>("timeout_secs") {
            self.config.timeout_secs = timeout_secs.max(1).min(300);
        }
        if let Some(mention_on_critical) = config.get_setting::<bool>("mention_on_critical") {
            self.config.mention_on_critical = mention_on_critical;
        }
        if let Some(mention_role_id) = config.get_setting::<String>("mention_role_id") {
            self.config.mention_role_id = if mention_role_id.is_empty() {
                None
            } else {
                Some(mention_role_id)
            };
        }
        if let Some(thread_id) = config.get_setting::<String>("thread_id") {
            self.config.thread_id = if thread_id.is_empty() {
                None
            } else {
                Some(thread_id)
            };
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
                    "description": "Discord Webhook URL",
                    "format": "uri"
                },
                "username": {
                    "type": "string",
                    "description": "Bot username to display",
                    "default": "Rimuru"
                },
                "avatar_url": {
                    "type": "string",
                    "description": "Avatar URL for the bot",
                    "format": "uri"
                },
                "timeout_secs": {
                    "type": "integer",
                    "description": "Request timeout in seconds",
                    "default": 30,
                    "minimum": 1,
                    "maximum": 300
                },
                "mention_on_critical": {
                    "type": "boolean",
                    "description": "Mention role or @here on critical notifications",
                    "default": true
                },
                "mention_role_id": {
                    "type": "string",
                    "description": "Discord role ID to mention on critical (uses @here if not set)"
                },
                "thread_id": {
                    "type": "string",
                    "description": "Send messages to a specific thread (optional)"
                }
            }
        }))
    }
}

#[async_trait]
impl NotifierPlugin for DiscordNotifierPlugin {
    fn notification_type(&self) -> &str {
        "discord"
    }

    async fn send(&self, notification: Notification) -> RimuruResult<()> {
        let payload = self.build_payload(&notification);
        self.send_to_discord(&payload).await
    }

    async fn test_connection(&self) -> RimuruResult<bool> {
        let test_notification = Notification::info(
            "Connection Test",
            "This is a test notification from Rimuru AI Agent Monitor",
        );
        let payload = self.build_payload(&test_notification);

        match self.send_to_discord(&payload).await {
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
    fn test_discord_notifier_new() {
        let plugin = DiscordNotifierPlugin::new();
        assert_eq!(plugin.info().name, "discord-notifier");
        assert_eq!(plugin.info().version, "1.0.0");
        assert!(!plugin.is_initialized());
    }

    #[test]
    fn test_level_to_color() {
        assert_eq!(
            DiscordNotifierPlugin::level_to_color(NotificationLevel::Debug),
            0x808080
        );
        assert_eq!(
            DiscordNotifierPlugin::level_to_color(NotificationLevel::Info),
            0x3498db
        );
        assert_eq!(
            DiscordNotifierPlugin::level_to_color(NotificationLevel::Warning),
            0xf39c12
        );
        assert_eq!(
            DiscordNotifierPlugin::level_to_color(NotificationLevel::Error),
            0xe74c3c
        );
        assert_eq!(
            DiscordNotifierPlugin::level_to_color(NotificationLevel::Critical),
            0x9b59b6
        );
    }

    #[test]
    fn test_level_to_emoji() {
        assert_eq!(
            DiscordNotifierPlugin::level_to_emoji(NotificationLevel::Debug),
            "🐛"
        );
        assert_eq!(
            DiscordNotifierPlugin::level_to_emoji(NotificationLevel::Info),
            "ℹ️"
        );
        assert_eq!(
            DiscordNotifierPlugin::level_to_emoji(NotificationLevel::Warning),
            "⚠️"
        );
        assert_eq!(
            DiscordNotifierPlugin::level_to_emoji(NotificationLevel::Error),
            "❌"
        );
        assert_eq!(
            DiscordNotifierPlugin::level_to_emoji(NotificationLevel::Critical),
            "🚨"
        );
    }

    #[test]
    fn test_build_payload_basic() {
        let plugin = DiscordNotifierPlugin::new();
        let notification = create_test_notification();
        let payload = plugin.build_payload(&notification);

        assert_eq!(payload.username, "Rimuru");
        assert!(payload.content.is_none());
        assert_eq!(payload.embeds.len(), 1);

        let embed = &payload.embeds[0];
        assert!(embed.title.contains("Test Alert"));
        assert_eq!(embed.description, "This is a test message");
        assert_eq!(embed.color, 0x3498db);
    }

    #[test]
    fn test_build_payload_critical_with_mention() {
        let plugin = DiscordNotifierPlugin::new();
        let notification = Notification::critical("Critical Alert", "System is down");
        let payload = plugin.build_payload(&notification);

        assert_eq!(payload.content, Some("@here".to_string()));
    }

    #[test]
    fn test_build_payload_critical_with_role_mention() {
        let mut plugin = DiscordNotifierPlugin::new();
        plugin.config.mention_role_id = Some("123456789".to_string());

        let notification = Notification::critical("Critical Alert", "System is down");
        let payload = plugin.build_payload(&notification);

        assert_eq!(payload.content, Some("<@&123456789>".to_string()));
    }

    #[test]
    fn test_build_payload_critical_without_mention() {
        let mut plugin = DiscordNotifierPlugin::new();
        plugin.config.mention_on_critical = false;

        let notification = Notification::critical("Critical Alert", "System is down");
        let payload = plugin.build_payload(&notification);

        assert!(payload.content.is_none());
    }

    #[test]
    fn test_build_payload_with_data() {
        let plugin = DiscordNotifierPlugin::new();
        let notification = Notification::info("Test", "msg").with_data(serde_json::json!({
            "agent": "claude-code",
            "cost": 1.23
        }));
        let payload = plugin.build_payload(&notification);

        let embed = &payload.embeds[0];
        assert_eq!(embed.fields.len(), 2);
    }

    #[test]
    fn test_config_schema() {
        let plugin = DiscordNotifierPlugin::new();
        let schema = plugin.config_schema().unwrap();

        assert_eq!(schema["type"], "object");
        assert!(schema["required"]
            .as_array()
            .unwrap()
            .contains(&serde_json::json!("webhook_url")));
        assert!(schema["properties"]["webhook_url"].is_object());
        assert!(schema["properties"]["username"].is_object());
        assert!(schema["properties"]["mention_role_id"].is_object());
        assert!(schema["properties"]["thread_id"].is_object());
    }

    #[test]
    fn test_configure() {
        let mut plugin = DiscordNotifierPlugin::new();
        let config = PluginConfig::new()
            .with_setting("webhook_url", "https://discord.com/api/webhooks/xxx/yyy")
            .with_setting("username", "RimuruBot")
            .with_setting("avatar_url", "https://example.com/avatar.png")
            .with_setting("mention_on_critical", false)
            .with_setting("mention_role_id", "123456")
            .with_setting("thread_id", "987654");

        plugin.configure(config).unwrap();

        assert_eq!(
            plugin.config.webhook_url,
            "https://discord.com/api/webhooks/xxx/yyy"
        );
        assert_eq!(plugin.config.username, "RimuruBot");
        assert_eq!(
            plugin.config.avatar_url,
            Some("https://example.com/avatar.png".to_string())
        );
        assert!(!plugin.config.mention_on_critical);
        assert_eq!(plugin.config.mention_role_id, Some("123456".to_string()));
        assert_eq!(plugin.config.thread_id, Some("987654".to_string()));
    }

    #[test]
    fn test_configure_empty_optionals() {
        let mut plugin = DiscordNotifierPlugin::new();
        plugin.config.avatar_url = Some("https://example.com/avatar.png".to_string());
        plugin.config.mention_role_id = Some("123".to_string());
        plugin.config.thread_id = Some("456".to_string());

        let config = PluginConfig::new()
            .with_setting("avatar_url", "")
            .with_setting("mention_role_id", "")
            .with_setting("thread_id", "");

        plugin.configure(config).unwrap();

        assert!(plugin.config.avatar_url.is_none());
        assert!(plugin.config.mention_role_id.is_none());
        assert!(plugin.config.thread_id.is_none());
    }

    #[tokio::test]
    async fn test_init_and_shutdown() {
        let mut plugin = DiscordNotifierPlugin::new();
        let ctx = PluginContext::new("discord-notifier", "/tmp");

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
        let plugin = DiscordNotifierPlugin::new();
        assert_eq!(plugin.notification_type(), "discord");
    }

    #[tokio::test]
    async fn test_send_without_webhook_url() {
        let mut plugin = DiscordNotifierPlugin::new();
        let ctx = PluginContext::new("discord-notifier", "/tmp");
        plugin.init(&ctx).await.unwrap();

        let notification = create_test_notification();
        let result = plugin.send(notification).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not configured"));
    }

    #[test]
    fn test_discord_payload_serialization() {
        let plugin = DiscordNotifierPlugin::new();
        let notification = create_test_notification();
        let payload = plugin.build_payload(&notification);

        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("username"));
        assert!(json.contains("embeds"));
        assert!(json.contains("Rimuru"));
    }

    #[test]
    fn test_embed_fields_limit() {
        let plugin = DiscordNotifierPlugin::new();

        let mut data = serde_json::Map::new();
        for i in 0..30 {
            data.insert(
                format!("field_{}", i),
                serde_json::json!(format!("value_{}", i)),
            );
        }

        let notification =
            Notification::info("Test", "msg").with_data(serde_json::Value::Object(data));
        let payload = plugin.build_payload(&notification);

        let embed = &payload.embeds[0];
        assert!(embed.fields.len() <= 25);
    }

    #[test]
    fn test_embed_with_avatar() {
        let mut plugin = DiscordNotifierPlugin::new();
        plugin.config.avatar_url = Some("https://example.com/avatar.png".to_string());

        let notification = create_test_notification();
        let payload = plugin.build_payload(&notification);

        assert_eq!(
            payload.avatar_url,
            Some("https://example.com/avatar.png".to_string())
        );
    }

    #[test]
    fn test_embed_timestamp() {
        let plugin = DiscordNotifierPlugin::new();
        let notification = create_test_notification();
        let payload = plugin.build_payload(&notification);

        let embed = &payload.embeds[0];
        assert!(embed.timestamp.is_some());
    }

    #[test]
    fn test_embed_footer() {
        let plugin = DiscordNotifierPlugin::new();
        let notification = create_test_notification();
        let payload = plugin.build_payload(&notification);

        let embed = &payload.embeds[0];
        assert!(embed.footer.is_some());
        assert_eq!(
            embed.footer.as_ref().unwrap().text,
            "Rimuru AI Agent Monitor"
        );
    }
}
