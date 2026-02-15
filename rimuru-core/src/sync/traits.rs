use async_trait::async_trait;

use crate::error::RimuruResult;
use crate::models::ModelInfo;

use super::types::SyncResult;

#[async_trait]
pub trait ModelSyncProvider: Send + Sync {
    fn provider_name(&self) -> &str;

    fn supports_streaming(&self) -> bool {
        false
    }

    async fn fetch_models(&self) -> RimuruResult<Vec<ModelInfo>>;

    fn is_official_source(&self) -> bool {
        false
    }

    fn priority(&self) -> u8 {
        100
    }

    async fn health_check(&self) -> RimuruResult<bool> {
        Ok(true)
    }
}

#[async_trait]
pub trait SyncScheduler: Send + Sync {
    async fn start(&self) -> RimuruResult<()>;

    async fn stop(&self) -> RimuruResult<()>;

    async fn trigger_sync(&self) -> RimuruResult<SyncResult>;

    async fn trigger_provider_sync(&self, provider: &str) -> RimuruResult<SyncResult>;

    fn is_running(&self) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockSyncProvider {
        name: String,
        models: Vec<ModelInfo>,
    }

    impl MockSyncProvider {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                models: vec![ModelInfo::new(
                    name.to_string(),
                    "test-model".to_string(),
                    0.01,
                    0.03,
                    128000,
                )],
            }
        }
    }

    #[async_trait]
    impl ModelSyncProvider for MockSyncProvider {
        fn provider_name(&self) -> &str {
            &self.name
        }

        async fn fetch_models(&self) -> RimuruResult<Vec<ModelInfo>> {
            Ok(self.models.clone())
        }

        fn is_official_source(&self) -> bool {
            true
        }

        fn priority(&self) -> u8 {
            10
        }
    }

    #[tokio::test]
    async fn test_mock_provider() {
        let provider = MockSyncProvider::new("test-provider");

        assert_eq!(provider.provider_name(), "test-provider");
        assert!(provider.is_official_source());
        assert_eq!(provider.priority(), 10);
        assert!(!provider.supports_streaming());

        let models = provider.fetch_models().await.unwrap();
        assert_eq!(models.len(), 1);
        assert_eq!(models[0].provider, "test-provider");
        assert_eq!(models[0].model_name, "test-model");
    }

    #[tokio::test]
    async fn test_default_health_check() {
        let provider = MockSyncProvider::new("test");
        assert!(provider.health_check().await.unwrap());
    }
}
