use std::env;
use dotenvy::dotenv;

/// API configuration loaded from environment variables
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// Environment (development, production)
    #[allow(dead_code)]
    pub environment: String,
    /// Application port
    pub port: Option<u16>,
    /// Database URL
    #[allow(dead_code)]
    pub database_url: Option<String>,
    /// Redis URL
    #[allow(dead_code)]
    pub redis_url: Option<String>,
    /// Admin API key for authentication
    pub admin_api_key: String,
}

impl AppConfig {
    /// Load configuration from environment variables
    pub fn load() -> anyhow::Result<Self> {
        // Load .env file if present
        let _ = dotenv();
        
        // Read configuration from environment variables
        let environment = env::var("ENVIRONMENT")
            .unwrap_or_else(|_| "development".into());
        
        // Parse PORT if available    
        let port = env::var("PORT")
            .ok()
            .and_then(|p| p.parse::<u16>().ok());
            
        let database_url = env::var("DATABASE_URL").ok();
        let redis_url = env::var("REDIS_URL").ok();
        
        // Get admin API key, use a default for development only
        let admin_api_key = env::var("ADMIN_API_KEY")
            .unwrap_or_else(|_| {
                if environment == "development" {
                    "dev-admin-api-key-insecure".to_string()
                } else {
                    panic!("ADMIN_API_KEY environment variable must be set in non-development environments")
                }
            });
        
        Ok(Self {
            environment,
            port,
            database_url,
            redis_url,
            admin_api_key,
        })
    }
}
