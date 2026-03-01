use anyhow::Result;
use iii_sdk::III;
use serde_json::{json, Value};

use crate::output::{self, OutputFormat};

pub async fn get(iii: &III, key: Option<&str>, format: &OutputFormat) -> Result<()> {
    let input = if let Some(k) = key {
        json!({"key": k})
    } else {
        json!({})
    };
    let result = iii.trigger("rimuru.config.get", input).await?;

    if let Some(k) = key {
        if let Some(val) = result.get("value") {
            let source = result
                .get("source")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            println!("{k} = {} ({})", val, source);
        } else {
            anyhow::bail!("Config key '{k}' not found");
        }
    } else {
        let config_obj = result.get("config").unwrap_or(&result);
        println!("{}", output::format_config(config_obj, format));
    }
    Ok(())
}

pub async fn set(iii: &III, key: &str, value: &str, format: &OutputFormat) -> Result<()> {
    let typed_value = if value == "true" {
        Value::Bool(true)
    } else if value == "false" {
        Value::Bool(false)
    } else if let Ok(n) = value.parse::<i64>() {
        Value::Number(n.into())
    } else if let Ok(n) = value.parse::<f64>() {
        serde_json::Number::from_f64(n)
            .map(Value::Number)
            .unwrap_or_else(|| Value::String(value.to_string()))
    } else {
        Value::String(value.to_string())
    };

    let result = iii
        .trigger(
            "rimuru.config.set",
            json!({
                "key": key,
                "value": typed_value,
            }),
        )
        .await?;

    println!("Set {key} = {value}");
    output::print_value(&result, format);
    Ok(())
}
