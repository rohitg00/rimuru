//! Built-in Plugins
//!
//! This module contains built-in plugins that come bundled with Rimuru.
//! These plugins provide essential functionality for data export and notifications.
//!
//! ## Exporter Plugins
//!
//! - [`CsvExporterPlugin`]: Export sessions and costs to CSV format
//! - [`JsonExporterPlugin`]: Export sessions and costs to JSON format
//!
//! ## Notifier Plugins
//!
//! - [`WebhookNotifierPlugin`]: Send notifications to generic webhook URLs
//! - [`SlackNotifierPlugin`]: Send notifications to Slack channels
//! - [`DiscordNotifierPlugin`]: Send notifications to Discord channels

mod csv_exporter;
mod discord_notifier;
mod json_exporter;
mod slack_notifier;
mod webhook_notifier;

pub use csv_exporter::{CsvExporterConfig, CsvExporterPlugin, LineEnding};
pub use discord_notifier::{DiscordNotifierConfig, DiscordNotifierPlugin};
pub use json_exporter::{JsonExporterConfig, JsonExporterPlugin};
pub use slack_notifier::{SlackNotifierConfig, SlackNotifierPlugin};
pub use webhook_notifier::{HttpMethod, WebhookNotifierConfig, WebhookNotifierPlugin};

use super::traits::{ExporterPlugin, NotifierPlugin, Plugin};
use super::types::PluginInfo;

pub fn list_builtin_plugins() -> Vec<PluginInfo> {
    vec![
        CsvExporterPlugin::new().info().clone(),
        JsonExporterPlugin::new().info().clone(),
        WebhookNotifierPlugin::new().info().clone(),
        SlackNotifierPlugin::new().info().clone(),
        DiscordNotifierPlugin::new().info().clone(),
    ]
}

pub fn create_builtin_exporter(name: &str) -> Option<Box<dyn ExporterPlugin>> {
    match name {
        "csv-exporter" => Some(Box::new(CsvExporterPlugin::new())),
        "json-exporter" => Some(Box::new(JsonExporterPlugin::new())),
        _ => None,
    }
}

pub fn create_builtin_notifier(name: &str) -> Option<Box<dyn NotifierPlugin>> {
    match name {
        "webhook-notifier" => Some(Box::new(WebhookNotifierPlugin::new())),
        "slack-notifier" => Some(Box::new(SlackNotifierPlugin::new())),
        "discord-notifier" => Some(Box::new(DiscordNotifierPlugin::new())),
        _ => None,
    }
}

pub fn is_builtin_plugin(name: &str) -> bool {
    matches!(
        name,
        "csv-exporter"
            | "json-exporter"
            | "webhook-notifier"
            | "slack-notifier"
            | "discord-notifier"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_builtin_plugins() {
        let plugins = list_builtin_plugins();
        assert_eq!(plugins.len(), 5);

        let names: Vec<&str> = plugins.iter().map(|p| p.name.as_str()).collect();
        assert!(names.contains(&"csv-exporter"));
        assert!(names.contains(&"json-exporter"));
        assert!(names.contains(&"webhook-notifier"));
        assert!(names.contains(&"slack-notifier"));
        assert!(names.contains(&"discord-notifier"));
    }

    #[test]
    fn test_create_builtin_exporter() {
        let csv = create_builtin_exporter("csv-exporter");
        assert!(csv.is_some());
        assert_eq!(csv.unwrap().format(), "csv");

        let json = create_builtin_exporter("json-exporter");
        assert!(json.is_some());
        assert_eq!(json.unwrap().format(), "json");

        let unknown = create_builtin_exporter("unknown");
        assert!(unknown.is_none());
    }

    #[test]
    fn test_create_builtin_notifier() {
        let webhook = create_builtin_notifier("webhook-notifier");
        assert!(webhook.is_some());
        assert_eq!(webhook.unwrap().notification_type(), "webhook");

        let slack = create_builtin_notifier("slack-notifier");
        assert!(slack.is_some());
        assert_eq!(slack.unwrap().notification_type(), "slack");

        let discord = create_builtin_notifier("discord-notifier");
        assert!(discord.is_some());
        assert_eq!(discord.unwrap().notification_type(), "discord");

        let unknown = create_builtin_notifier("unknown");
        assert!(unknown.is_none());
    }

    #[test]
    fn test_is_builtin_plugin() {
        assert!(is_builtin_plugin("csv-exporter"));
        assert!(is_builtin_plugin("json-exporter"));
        assert!(is_builtin_plugin("webhook-notifier"));
        assert!(is_builtin_plugin("slack-notifier"));
        assert!(is_builtin_plugin("discord-notifier"));
        assert!(!is_builtin_plugin("custom-plugin"));
        assert!(!is_builtin_plugin(""));
    }
}
