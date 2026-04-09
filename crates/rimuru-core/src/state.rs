use iii_sdk::{III, TriggerRequest};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::json;
use tracing::warn;

use crate::error::RimuruError;

type Result<T> = std::result::Result<T, RimuruError>;

const DEFAULT_STATE_TIMEOUT_MS: u64 = 10_000;

#[derive(Clone)]
pub struct StateKV {
    iii: III,
}

impl StateKV {
    pub fn new(iii: III) -> Self {
        Self { iii }
    }

    pub async fn get<T: DeserializeOwned>(&self, scope: &str, key: &str) -> Result<Option<T>> {
        let result = self
            .iii
            .trigger(TriggerRequest {
                function_id: "state::get".to_string(),
                payload: json!({"scope": scope, "key": key}),
                action: None,
                timeout_ms: Some(DEFAULT_STATE_TIMEOUT_MS),
            })
            .await
            .map_err(|e| RimuruError::Bridge(e.to_string()))?;

        if result.is_null() {
            return Ok(None);
        }

        let val: T = serde_json::from_value(result)?;
        Ok(Some(val))
    }

    pub async fn set<T: Serialize>(&self, scope: &str, key: &str, data: &T) -> Result<()> {
        let value = serde_json::to_value(data)?;
        self.iii
            .trigger(TriggerRequest {
                function_id: "state::set".to_string(),
                payload: json!({"scope": scope, "key": key, "value": value}),
                action: None,
                timeout_ms: Some(DEFAULT_STATE_TIMEOUT_MS),
            })
            .await
            .map_err(|e| RimuruError::Bridge(e.to_string()))?;
        Ok(())
    }

    pub async fn delete(&self, scope: &str, key: &str) -> Result<()> {
        self.iii
            .trigger(TriggerRequest {
                function_id: "state::delete".to_string(),
                payload: json!({"scope": scope, "key": key}),
                action: None,
                timeout_ms: Some(DEFAULT_STATE_TIMEOUT_MS),
            })
            .await
            .map_err(|e| RimuruError::Bridge(e.to_string()))?;
        Ok(())
    }

    pub async fn list_keys(&self, scope: &str) -> Result<Vec<String>> {
        let result = self
            .iii
            .trigger(TriggerRequest {
                function_id: "state::list".to_string(),
                payload: json!({"scope": scope}),
                action: None,
                timeout_ms: Some(DEFAULT_STATE_TIMEOUT_MS),
            })
            .await
            .map_err(|e| RimuruError::Bridge(e.to_string()))?;

        if let Some(arr) = result.as_array() {
            let keys: Vec<String> = arr.iter().enumerate().map(|(i, _)| i.to_string()).collect();
            Ok(keys)
        } else {
            warn!(
                "state::list for scope '{}' returned unexpected format: {}",
                scope, result
            );
            Err(RimuruError::Bridge(format!(
                "state::list returned non-array for scope '{scope}'"
            )))
        }
    }

    pub async fn list<T: DeserializeOwned>(&self, scope: &str) -> Result<Vec<T>> {
        let result = self
            .iii
            .trigger(TriggerRequest {
                function_id: "state::list".to_string(),
                payload: json!({"scope": scope}),
                action: None,
                timeout_ms: Some(DEFAULT_STATE_TIMEOUT_MS),
            })
            .await
            .map_err(|e| RimuruError::Bridge(e.to_string()))?;

        if let Some(arr) = result.as_array() {
            let items: Vec<T> = arr
                .iter()
                .filter_map(|entry| serde_json::from_value::<T>(entry.clone()).ok())
                .collect();
            Ok(items)
        } else {
            warn!(
                "state::list for scope '{}' returned unexpected format: {}",
                scope, result
            );
            Err(RimuruError::Bridge(format!(
                "state::list returned non-array for scope '{scope}'"
            )))
        }
    }

    pub async fn update_field<T: Serialize>(
        &self,
        scope: &str,
        key: &str,
        field: &str,
        value: &T,
    ) -> Result<()> {
        let json_val = serde_json::to_value(value)?;
        self.iii
            .trigger(TriggerRequest {
                function_id: "state::update".to_string(),
                payload: json!({
                    "scope": scope,
                    "key": key,
                    "ops": [{"type": "set", "path": field, "value": json_val}]
                }),
                action: None,
                timeout_ms: Some(DEFAULT_STATE_TIMEOUT_MS),
            })
            .await
            .map_err(|e| RimuruError::Bridge(e.to_string()))?;
        Ok(())
    }

    pub async fn increment(&self, scope: &str, key: &str, field: &str, by: i64) -> Result<i64> {
        let result = self
            .iii
            .trigger(TriggerRequest {
                function_id: "state::update".to_string(),
                payload: json!({
                    "scope": scope,
                    "key": key,
                    "ops": [{"type": "increment", "path": field, "by": by}]
                }),
                action: None,
                timeout_ms: Some(DEFAULT_STATE_TIMEOUT_MS),
            })
            .await
            .map_err(|e| RimuruError::Bridge(e.to_string()))?;

        match result
            .get("new_value")
            .and_then(|v| v.get(field))
            .and_then(|v| v.as_i64())
        {
            Some(val) => Ok(val),
            None => {
                warn!(
                    "state::update increment for {}/{}.'{}' (by={}) returned unexpected format: {}",
                    scope, key, field, by, result
                );
                Err(RimuruError::Bridge(format!(
                    "state::update increment returned unexpected result for {scope}/{key}.'{field}'"
                )))
            }
        }
    }

    pub fn iii(&self) -> &III {
        &self.iii
    }
}
