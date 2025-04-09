use std::sync::Arc;

use diesel;
use innosystem_common::{
    queue::{JobQueue, JobQueueConfig, RedisJobQueue, QueueError},
    repositories::{CustomerRepository, JobRepository, JobTypeRepository, WalletRepository},
    repositories::{DieselCustomerRepository, DieselJobRepository, DieselJobTypeRepository, DieselWalletRepository},
};

use crate::config::AppConfig;

/// Application state shared across API handlers
/// Kept as a contract for the application's shared state
#[derive(Clone)]
pub struct AppState {
    #[allow(dead_code)]
    pub customer_repo: Arc<dyn CustomerRepository>,
    pub job_repo: Arc<dyn JobRepository>,
    #[allow(dead_code)]
    pub job_type_repo: Arc<dyn JobTypeRepository>,
    #[allow(dead_code)]
    pub wallet_repo: Arc<dyn WalletRepository>,
    pub job_queue: Arc<dyn JobQueue>,
    #[allow(dead_code)]
    pub config: AppConfig,
}

impl AppState {
    /// Create a new application state (uses Diesel repositories as in-memory repositories were removed in Phase 3)
    #[allow(dead_code)]
    pub async fn new(config: AppConfig) -> Result<Self, QueueError> {
        // In Phase 3, we use Diesel repositories for all environments
        Self::new_with_diesel(config).await
    }
    
    /// Create a new application state with Diesel repositories for production
    pub async fn new_with_diesel(config: AppConfig) -> Result<Self, QueueError> {
        // Get database URL from config or use default
        let database_url = config.database_url.clone().unwrap_or_else(|| "postgres://postgres:postgres@postgres:5432/innosystem".to_string());
        
        // Create a database connection manager
        let manager = diesel::r2d2::ConnectionManager::<diesel::pg::PgConnection>::new(database_url);
        
        // Build the connection pool
        let pool = diesel::r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to establish database connection");
        
        // Use the Diesel implementations from common crate
        let customer_repo = Arc::new(DieselCustomerRepository::new(pool.clone()));
        let job_repo = Arc::new(DieselJobRepository::new(pool.clone()));
        let job_type_repo = Arc::new(DieselJobTypeRepository::new(pool.clone()));
        let wallet_repo = Arc::new(DieselWalletRepository::new(pool.clone()));
        
        // Initialize Redis job queue
        let queue_config = JobQueueConfig::new(config.redis_url.clone().unwrap_or_else(|| "redis://redis:6379".to_string()));
        let job_queue = Arc::new(RedisJobQueue::new(queue_config).await?);

        Ok(AppState {
            customer_repo,
            job_repo,
            job_type_repo,
            wallet_repo,
            job_queue,
            config,
        })
    }
}
