use std::sync::Arc;

use dashmap::DashMap;
use iii_sdk::III;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;

use crate::error::RimuruError;

type Result<T> = std::result::Result<T, RimuruError>;

#[derive(Clone)]
pub struct StateKV {
    iii: III,
    store: Arc<DashMap<String, Value>>,
}

impl StateKV {
    pub fn new(iii: III) -> Self {
        Self {
            iii,
            store: Arc::new(DashMap::new()),
        }
    }

    fn scope_key(scope: &str, key: &str) -> String {
        format!("{}::{}", scope, key)
    }

    pub async fn get<T: DeserializeOwned>(&self, scope: &str, key: &str) -> Result<Option<T>> {
        let full_key = Self::scope_key(scope, key);

        if let Some(entry) = self.store.get(&full_key) {
            let val: T = serde_json::from_value(entry.value().clone())?;
            return Ok(Some(val));
        }

        Ok(None)
    }

    pub async fn set<T: Serialize>(&self, scope: &str, key: &str, data: &T) -> Result<()> {
        let full_key = Self::scope_key(scope, key);
        let value = serde_json::to_value(data)?;
        self.store.insert(full_key, value);
        Ok(())
    }

    pub async fn delete(&self, scope: &str, key: &str) -> Result<()> {
        let full_key = Self::scope_key(scope, key);
        self.store.remove(&full_key);
        Ok(())
    }

    pub async fn list_keys(&self, scope: &str) -> Result<Vec<String>> {
        let prefix = format!("{}::", scope);
        let keys: Vec<String> = self
            .store
            .iter()
            .filter_map(|entry| {
                let k = entry.key();
                if k.starts_with(&prefix) {
                    Some(k.strip_prefix(&prefix).unwrap_or(k).to_string())
                } else {
                    None
                }
            })
            .collect();
        Ok(keys)
    }

    pub async fn list<T: DeserializeOwned>(&self, scope: &str) -> Result<Vec<T>> {
        let prefix = format!("{}::", scope);
        let mut items = Vec::new();

        for entry in self.store.iter() {
            if entry.key().starts_with(&prefix) {
                let val = entry.value().clone();
                if let Ok(item) = serde_json::from_value::<T>(val) {
                    items.push(item);
                }
            }
        }

        Ok(items)
    }

    pub async fn update_field<T: Serialize>(
        &self,
        scope: &str,
        key: &str,
        field: &str,
        value: &T,
    ) -> Result<()> {
        let full_key = Self::scope_key(scope, key);
        let json_val = serde_json::to_value(value)?;

        self.store
            .entry(full_key)
            .and_modify(|existing| {
                if let Value::Object(ref mut map) = existing {
                    map.insert(field.to_string(), json_val.clone());
                }
            })
            .or_insert_with(|| {
                let mut map = serde_json::Map::new();
                map.insert(field.to_string(), json_val);
                Value::Object(map)
            });

        Ok(())
    }

    pub async fn increment(&self, scope: &str, key: &str, field: &str, by: i64) -> Result<i64> {
        let full_key = Self::scope_key(scope, key);

        let new_val = {
            let mut entry = self
                .store
                .entry(full_key)
                .or_insert_with(|| Value::Object(serde_json::Map::new()));

            if let Value::Object(ref mut map) = entry.value_mut() {
                let current = map
                    .get(field)
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                let updated = current + by;
                map.insert(field.to_string(), Value::Number(updated.into()));
                updated
            } else {
                by
            }
        };

        Ok(new_val)
    }

    pub fn iii(&self) -> &III {
        &self.iii
    }
}
