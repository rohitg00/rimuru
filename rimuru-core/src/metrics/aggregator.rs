use crate::db::DatabaseError;
use crate::repo::SessionRepository;
use sqlx::PgPool;
use tracing::{debug, trace};

pub struct SessionAggregator {
    pool: PgPool,
}

impl SessionAggregator {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_active_sessions_count(&self) -> Result<i32, DatabaseError> {
        let session_repo = SessionRepository::new(self.pool.clone());
        let count = session_repo.get_active_count().await?;
        trace!(active_sessions = count, "Fetched active sessions count");
        Ok(count as i32)
    }

    pub async fn get_active_sessions_count_or_default(&self) -> i32 {
        match self.get_active_sessions_count().await {
            Ok(count) => count,
            Err(e) => {
                debug!(error = %e, "Failed to fetch active sessions count, defaulting to 0");
                0
            }
        }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

impl Clone for SessionAggregator {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
        }
    }
}

#[cfg(test)]
mod tests {}
