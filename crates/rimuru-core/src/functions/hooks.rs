use iii_sdk::III;
use serde_json::{json, Value};

use crate::models::HookRegistration;
use crate::state::StateKV;

pub fn register(iii: &III, kv: &StateKV) {
    register_dispatch(iii, kv);
    register_register(iii, kv);
    register_list(iii, kv);
}

fn register_dispatch(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function("rimuru.hooks.dispatch", move |input: Value| {
        let kv = kv.clone();
        async move {
            let event_type = input
                .get("event_type")
                .and_then(|v| v.as_str())
                .ok_or_else(|| iii_sdk::IIIError::Handler("event_type is required".into()))?
                .to_string();

            let payload = input.get("payload").cloned().unwrap_or(json!({}));

            let hooks: Vec<HookRegistration> = kv
                .list("hooks")
                .await
                .map_err(|e| iii_sdk::IIIError::Handler(e.to_string()))?;

            let mut matching: Vec<&HookRegistration> = hooks
                .iter()
                .filter(|h| h.event_type == event_type)
                .collect();

            matching.sort_by(|a, b| b.priority.cmp(&a.priority));

            let mut results: Vec<Value> = Vec::new();
            let mut errors: Vec<Value> = Vec::new();

            let iii_ref = kv.iii().clone();

            for hook in &matching {
                let invoke_result: Result<Value, iii_sdk::IIIError> = iii_ref
                    .trigger(&hook.function_id, payload.clone())
                    .await;

                match invoke_result {
                    Ok(result) => {
                        results.push(json!({
                            "function_id": hook.function_id,
                            "priority": hook.priority,
                            "result": result
                        }));
                    }
                    Err(e) => {
                        errors.push(json!({
                            "function_id": hook.function_id,
                            "priority": hook.priority,
                            "error": e.to_string()
                        }));
                    }
                }
            }

            Ok(json!({
                "event_type": event_type,
                "hooks_matched": matching.len(),
                "results": results,
                "errors": errors,
                "total_succeeded": results.len(),
                "total_failed": errors.len()
            }))
        }
    });
}

fn register_register(iii: &III, kv: &StateKV) {
    let kv = kv.clone();
    iii.register_function("rimuru.hooks.register", move |input: Value| {
        let kv = kv.clone();
        async move {
            let event_type = input
                .get("event_type")
                .and_then(|v| v.as_str())
                .ok_or_else(|| iii_sdk::IIIError::Handler("event_type is required".into()))?
                .to_string();

            let function_id = input
                .get("function_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| iii_sdk::IIIError::Handler("function_id is required".into()))?
                .to_string();

            let priority = input
                .get("priority")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32;

            let hook = HookRegistration {
                event_type: event_type.clone(),
                function_id: function_id.clone(),
                priority,
            };

            let hook_key = format!("{}::{}", event_type, function_id);

            let existing: Vec<HookRegistration> = kv
                .list("hooks")
                .await
                .map_err(|e| iii_sdk::IIIError::Handler(e.to_string()))?;

            let already_exists = existing
                .iter()
                .any(|h| h.event_type == event_type && h.function_id == function_id);

            if already_exists {
                kv.set("hooks", &hook_key, &hook)
                    .await
                    .map_err(|e| iii_sdk::IIIError::Handler(e.to_string()))?;

                Ok(json!({
                    "hook": hook,
                    "updated": true
                }))
            } else {
                kv.set("hooks", &hook_key, &hook)
                    .await
                    .map_err(|e| iii_sdk::IIIError::Handler(e.to_string()))?;

                Ok(json!({
                    "hook": hook,
                    "registered": true
                }))
            }
        }
    });
}

fn register_list(iii: &III, _kv: &StateKV) {
    iii.register_function("rimuru.hooks.list", move |_input: Value| {
        async move {
            let hooks = crate::discovery::discover_hooks().await;

            let mut event_types: Vec<String> = hooks
                .iter()
                .filter_map(|h| h.get("event_type").and_then(|e| e.as_str()).map(String::from))
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();
            event_types.sort();

            Ok(json!({
                "hooks": hooks,
                "total": hooks.len(),
                "event_types": event_types
            }))
        }
    });
}
