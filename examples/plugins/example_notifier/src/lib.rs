use rimuru_plugin_sdk::*;

define_notifier!(
    TeamsNotifierPlugin,
    name: "teams-notifier",
    version: "0.1.0",
    notification_type: "teams",
    author: "Example Author",
    description: "Send notifications to Microsoft Teams channels via incoming webhooks"
);

impl_plugin_base!(TeamsNotifierPlugin);

fn notification_level_to_string(level: &NotificationLevel) -> &'static str {
    match level {
        NotificationLevel::Debug => "Debug",
        NotificationLevel::Info => "Info",
        NotificationLevel::Warning => "Warning",
        NotificationLevel::Error => "Error",
        NotificationLevel::Critical => "Critical",
    }
}

fn notification_level_to_color(level: &NotificationLevel) -> &'static str {
    match level {
        NotificationLevel::Debug => "808080",
        NotificationLevel::Info => "0078D7",
        NotificationLevel::Warning => "FFA500",
        NotificationLevel::Error => "FF0000",
        NotificationLevel::Critical => "8B0000",
    }
}

#[async_trait]
impl NotifierPlugin for TeamsNotifierPlugin {
    fn notification_type(&self) -> &str {
        self.notif_type()
    }

    async fn send(&self, notification: Notification) -> RimuruResult<()> {
        let webhook_url: String = self.config.get_setting("webhook_url")
            .ok_or_else(|| RimuruError::plugin("webhook_url not configured"))?;

        let color = notification_level_to_color(&notification.level);

        let mention_user: Option<String> = self.config.get_setting("mention_user");
        let mention_text = mention_user
            .map(|u| format!("<at>{}</at> ", u))
            .unwrap_or_default();

        let card = json!({
            "@type": "MessageCard",
            "@context": "http://schema.org/extensions",
            "themeColor": color,
            "summary": notification.title,
            "sections": [{
                "activityTitle": format!("{}{}", mention_text, notification.title),
                "activitySubtitle": notification.timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                "activityImage": "https://adaptivecards.io/content/cats/1.png",
                "facts": [
                    {
                        "name": "Level",
                        "value": notification_level_to_string(&notification.level)
                    },
                    {
                        "name": "Message",
                        "value": notification.message
                    }
                ],
                "markdown": true
            }],
            "potentialAction": [{
                "@type": "OpenUri",
                "name": "View in Rimuru",
                "targets": [{
                    "os": "default",
                    "uri": "https://rimuru.local/dashboard"
                }]
            }]
        });

        let client = reqwest::Client::new();
        let response = client
            .post(&webhook_url)
            .header("Content-Type", "application/json")
            .json(&card)
            .send()
            .await
            .map_err(|e| RimuruError::plugin(format!("Failed to send Teams notification: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(RimuruError::plugin(format!(
                "Teams API error: {} - {}",
                status, body
            )));
        }

        info!(
            "Teams notification sent: {} - {}",
            notification.title, notification_level_to_string(&notification.level)
        );

        Ok(())
    }

    async fn send_batch(&self, notifications: Vec<Notification>) -> RimuruResult<()> {
        for notification in notifications {
            self.send(notification).await?;

            let rate_limit: u64 = self.config.get_setting("rate_limit_ms").unwrap_or(1000);
            tokio::time::sleep(tokio::time::Duration::from_millis(rate_limit)).await;
        }
        Ok(())
    }

    async fn test_connection(&self) -> RimuruResult<bool> {
        let webhook_url: Option<String> = self.config.get_setting("webhook_url");

        if webhook_url.is_none() {
            return Ok(false);
        }

        let test_notification = Notification::info(
            "Rimuru Connection Test",
            "This is a test notification from Rimuru to verify the Teams webhook is working."
        );

        match self.send(test_notification).await {
            Ok(()) => Ok(true),
            Err(e) => {
                warn!("Teams connection test failed: {}", e);
                Ok(false)
            }
        }
    }
}

impl TeamsNotifierPlugin {
    pub fn config_schema(&self) -> Option<serde_json::Value> {
        Some(helpers::create_config_schema(json!({
            "webhook_url": helpers::string_property("Microsoft Teams Incoming Webhook URL", None),
            "mention_user": helpers::string_property("User email to mention in critical notifications", None),
            "rate_limit_ms": helpers::integer_property("Minimum milliseconds between notifications", Some(1000)),
            "include_metadata": helpers::boolean_property("Include notification metadata in message", Some(false)),
            "theme_color_override": helpers::string_property("Override theme color (hex without #)", None)
        })))
    }

    pub fn with_webhook_url(mut self, url: &str) -> Self {
        self.config = self.config.with_setting("webhook_url", url);
        self
    }

    pub fn with_mention_user(mut self, email: &str) -> Self {
        self.config = self.config.with_setting("mention_user", email);
        self
    }
}

pub fn create_plugin() -> Box<dyn Plugin> {
    Box::new(TeamsNotifierPlugin::new())
}

pub fn create_notifier_plugin() -> Box<dyn NotifierPlugin> {
    Box::new(TeamsNotifierPlugin::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_plugin_info() {
        let plugin = TeamsNotifierPlugin::new();
        let info = plugin.info();

        assert_eq!(info.name, "teams-notifier");
        assert_eq!(info.version, "0.1.0");
        assert!(info.capabilities.contains(&PluginCapability::Notifier));
    }

    #[tokio::test]
    async fn test_notification_type() {
        let plugin = TeamsNotifierPlugin::new();

        assert_eq!(plugin.notification_type(), "teams");
    }

    #[tokio::test]
    async fn test_send_without_config() {
        let plugin = TeamsNotifierPlugin::new();
        let notification = Notification::info("Test", "Test message");

        let result = plugin.send(notification).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_test_connection_without_config() {
        let plugin = TeamsNotifierPlugin::new();

        let result = plugin.test_connection().await.unwrap();
        assert!(!result);
    }

    #[tokio::test]
    async fn test_config_schema() {
        let plugin = TeamsNotifierPlugin::new();
        let schema = plugin.config_schema();

        assert!(schema.is_some());
        let schema = schema.unwrap();
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["webhook_url"].is_object());
    }

    #[test]
    fn test_notification_level_to_string_fn() {
        assert_eq!(notification_level_to_string(&NotificationLevel::Debug), "Debug");
        assert_eq!(notification_level_to_string(&NotificationLevel::Info), "Info");
        assert_eq!(notification_level_to_string(&NotificationLevel::Warning), "Warning");
        assert_eq!(notification_level_to_string(&NotificationLevel::Error), "Error");
        assert_eq!(notification_level_to_string(&NotificationLevel::Critical), "Critical");
    }

    #[test]
    fn test_builder_methods() {
        let plugin = TeamsNotifierPlugin::new()
            .with_webhook_url("https://example.com/webhook")
            .with_mention_user("user@example.com");

        let webhook: Option<String> = plugin.config.get_setting("webhook_url");
        let mention: Option<String> = plugin.config.get_setting("mention_user");

        assert_eq!(webhook, Some("https://example.com/webhook".to_string()));
        assert_eq!(mention, Some("user@example.com".to_string()));
    }
}
