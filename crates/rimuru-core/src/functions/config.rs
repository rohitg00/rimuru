use iii_sdk::III;
use serde_json::{json, Value};

use crate::state::StateKV;

pub fn register(iii: &III, kv: &StateKV) {
    register_get(iii, kv);
    register_set(iii, kv);
}

fn default_config() -> Value {
    json!({
        "poll_interval_secs": 30,
        "cost_tracking_enabled": true,
        "session_monitoring_enabled": true,
        "metrics_collection_enabled": true,
        "metrics_interval_secs": 60,
        "max_session_history_days": 90,
        "max_cost_history_days": 365,
        "max_metrics_entries": 1440,
        "api_port": 3100,
        "enable_hooks": true,
        "enable_plugins": true,
        "log_level": "info",
        "theme": "dark",
        "currency": "USD",
        "budget_monthly": 0.0,
        "budget_alert_threshold": 0.8,
        "auto_detect_agents": true,
        "auto_sync_models": true,
        "model_sync_interval_hours": 24
    })
}

fn register_get(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function("rimuru.config.get", move |input: Value| {
        let kv = kv.clone();
        async move {
            let key = input.get("key").and_then(|v| v.as_str());

            match key {
                Some(k) => {
                    let value: Option<Value> = kv
                        .get("config", k)
                        .await
                        .map_err(|e| iii_sdk::IIIError::Handler(e.to_string()))?;

                    let defaults = default_config();
                    let default_val = defaults.get(k);

                    match value {
                        Some(v) => Ok(json!({
                            "key": k,
                            "value": v,
                            "source": "user"
                        })),
                        None => match default_val {
                            Some(d) => Ok(json!({
                                "key": k,
                                "value": d,
                                "source": "default"
                            })),
                            None => Err(iii_sdk::IIIError::Handler(format!(
                                "unknown config key: {}",
                                k
                            ))),
                        },
                    }
                }
                None => {
                    let defaults = default_config();
                    let default_map = defaults.as_object().cloned().unwrap_or_default();

                    let mut merged = serde_json::Map::new();
                    let mut sources = serde_json::Map::new();

                    for (k, default_val) in &default_map {
                        let stored: Option<Value> = kv
                            .get("config", k)
                            .await
                            .map_err(|e| iii_sdk::IIIError::Handler(e.to_string()))?;

                        match stored {
                            Some(v) => {
                                merged.insert(k.clone(), v);
                                sources.insert(k.clone(), json!("user"));
                            }
                            None => {
                                merged.insert(k.clone(), default_val.clone());
                                sources.insert(k.clone(), json!("default"));
                            }
                        }
                    }

                    let custom_keys = kv
                        .list_keys("config")
                        .await
                        .map_err(|e| iii_sdk::IIIError::Handler(e.to_string()))?;

                    for k in custom_keys {
                        if k.starts_with("search::") || k == "__health_probe" {
                            continue;
                        }
                        if !merged.contains_key(&k) {
                            let val: Option<Value> = kv
                                .get("config", &k)
                                .await
                                .map_err(|e| iii_sdk::IIIError::Handler(e.to_string()))?;
                            if let Some(v) = val {
                                merged.insert(k.clone(), v);
                                sources.insert(k, json!("user"));
                            }
                        }
                    }

                    Ok(json!({
                        "config": merged,
                        "sources": sources
                    }))
                }
            }
        }
    });
}

fn register_set(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function("rimuru.config.set", move |input: Value| {
        let kv = kv.clone();
        async move {
            let key = input
                .get("key")
                .and_then(|v| v.as_str())
                .ok_or_else(|| iii_sdk::IIIError::Handler("key is required".into()))?
                .to_string();

            let value = input
                .get("value")
                .ok_or_else(|| iii_sdk::IIIError::Handler("value is required".into()))?
                .clone();

            let defaults = default_config();
            if let Some(default_val) = defaults.get(&key) {
                let type_matches = match (default_val, &value) {
                    (Value::Bool(_), Value::Bool(_)) => true,
                    (Value::Number(_), Value::Number(_)) => true,
                    (Value::String(_), Value::String(_)) => true,
                    _ => false,
                };

                if !type_matches {
                    return Err(iii_sdk::IIIError::Handler(format!(
                        "type mismatch for {}: expected {}, got {}",
                        key,
                        value_type_name(default_val),
                        value_type_name(&value)
                    )));
                }
            }

            let old_value: Option<Value> = kv
                .get("config", &key)
                .await
                .map_err(|e| iii_sdk::IIIError::Handler(e.to_string()))?;

            kv.set("config", &key, &value)
                .await
                .map_err(|e| iii_sdk::IIIError::Handler(e.to_string()))?;

            Ok(json!({
                "key": key,
                "value": value,
                "old_value": old_value,
                "updated": true
            }))
        }
    });
}

fn value_type_name(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}
