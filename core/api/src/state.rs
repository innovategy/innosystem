use std::sync::Arc;

use innosystem_common::{
    queue::{JobQueue, JobQueueConfig, RedisJobQueue, QueueError},
    repositories::{CustomerRepository, JobRepository, JobTypeRepository, WalletRepository},
    repositories::in_memory::{InMemoryCustomerRepository, InMemoryJobRepository, InMemoryJobTypeRepository, InMemoryWalletRepository},
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
    /// Create a new application state with in-memory repositories for development
    #[allow(dead_code)]
    pub async fn new(config: AppConfig) -> Result<Self, QueueError> {
        // Use the in-memory implementations from common crate
        let customer_repo = Arc::new(InMemoryCustomerRepository::new());
        let job_repo = Arc::new(InMemoryJobRepository::new());
        let job_type_repo = Arc::new(InMemoryJobTypeRepository::new());
        let wallet_repo = Arc::new(InMemoryWalletRepository::new());
        
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
