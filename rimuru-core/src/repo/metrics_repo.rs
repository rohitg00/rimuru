use crate::db::DatabaseError;
use crate::models::SystemMetrics;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

pub struct MetricsRepository {
    pool: PgPool,
}

impl MetricsRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn record_snapshot(
        &self,
        metrics: &SystemMetrics,
    ) -> Result<SystemMetrics, DatabaseError> {
        let record = sqlx::query_as::<_, SystemMetrics>(
            r#"
            INSERT INTO system_metrics (timestamp, cpu_percent, memory_used_mb, memory_total_mb, active_sessions)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (timestamp) DO UPDATE SET
                cpu_percent = EXCLUDED.cpu_percent,
                memory_used_mb = EXCLUDED.memory_used_mb,
                memory_total_mb = EXCLUDED.memory_total_mb,
                active_sessions = EXCLUDED.active_sessions
            RETURNING timestamp, cpu_percent, memory_used_mb, memory_total_mb, active_sessions
            "#,
        )
        .bind(metrics.timestamp)
        .bind(metrics.cpu_percent)
        .bind(metrics.memory_used_mb)
        .bind(metrics.memory_total_mb)
        .bind(metrics.active_sessions)
        .fetch_one(&self.pool)
        .await?;

        Ok(record)
    }

    pub async fn get_latest(&self) -> Result<Option<SystemMetrics>, DatabaseError> {
        let record = sqlx::query_as::<_, SystemMetrics>(
            r#"
            SELECT timestamp, cpu_percent, memory_used_mb, memory_total_mb, active_sessions
            FROM system_metrics
            ORDER BY timestamp DESC
            LIMIT 1
            "#,
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(record)
    }

    pub async fn get_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<SystemMetrics>, DatabaseError> {
        let records = sqlx::query_as::<_, SystemMetrics>(
            r#"
            SELECT timestamp, cpu_percent, memory_used_mb, memory_total_mb, active_sessions
            FROM system_metrics
            WHERE timestamp >= $1 AND timestamp <= $2
            ORDER BY timestamp ASC
            "#,
        )
        .bind(start)
        .bind(end)
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }

    pub async fn get_recent(&self, limit: i64) -> Result<Vec<SystemMetrics>, DatabaseError> {
        let records = sqlx::query_as::<_, SystemMetrics>(
            r#"
            SELECT timestamp, cpu_percent, memory_used_mb, memory_total_mb, active_sessions
            FROM system_metrics
            ORDER BY timestamp DESC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }

    pub async fn get_average_cpu(&self, minutes: i32) -> Result<f32, DatabaseError> {
        let record = sqlx::query_as::<_, (f32,)>(
            r#"
            SELECT COALESCE(AVG(cpu_percent), 0.0)::REAL
            FROM system_metrics
            WHERE timestamp >= NOW() - ($1 || ' minutes')::INTERVAL
            "#,
        )
        .bind(minutes)
        .fetch_one(&self.pool)
        .await?;

        Ok(record.0)
    }

    pub async fn get_average_memory(&self, minutes: i32) -> Result<(i64, i64), DatabaseError> {
        let record = sqlx::query_as::<_, (i64, i64)>(
            r#"
            SELECT
                COALESCE(AVG(memory_used_mb), 0)::BIGINT,
                COALESCE(AVG(memory_total_mb), 0)::BIGINT
            FROM system_metrics
            WHERE timestamp >= NOW() - ($1 || ' minutes')::INTERVAL
            "#,
        )
        .bind(minutes)
        .fetch_one(&self.pool)
        .await?;

        Ok(record)
    }

    pub async fn get_peak_cpu(&self, minutes: i32) -> Result<f32, DatabaseError> {
        let record = sqlx::query_as::<_, (f32,)>(
            r#"
            SELECT COALESCE(MAX(cpu_percent), 0.0)::REAL
            FROM system_metrics
            WHERE timestamp >= NOW() - ($1 || ' minutes')::INTERVAL
            "#,
        )
        .bind(minutes)
        .fetch_one(&self.pool)
        .await?;

        Ok(record.0)
    }

    pub async fn get_peak_memory(&self, minutes: i32) -> Result<i64, DatabaseError> {
        let record = sqlx::query_as::<_, (i64,)>(
            r#"
            SELECT COALESCE(MAX(memory_used_mb), 0)::BIGINT
            FROM system_metrics
            WHERE timestamp >= NOW() - ($1 || ' minutes')::INTERVAL
            "#,
        )
        .bind(minutes)
        .fetch_one(&self.pool)
        .await?;

        Ok(record.0)
    }

    pub async fn cleanup_old_metrics(&self, days_to_keep: i32) -> Result<i64, DatabaseError> {
        let result = sqlx::query_as::<_, (i32,)>("SELECT cleanup_old_system_metrics($1)")
            .bind(days_to_keep)
            .fetch_one(&self.pool)
            .await?;

        Ok(result.0 as i64)
    }

    pub async fn count(&self) -> Result<i64, DatabaseError> {
        let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM system_metrics")
            .fetch_one(&self.pool)
            .await?;

        Ok(count.0)
    }

    pub async fn count_in_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<i64, DatabaseError> {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM system_metrics WHERE timestamp >= $1 AND timestamp <= $2",
        )
        .bind(start)
        .bind(end)
        .fetch_one(&self.pool)
        .await?;

        Ok(count.0)
    }

    pub async fn delete_before(&self, before: DateTime<Utc>) -> Result<i64, DatabaseError> {
        let result = sqlx::query("DELETE FROM system_metrics WHERE timestamp < $1")
            .bind(before)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_metrics_creation() {
        let metrics = SystemMetrics::new(45.5, 8192, 16384, 3);

        assert_eq!(metrics.cpu_percent, 45.5);
        assert_eq!(metrics.memory_used_mb, 8192);
        assert_eq!(metrics.memory_total_mb, 16384);
        assert_eq!(metrics.active_sessions, 3);
        assert_eq!(metrics.memory_percent(), 50.0);
    }

    #[test]
    fn test_memory_calculations() {
        let metrics = SystemMetrics::new(25.0, 4096, 16384, 2);

        assert_eq!(metrics.memory_percent(), 25.0);
        assert_eq!(metrics.memory_available_mb(), 12288);
    }
}
