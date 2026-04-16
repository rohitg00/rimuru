use serde_json::Value;

use crate::state::StateKV;

use super::sysutil::kv_err;

pub async fn load_webhook_url(kv: &StateKV, key: &str) -> Option<String> {
    match kv.get::<Value>("config", key).await {
        Ok(Some(v)) => v.as_str().map(|s| s.to_string()).filter(|s| !s.is_empty()),
        Ok(None) => None,
        Err(e) => {
            tracing::warn!("failed to read webhook config {}: {}", key, e);
            None
        }
    }
}

pub async fn post_webhook(url: &str, payload: &Value) {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build();
    let client = match client {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("webhook client build failed: {}", e);
            return;
        }
    };

    match client.post(url).json(payload).send().await {
        Ok(resp) => {
            if !resp.status().is_success() {
                tracing::warn!(
                    "webhook {} returned non-success status: {}",
                    url,
                    resp.status()
                );
            }
        }
        Err(e) => {
            tracing::warn!("webhook post to {} failed: {}", url, e);
        }
    }
}

#[allow(dead_code)]
pub async fn dispatch_webhook(
    kv: &StateKV,
    config_key: &str,
    payload: &Value,
) -> Result<(), iii_sdk::IIIError> {
    if let Some(url) = load_webhook_url(kv, config_key).await {
        post_webhook(&url, payload).await;
    }
    Ok(())
}

#[allow(dead_code)]
pub fn mark_kv_err(e: impl std::fmt::Display) -> iii_sdk::IIIError {
    kv_err(e)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn posts_json_body() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/hook"))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&server)
            .await;

        let url = format!("{}/hook", server.uri());
        post_webhook(&url, &json!({"event": "budget.exceeded", "level": "high"})).await;
    }

    #[tokio::test]
    async fn tolerates_non_success_status() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/hook"))
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&server)
            .await;

        let url = format!("{}/hook", server.uri());
        post_webhook(&url, &json!({"event": "runaway_detected"})).await;
    }
}
