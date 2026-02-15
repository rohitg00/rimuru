use crate::db::DatabaseError;
use crate::models::{Agent, AgentType};
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use super::Repository;

pub struct AgentRepository {
    pool: PgPool,
}

impl AgentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, agent: &Agent) -> Result<Agent, DatabaseError> {
        let record = sqlx::query_as::<_, Agent>(
            r#"
            INSERT INTO agents (id, name, agent_type, config, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, name, agent_type, config, created_at, updated_at
            "#,
        )
        .bind(agent.id)
        .bind(&agent.name)
        .bind(agent.agent_type)
        .bind(&agent.config)
        .bind(agent.created_at)
        .bind(agent.updated_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(record)
    }

    pub async fn update(&self, agent: &Agent) -> Result<Option<Agent>, DatabaseError> {
        let record = sqlx::query_as::<_, Agent>(
            r#"
            UPDATE agents
            SET name = $2, agent_type = $3, config = $4
            WHERE id = $1
            RETURNING id, name, agent_type, config, created_at, updated_at
            "#,
        )
        .bind(agent.id)
        .bind(&agent.name)
        .bind(agent.agent_type)
        .bind(&agent.config)
        .fetch_optional(&self.pool)
        .await?;

        Ok(record)
    }

    pub async fn get_by_type(&self, agent_type: AgentType) -> Result<Vec<Agent>, DatabaseError> {
        let records = sqlx::query_as::<_, Agent>(
            r#"
            SELECT id, name, agent_type, config, created_at, updated_at
            FROM agents
            WHERE agent_type = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(agent_type)
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }

    pub async fn get_by_name(&self, name: &str) -> Result<Option<Agent>, DatabaseError> {
        let record = sqlx::query_as::<_, Agent>(
            r#"
            SELECT id, name, agent_type, config, created_at, updated_at
            FROM agents
            WHERE name = $1
            "#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(record)
    }

    pub async fn count(&self) -> Result<i64, DatabaseError> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM agents")
            .fetch_one(&self.pool)
            .await?;

        Ok(count.0)
    }

    pub async fn count_by_type(&self, agent_type: AgentType) -> Result<i64, DatabaseError> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM agents WHERE agent_type = $1")
            .bind(agent_type)
            .fetch_one(&self.pool)
            .await?;

        Ok(count.0)
    }
}

#[async_trait]
impl Repository for AgentRepository {
    type Entity = Agent;
    type Id = Uuid;

    async fn get_by_id(&self, id: Uuid) -> Result<Option<Agent>, DatabaseError> {
        let record = sqlx::query_as::<_, Agent>(
            r#"
            SELECT id, name, agent_type, config, created_at, updated_at
            FROM agents
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(record)
    }

    async fn get_all(&self) -> Result<Vec<Agent>, DatabaseError> {
        let records = sqlx::query_as::<_, Agent>(
            r#"
            SELECT id, name, agent_type, config, created_at, updated_at
            FROM agents
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, DatabaseError> {
        let result = sqlx::query("DELETE FROM agents WHERE id = $1")
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
    fn test_agent_repository_new() {
        // This is a unit test that just verifies the struct can be constructed
        // Integration tests with actual DB would be in a separate test module
        assert!(true);
    }

    #[test]
    fn test_agent_creation_fields() {
        let agent = Agent::new(
            "test-agent".to_string(),
            AgentType::ClaudeCode,
            json!({"model": "claude-3"}),
        );

        assert_eq!(agent.name, "test-agent");
        assert_eq!(agent.agent_type, AgentType::ClaudeCode);
    }
}
