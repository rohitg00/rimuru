use crate::db::DatabaseError;
use crate::models::{Session, SessionStatus};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use super::Repository;

pub struct SessionRepository {
    pool: PgPool,
}

impl SessionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, session: &Session) -> Result<Session, DatabaseError> {
        let record = sqlx::query_as::<_, Session>(
            r#"
            INSERT INTO sessions (id, agent_id, status, started_at, ended_at, metadata)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, agent_id, status, started_at, ended_at, metadata
            "#,
        )
        .bind(session.id)
        .bind(session.agent_id)
        .bind(session.status)
        .bind(session.started_at)
        .bind(session.ended_at)
        .bind(&session.metadata)
        .fetch_one(&self.pool)
        .await?;

        Ok(record)
    }

    pub async fn get_active(&self) -> Result<Vec<Session>, DatabaseError> {
        let records = sqlx::query_as::<_, Session>(
            r#"
            SELECT id, agent_id, status, started_at, ended_at, metadata
            FROM sessions
            WHERE status = 'active'
            ORDER BY started_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }

    pub async fn get_active_count(&self) -> Result<i64, DatabaseError> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM sessions WHERE status = 'active'")
            .fetch_one(&self.pool)
            .await?;

        Ok(count.0)
    }

    pub async fn get_by_agent(&self, agent_id: Uuid) -> Result<Vec<Session>, DatabaseError> {
        let records = sqlx::query_as::<_, Session>(
            r#"
            SELECT id, agent_id, status, started_at, ended_at, metadata
            FROM sessions
            WHERE agent_id = $1
            ORDER BY started_at DESC
            "#,
        )
        .bind(agent_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }

    pub async fn get_active_by_agent(&self, agent_id: Uuid) -> Result<Vec<Session>, DatabaseError> {
        let records = sqlx::query_as::<_, Session>(
            r#"
            SELECT id, agent_id, status, started_at, ended_at, metadata
            FROM sessions
            WHERE agent_id = $1 AND status = 'active'
            ORDER BY started_at DESC
            "#,
        )
        .bind(agent_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }

    pub async fn update_status(
        &self,
        id: Uuid,
        status: SessionStatus,
    ) -> Result<Option<Session>, DatabaseError> {
        let record = sqlx::query_as::<_, Session>(
            r#"
            UPDATE sessions
            SET status = $2
            WHERE id = $1
            RETURNING id, agent_id, status, started_at, ended_at, metadata
            "#,
        )
        .bind(id)
        .bind(status)
        .fetch_optional(&self.pool)
        .await?;

        Ok(record)
    }

    pub async fn end_session(
        &self,
        id: Uuid,
        status: SessionStatus,
    ) -> Result<Option<Session>, DatabaseError> {
        let record = sqlx::query_as::<_, Session>(
            r#"
            UPDATE sessions
            SET status = $2, ended_at = $3
            WHERE id = $1
            RETURNING id, agent_id, status, started_at, ended_at, metadata
            "#,
        )
        .bind(id)
        .bind(status)
        .bind(Utc::now())
        .fetch_optional(&self.pool)
        .await?;

        Ok(record)
    }

    pub async fn get_by_date_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Session>, DatabaseError> {
        let records = sqlx::query_as::<_, Session>(
            r#"
            SELECT id, agent_id, status, started_at, ended_at, metadata
            FROM sessions
            WHERE started_at >= $1 AND started_at <= $2
            ORDER BY started_at DESC
            "#,
        )
        .bind(start)
        .bind(end)
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }

    pub async fn count(&self) -> Result<i64, DatabaseError> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM sessions")
            .fetch_one(&self.pool)
            .await?;

        Ok(count.0)
    }

    pub async fn count_by_status(&self, status: SessionStatus) -> Result<i64, DatabaseError> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM sessions WHERE status = $1")
            .bind(status)
            .fetch_one(&self.pool)
            .await?;

        Ok(count.0)
    }
}

#[async_trait]
impl Repository for SessionRepository {
    type Entity = Session;
    type Id = Uuid;

    async fn get_by_id(&self, id: Uuid) -> Result<Option<Session>, DatabaseError> {
        let record = sqlx::query_as::<_, Session>(
            r#"
            SELECT id, agent_id, status, started_at, ended_at, metadata
            FROM sessions
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(record)
    }

    async fn get_all(&self) -> Result<Vec<Session>, DatabaseError> {
        let records = sqlx::query_as::<_, Session>(
            r#"
            SELECT id, agent_id, status, started_at, ended_at, metadata
            FROM sessions
            ORDER BY started_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, DatabaseError> {
        let result = sqlx::query("DELETE FROM sessions WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_session_creation() {
        let agent_id = Uuid::new_v4();
        let session = Session::new(agent_id, json!({"project": "test"}));

        assert_eq!(session.agent_id, agent_id);
        assert_eq!(session.status, SessionStatus::Active);
        assert!(session.is_active());
    }

    #[test]
    fn test_session_end() {
        let agent_id = Uuid::new_v4();
        let mut session = Session::new(agent_id, json!({}));

        session.end(SessionStatus::Completed);

        assert!(!session.is_active());
        assert_eq!(session.status, SessionStatus::Completed);
        assert!(session.ended_at.is_some());
    }
}
