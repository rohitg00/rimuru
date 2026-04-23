use serde_json::Value;

pub struct ApiClient {
    base: String,
    client: reqwest::Client,
}

impl ApiClient {
    pub fn new(base: &str) -> Self {
        Self {
            base: base.to_string(),
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(5))
                .build()
                .unwrap_or_default(),
        }
    }

    pub fn from_env() -> Self {
        let base = std::env::var("RIMURU_API")
            .unwrap_or_else(|_| "http://127.0.0.1:3111".to_string());
        Self::new(&base)
    }

    pub async fn get(&self, path: &str) -> Option<Value> {
        let url = format!("{}/api{}", self.base, path);
        self.client
            .get(&url)
            .send()
            .await
            .ok()?
            .json::<Value>()
            .await
            .ok()
    }
}
