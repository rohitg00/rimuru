use crate::db::DatabaseError;
use crate::models::{CostRecord, CostSummary};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use super::Repository;

pub struct CostRepository {
    pool: PgPool,
}

impl CostRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn record_cost(&self, cost: &CostRecord) -> Result<CostRecord, DatabaseError> {
        let record = sqlx::query_as::<_, CostRecord>(
            r#"
            INSERT INTO cost_records (id, session_id, agent_id, model_name, input_tokens, output_tokens, cost_usd, recorded_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, session_id, agent_id, model_name, input_tokens, output_tokens, cost_usd, recorded_at
            "#,
        )
        .bind(cost.id)
        .bind(cost.session_id)
        .bind(cost.agent_id)
        .bind(&cost.model_name)
        .bind(cost.input_tokens)
        .bind(cost.output_tokens)
        .bind(cost.cost_usd)
        .bind(cost.recorded_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(record)
    }

    pub async fn get_by_session(&self, session_id: Uuid) -> Result<Vec<CostRecord>, DatabaseError> {
        let records = sqlx::query_as::<_, CostRecord>(
            r#"
            SELECT id, session_id, agent_id, model_name, input_tokens, output_tokens, cost_usd, recorded_at
            FROM cost_records
            WHERE session_id = $1
            ORDER BY recorded_at DESC
            "#,
        )
        .bind(session_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }

    pub async fn get_by_agent(&self, agent_id: Uuid) -> Result<Vec<CostRecord>, DatabaseError> {
        let records = sqlx::query_as::<_, CostRecord>(
            r#"
            SELECT id, session_id, agent_id, model_name, input_tokens, output_tokens, cost_usd, recorded_at
            FROM cost_records
            WHERE agent_id = $1
            ORDER BY recorded_at DESC
            "#,
        )
        .bind(agent_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }

    pub async fn get_by_model(&self, model_name: &str) -> Result<Vec<CostRecord>, DatabaseError> {
        let records = sqlx::query_as::<_, CostRecord>(
            r#"
            SELECT id, session_id, agent_id, model_name, input_tokens, output_tokens, cost_usd, recorded_at
            FROM cost_records
            WHERE model_name = $1
            ORDER BY recorded_at DESC
            "#,
        )
        .bind(model_name)
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }

    pub async fn get_total_by_date_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<CostSummary, DatabaseError> {
        let summary = sqlx::query_as::<_, (f64, i64, i64, i64)>(
            r#"
            SELECT
                COALESCE(SUM(cost_usd), 0.0) as total_cost_usd,
                COALESCE(SUM(input_tokens), 0) as total_input_tokens,
                COALESCE(SUM(output_tokens), 0) as total_output_tokens,
                COUNT(*) as record_count
            FROM cost_records
            WHERE recorded_at >= $1 AND recorded_at <= $2
            "#,
        )
        .bind(start)
        .bind(end)
        .fetch_one(&self.pool)
        .await?;

        Ok(CostSummary {
            total_cost_usd: summary.0,
            total_input_tokens: summary.1,
            total_output_tokens: summary.2,
            record_count: summary.3,
        })
    }

    pub async fn get_total_by_agent(&self, agent_id: Uuid) -> Result<CostSummary, DatabaseError> {
        let summary = sqlx::query_as::<_, (f64, i64, i64, i64)>(
            r#"
            SELECT
                COALESCE(SUM(cost_usd), 0.0) as total_cost_usd,
                COALESCE(SUM(input_tokens), 0) as total_input_tokens,
                COALESCE(SUM(output_tokens), 0) as total_output_tokens,
                COUNT(*) as record_count
            FROM cost_records
            WHERE agent_id = $1
            "#,
        )
        .bind(agent_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(CostSummary {
            total_cost_usd: summary.0,
            total_input_tokens: summary.1,
            total_output_tokens: summary.2,
            record_count: summary.3,
        })
    }

    pub async fn get_total_by_session(
        &self,
        session_id: Uuid,
    ) -> Result<CostSummary, DatabaseError> {
        let summary = sqlx::query_as::<_, (f64, i64, i64, i64)>(
            r#"
            SELECT
                COALESCE(SUM(cost_usd), 0.0) as total_cost_usd,
                COALESCE(SUM(input_tokens), 0) as total_input_tokens,
                COALESCE(SUM(output_tokens), 0) as total_output_tokens,
                COUNT(*) as record_count
            FROM cost_records
            WHERE session_id = $1
            "#,
        )
        .bind(session_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(CostSummary {
            total_cost_usd: summary.0,
            total_input_tokens: summary.1,
            total_output_tokens: summary.2,
            record_count: summary.3,
        })
    }

    pub async fn get_total_by_agent_and_date_range(
        &self,
        agent_id: Uuid,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<CostSummary, DatabaseError> {
        let summary = sqlx::query_as::<_, (f64, i64, i64, i64)>(
            r#"
            SELECT
                COALESCE(SUM(cost_usd), 0.0) as total_cost_usd,
                COALESCE(SUM(input_tokens), 0) as total_input_tokens,
                COALESCE(SUM(output_tokens), 0) as total_output_tokens,
                COUNT(*) as record_count
            FROM cost_records
            WHERE agent_id = $1 AND recorded_at >= $2 AND recorded_at <= $3
            "#,
        )
        .bind(agent_id)
        .bind(start)
        .bind(end)
        .fetch_one(&self.pool)
        .await?;

        Ok(CostSummary {
            total_cost_usd: summary.0,
            total_input_tokens: summary.1,
            total_output_tokens: summary.2,
            record_count: summary.3,
        })
    }

    pub async fn get_daily_costs(
        &self,
        days: i32,
    ) -> Result<Vec<(DateTime<Utc>, f64)>, DatabaseError> {
        let records = sqlx::query_as::<_, (DateTime<Utc>, f64)>(
            r#"
            SELECT
                DATE_TRUNC('day', recorded_at) as day,
                COALESCE(SUM(cost_usd), 0.0) as total_cost
            FROM cost_records
            WHERE recorded_at >= NOW() - ($1 || ' days')::INTERVAL
            GROUP BY DATE_TRUNC('day', recorded_at)
            ORDER BY day DESC
            "#,
        )
        .bind(days)
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }
}

#[async_trait]
impl Repository for CostRepository {
    type Entity = CostRecord;
    type Id = Uuid;

    async fn get_by_id(&self, id: Uuid) -> Result<Option<CostRecord>, DatabaseError> {
        let record = sqlx::query_as::<_, CostRecord>(
            r#"
            SELECT id, session_id, agent_id, model_name, input_tokens, output_tokens, cost_usd, recorded_at
            FROM cost_records
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(record)
    }

    async fn get_all(&self) -> Result<Vec<CostRecord>, DatabaseError> {
        let records = sqlx::query_as::<_, CostRecord>(
            r#"
            SELECT id, session_id, agent_id, model_name, input_tokens, output_tokens, cost_usd, recorded_at
            FROM cost_records
            ORDER BY recorded_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, DatabaseError> {
        let result = sqlx::query("DELETE FROM cost_records WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cost_record_creation() {
        let session_id = Uuid::new_v4();
        let agent_id = Uuid::new_v4();
        let record = CostRecord::new(
            session_id,
            agent_id,
            "claude-opus-4-5".to_string(),
            1000,
            500,
            0.05,
        );

        assert_eq!(record.session_id, session_id);
        assert_eq!(record.agent_id, agent_id);
        assert_eq!(record.total_tokens(), 1500);
    }

    #[test]
    fn test_cost_summary_calculations() {
        let summary = CostSummary {
            total_cost_usd: 10.0,
            total_input_tokens: 50000,
            total_output_tokens: 25000,
            record_count: 100,
        };

        assert_eq!(summary.total_tokens(), 75000);
        assert_eq!(summary.average_cost_per_request(), 0.1);
    }
}
