use std::env;
use dotenvy::dotenv;

/// Runner configuration loaded from environment variables
#[derive(Debug, Clone)]
pub struct RunnerConfig {
    /// Redis connection URL
    pub redis_url: String,
    /// Environment (development, production)
    #[allow(dead_code)]
    pub environment: String,
    /// Database URL (for Phase 2)
    #[allow(dead_code)]
    pub database_url: Option<String>,
    /// Queue polling interval in milliseconds
    pub poll_interval_ms: u64,
    /// Queue timeout in seconds
    pub queue_timeout_seconds: u64,
    /// Maximum number of concurrent jobs
    #[allow(dead_code)]
    pub max_concurrent_jobs: usize,
}

impl RunnerConfig {
    /// Load configuration from environment variables
    pub fn load() -> anyhow::Result<Self> {
        // Load .env file if present
        let _ = dotenv();
        
        // Read configuration from environment variables
        let redis_url = env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://127.0.0.1:6379".into());
            
        let environment = env::var("ENVIRONMENT")
            .unwrap_or_else(|_| "development".into());
            
        let database_url = env::var("DATABASE_URL").ok();
        
        let poll_interval_ms = env::var("POLL_INTERVAL_MS")
            .unwrap_or_else(|_| "1000".into())
            .parse::<u64>()?;
            
        let queue_timeout_seconds = env::var("QUEUE_TIMEOUT_SECONDS")
            .unwrap_or_else(|_| "30".into())
            .parse::<u64>()?;
            
        let max_concurrent_jobs = env::var("MAX_CONCURRENT_JOBS")
            .unwrap_or_else(|_| "4".into())
            .parse::<usize>()?;
            
        Ok(Self {
            redis_url,
            environment,
            database_url,
            poll_interval_ms,
            queue_timeout_seconds,
            max_concurrent_jobs,
        })
    }
}
