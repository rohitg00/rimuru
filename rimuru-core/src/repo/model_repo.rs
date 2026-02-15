use crate::db::DatabaseError;
use crate::models::ModelInfo;
use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use super::Repository;

pub struct ModelRepository {
    pool: PgPool,
}

impl ModelRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn upsert(&self, model: &ModelInfo) -> Result<ModelInfo, DatabaseError> {
        let record = sqlx::query_as::<_, ModelInfo>(
            r#"
            INSERT INTO model_info (id, provider, model_name, input_price_per_1k, output_price_per_1k, context_window, last_synced)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (provider, model_name) DO UPDATE SET
                input_price_per_1k = EXCLUDED.input_price_per_1k,
                output_price_per_1k = EXCLUDED.output_price_per_1k,
                context_window = EXCLUDED.context_window,
                last_synced = NOW()
            RETURNING id, provider, model_name, input_price_per_1k, output_price_per_1k, context_window, last_synced
            "#,
        )
        .bind(model.id)
        .bind(&model.provider)
        .bind(&model.model_name)
        .bind(model.input_price_per_1k)
        .bind(model.output_price_per_1k)
        .bind(model.context_window)
        .bind(model.last_synced)
        .fetch_one(&self.pool)
        .await?;

        Ok(record)
    }

    pub async fn get_by_provider(&self, provider: &str) -> Result<Vec<ModelInfo>, DatabaseError> {
        let records = sqlx::query_as::<_, ModelInfo>(
            r#"
            SELECT id, provider, model_name, input_price_per_1k, output_price_per_1k, context_window, last_synced
            FROM model_info
            WHERE provider = $1
            ORDER BY model_name ASC
            "#,
        )
        .bind(provider)
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }

    pub async fn get_by_name(
        &self,
        provider: &str,
        model_name: &str,
    ) -> Result<Option<ModelInfo>, DatabaseError> {
        let record = sqlx::query_as::<_, ModelInfo>(
            r#"
            SELECT id, provider, model_name, input_price_per_1k, output_price_per_1k, context_window, last_synced
            FROM model_info
            WHERE provider = $1 AND model_name = $2
            "#,
        )
        .bind(provider)
        .bind(model_name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(record)
    }

    pub async fn get_pricing(
        &self,
        provider: &str,
        model_name: &str,
    ) -> Result<Option<(f64, f64)>, DatabaseError> {
        let record = sqlx::query_as::<_, (f64, f64)>(
            r#"
            SELECT input_price_per_1k, output_price_per_1k
            FROM model_info
            WHERE provider = $1 AND model_name = $2
            "#,
        )
        .bind(provider)
        .bind(model_name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(record)
    }

    pub async fn calculate_cost(
        &self,
        provider: &str,
        model_name: &str,
        input_tokens: i64,
        output_tokens: i64,
    ) -> Result<Option<f64>, DatabaseError> {
        let pricing = self.get_pricing(provider, model_name).await?;

        Ok(pricing.map(|(input_price, output_price)| {
            (input_tokens as f64 / 1000.0) * input_price
                + (output_tokens as f64 / 1000.0) * output_price
        }))
    }

    pub async fn get_providers(&self) -> Result<Vec<String>, DatabaseError> {
        let records = sqlx::query_as::<_, (String,)>(
            r#"
            SELECT DISTINCT provider
            FROM model_info
            ORDER BY provider ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(records.into_iter().map(|r| r.0).collect())
    }

    pub async fn search_by_name(&self, query: &str) -> Result<Vec<ModelInfo>, DatabaseError> {
        let pattern = format!("%{}%", query.to_lowercase());
        let records = sqlx::query_as::<_, ModelInfo>(
            r#"
            SELECT id, provider, model_name, input_price_per_1k, output_price_per_1k, context_window, last_synced
            FROM model_info
            WHERE LOWER(model_name) LIKE $1 OR LOWER(provider) LIKE $1
            ORDER BY provider ASC, model_name ASC
            "#,
        )
        .bind(&pattern)
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }

    pub async fn count(&self) -> Result<i64, DatabaseError> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM model_info")
            .fetch_one(&self.pool)
            .await?;

        Ok(count.0)
    }

    pub async fn update_pricing(
        &self,
        provider: &str,
        model_name: &str,
        input_price_per_1k: f64,
        output_price_per_1k: f64,
    ) -> Result<Option<ModelInfo>, DatabaseError> {
        let record = sqlx::query_as::<_, ModelInfo>(
            r#"
            UPDATE model_info
            SET input_price_per_1k = $3, output_price_per_1k = $4, last_synced = NOW()
            WHERE provider = $1 AND model_name = $2
            RETURNING id, provider, model_name, input_price_per_1k, output_price_per_1k, context_window, last_synced
            "#,
        )
        .bind(provider)
        .bind(model_name)
        .bind(input_price_per_1k)
        .bind(output_price_per_1k)
        .fetch_optional(&self.pool)
        .await?;

        Ok(record)
    }
}

#[async_trait]
impl Repository for ModelRepository {
    type Entity = ModelInfo;
    type Id = Uuid;

    async fn get_by_id(&self, id: Uuid) -> Result<Option<ModelInfo>, DatabaseError> {
        let record = sqlx::query_as::<_, ModelInfo>(
            r#"
            SELECT id, provider, model_name, input_price_per_1k, output_price_per_1k, context_window, last_synced
            FROM model_info
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(record)
    }

    async fn get_all(&self) -> Result<Vec<ModelInfo>, DatabaseError> {
        let records = sqlx::query_as::<_, ModelInfo>(
            r#"
            SELECT id, provider, model_name, input_price_per_1k, output_price_per_1k, context_window, last_synced
            FROM model_info
            ORDER BY provider ASC, model_name ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }

    async fn delete(&self, id: Uuid) -> Result<bool, DatabaseError> {
        let result = sqlx::query("DELETE FROM model_info WHERE id = $1")
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
    fn test_model_info_creation() {
        let model = ModelInfo::new(
            "anthropic".to_string(),
            "claude-opus-4-5".to_string(),
            0.015,
            0.075,
            200000,
        );

        assert_eq!(model.provider, "anthropic");
        assert_eq!(model.model_name, "claude-opus-4-5");
        assert_eq!(model.full_name(), "anthropic/claude-opus-4-5");
    }

    #[test]
    fn test_model_cost_calculation() {
        let model = ModelInfo::new(
            "openai".to_string(),
            "gpt-4o".to_string(),
            0.005,
            0.015,
            128000,
        );

        let cost = model.calculate_cost(10000, 5000);
        let expected = (10000.0 / 1000.0) * 0.005 + (5000.0 / 1000.0) * 0.015;

        assert!((cost - expected).abs() < 0.0001);
    }
}
