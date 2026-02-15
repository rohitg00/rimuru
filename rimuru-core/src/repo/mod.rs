pub mod agent_repo;
pub mod cost_repo;
pub mod metrics_repo;
pub mod model_repo;
pub mod session_repo;

pub use agent_repo::AgentRepository;
pub use cost_repo::CostRepository;
pub use metrics_repo::MetricsRepository;
pub use model_repo::ModelRepository;
pub use session_repo::SessionRepository;

use crate::db::DatabaseError;
use async_trait::async_trait;

#[async_trait]
pub trait Repository {
    type Entity;
    type Id;

    async fn get_by_id(&self, id: Self::Id) -> Result<Option<Self::Entity>, DatabaseError>;
    async fn get_all(&self) -> Result<Vec<Self::Entity>, DatabaseError>;
    async fn delete(&self, id: Self::Id) -> Result<bool, DatabaseError>;
}
