use std::env;

/// Configuration for the application
#[derive(Debug, Clone)]
pub struct Config {
    /// Environment (development, production)
    pub environment: String,
    /// Port to run the API server on
    pub port: u16,
    /// Redis connection URL
    pub redis_url: String,
    /// Polling interval for the job queue in milliseconds
    pub poll_interval_ms: u64,
    /// Timeout for queue operations in seconds
    pub queue_timeout_seconds: u64,
    /// Maximum number of concurrent jobs
    pub max_concurrent_jobs: usize,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        Self {
            environment: env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .unwrap_or(3000),
            redis_url: env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
            poll_interval_ms: env::var("POLL_INTERVAL_MS")
                .unwrap_or_else(|_| "1000".to_string())
                .parse()
                .unwrap_or(1000),
            queue_timeout_seconds: env::var("QUEUE_TIMEOUT_SECONDS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),
            max_concurrent_jobs: env::var("MAX_CONCURRENT_JOBS")
                .unwrap_or_else(|_| "4".to_string())
                .parse()
                .unwrap_or(4),
        }
    }
    
    /// Check if we're in development mode
    pub fn is_development(&self) -> bool {
        self.environment == "development"
    }
}
