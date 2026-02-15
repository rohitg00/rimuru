use anyhow::{anyhow, Context, Result};
use std::path::PathBuf;

const DEFAULT_DATABASE_URL: &str = "postgres://localhost/rimuru_dev";

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct CliConfig {
    pub database_url: String,
    pub log_level: String,
    pub metrics_interval_secs: u64,
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            database_url: DEFAULT_DATABASE_URL.to_string(),
            log_level: "info".to_string(),
            metrics_interval_secs: 5,
        }
    }
}

#[allow(dead_code)]
impl CliConfig {
    pub fn load() -> Result<Self> {
        load_dotenv_files();

        let database_url = std::env::var("DATABASE_URL")
            .or_else(|_| std::env::var("RIMURU_DATABASE_URL"))
            .context(
                "DATABASE_URL environment variable not set. \n\
                 Please set DATABASE_URL in your environment or create a .env file with:\n\
                 DATABASE_URL=postgres://user:password@localhost/rimuru_dev",
            )?;

        let log_level = std::env::var("RIMURU_LOG_LEVEL")
            .or_else(|_| std::env::var("RUST_LOG"))
            .unwrap_or_else(|_| "info".to_string());

        let metrics_interval_secs = std::env::var("RIMURU_METRICS_INTERVAL")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(5);

        Ok(Self {
            database_url,
            log_level,
            metrics_interval_secs,
        })
    }

    pub fn database_url(&self) -> &str {
        &self.database_url
    }

    pub fn log_level(&self) -> &str {
        &self.log_level
    }

    pub fn metrics_interval_secs(&self) -> u64 {
        self.metrics_interval_secs
    }
}

fn load_dotenv_files() {
    let current_dir = std::env::current_dir().ok();

    let env_paths = [
        current_dir.as_ref().map(|d| d.join(".env")),
        current_dir.as_ref().map(|d| d.join(".env.local")),
        dirs::home_dir().map(|d| d.join(".rimuru").join(".env")),
        dirs::config_dir().map(|d| d.join("rimuru").join(".env")),
    ];

    for path in env_paths.iter().flatten() {
        if path.exists() {
            let _ = dotenvy::from_path(path);
        }
    }
}

#[allow(dead_code)]
pub fn get_config_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("rimuru"))
}

#[allow(dead_code)]
pub fn get_data_dir() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join("rimuru"))
}

#[allow(dead_code)]
pub fn ensure_config_dir() -> Result<PathBuf> {
    let config_dir =
        get_config_dir().ok_or_else(|| anyhow!("Could not determine config directory"))?;

    if !config_dir.exists() {
        std::fs::create_dir_all(&config_dir).context("Failed to create config directory")?;
    }

    Ok(config_dir)
}

#[allow(dead_code)]
pub fn ensure_data_dir() -> Result<PathBuf> {
    let data_dir = get_data_dir().ok_or_else(|| anyhow!("Could not determine data directory"))?;

    if !data_dir.exists() {
        std::fs::create_dir_all(&data_dir).context("Failed to create data directory")?;
    }

    Ok(data_dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = CliConfig::default();
        assert_eq!(config.database_url, DEFAULT_DATABASE_URL);
        assert_eq!(config.log_level, "info");
        assert_eq!(config.metrics_interval_secs, 5);
    }

    #[test]
    fn test_config_dir_functions() {
        let config_dir = get_config_dir();
        assert!(config_dir.is_some());

        let data_dir = get_data_dir();
        assert!(data_dir.is_some());
    }
}
