use anyhow::Result;
use iii_sdk::III;
use serde_json::json;

use crate::output::{self, OutputFormat};

pub async fn list(iii: &III, format: &OutputFormat) -> Result<()> {
    let result = iii.trigger("rimuru.plugins.list", json!({})).await?;
    let plugins = if let Some(arr) = result.get("plugins").and_then(|v| v.as_array()) {
        arr.clone()
    } else {
        result.as_array().cloned().unwrap_or_default()
    };
    println!("{}", output::format_plugins_list(&plugins, format));
    Ok(())
}

pub async fn install(iii: &III, plugin_path: &str, format: &OutputFormat) -> Result<()> {
    println!("Installing plugin from {plugin_path}...");
    let result = iii
        .trigger("rimuru.plugins.install", json!({"path": plugin_path}))
        .await?;

    let success = result
        .get("installed")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if success {
        let name = result
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        println!("Plugin '{name}' installed successfully.");
    }

    output::print_value(&result, format);
    Ok(())
}

pub async fn uninstall(iii: &III, plugin_id: &str, format: &OutputFormat) -> Result<()> {
    let result = iii
        .trigger(
            "rimuru.plugins.uninstall",
            json!({"plugin_id": plugin_id}),
        )
        .await?;

    let success = result
        .get("uninstalled")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if success {
        println!("Plugin '{plugin_id}' uninstalled.");
    } else {
        let msg = result
            .get("message")
            .and_then(|v| v.as_str())
            .unwrap_or("Failed to uninstall plugin");
        anyhow::bail!("{msg}");
    }

    output::print_value(&result, format);
    Ok(())
}
