use sqlx::migrate::Migrator;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;
use tracing::info;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout_secs: u64,
    pub idle_timeout_secs: u64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            max_connections: 10,
            min_connections: 1,
            connect_timeout_secs: 30,
            idle_timeout_secs: 600,
        }
    }
}

impl DatabaseConfig {
    pub fn from_env() -> Result<Self, DatabaseError> {
        let url = std::env::var("DATABASE_URL")
            .map_err(|_| DatabaseError::MissingEnvVar("DATABASE_URL".to_string()))?;

        let max_connections = std::env::var("DB_MAX_CONNECTIONS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(10);

        let min_connections = std::env::var("DB_MIN_CONNECTIONS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1);

        let connect_timeout_secs = std::env::var("DB_CONNECT_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(30);

        let idle_timeout_secs = std::env::var("DB_IDLE_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(600);

        Ok(Self {
            url,
            max_connections,
            min_connections,
            connect_timeout_secs,
            idle_timeout_secs,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("Missing environment variable: {0}")]
    MissingEnvVar(String),

    #[error("Failed to connect to database: {0}")]
    ConnectionFailed(#[from] sqlx::Error),

    #[error("Migration failed: {0}")]
    MigrationFailed(#[source] sqlx::migrate::MigrateError),

    #[error("Invalid database configuration: {0}")]
    InvalidConfig(String),
}

#[derive(Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn connect(config: &DatabaseConfig) -> Result<Self, DatabaseError> {
        info!("Connecting to database...");

        let pool = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .acquire_timeout(Duration::from_secs(config.connect_timeout_secs))
            .idle_timeout(Duration::from_secs(config.idle_timeout_secs))
            .connect(&config.url)
            .await?;

        info!("Database connection pool established");

        Ok(Self { pool })
    }

    pub async fn connect_with_url(url: &str) -> Result<Self, DatabaseError> {
        let config = DatabaseConfig {
            url: url.to_string(),
            ..Default::default()
        };
        Self::connect(&config).await
    }

    pub async fn run_migrations(&self) -> Result<(), DatabaseError> {
        info!("Running database migrations...");

        MIGRATOR
            .run(&self.pool)
            .await
            .map_err(DatabaseError::MigrationFailed)?;

        info!("Database migrations completed successfully");
        Ok(())
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn health_check(&self) -> Result<(), DatabaseError> {
        sqlx::query("SELECT 1").execute(&self.pool).await?;
        Ok(())
    }

    pub async fn close(&self) {
        info!("Closing database connection pool...");
        self.pool.close().await;
        info!("Database connection pool closed");
    }
}

pub async fn init_database() -> Result<Database, DatabaseError> {
    dotenvy::dotenv().ok();

    let config = DatabaseConfig::from_env()?;
    let db = Database::connect(&config).await?;
    db.run_migrations().await?;

    Ok(db)
}

pub async fn init_database_with_url(url: &str) -> Result<Database, DatabaseError> {
    let db = Database::connect_with_url(url).await?;
    db.run_migrations().await?;
    Ok(db)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_config_default() {
        let config = DatabaseConfig::default();
        assert_eq!(config.max_connections, 10);
        assert_eq!(config.min_connections, 1);
        assert_eq!(config.connect_timeout_secs, 30);
        assert_eq!(config.idle_timeout_secs, 600);
    }

    #[test]
    fn test_database_config_from_env_missing_url() {
        std::env::remove_var("DATABASE_URL");
        let result = DatabaseConfig::from_env();
        assert!(matches!(result, Err(DatabaseError::MissingEnvVar(_))));
    }
}
