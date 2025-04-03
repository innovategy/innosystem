use async_trait::async_trait;
use uuid::Uuid;

use crate::models::job::PriorityLevel;
use crate::queue::error::QueueError;

/// Configuration for a job queue
#[derive(Debug, Clone)]
pub struct JobQueueConfig {
    /// Redis URL (e.g., "redis://127.0.0.1:6379")
    pub redis_url: String,
    /// Base key prefix for all queue keys
    pub key_prefix: String,
    /// Connection pool size
    pub pool_size: u32,
    /// Queue timeout in seconds
    pub timeout_seconds: u64,
}

impl JobQueueConfig {
    pub fn new(redis_url: String) -> Self {
        Self {
            redis_url,
            key_prefix: "innosystem:jobs".to_string(),
            pool_size: 10,
            timeout_seconds: 60,
        }
    }
    
    pub fn with_prefix(mut self, prefix: &str) -> Self {
        self.key_prefix = prefix.to_string();
        self
    }
    
    pub fn with_pool_size(mut self, size: u32) -> Self {
        self.pool_size = size;
        self
    }
    
    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.timeout_seconds = seconds;
        self
    }
}

/// Trait defining the job queue interface
#[async_trait]
pub trait JobQueue: Send + Sync {
    /// Push a job to the queue
    async fn push_job(&self, job_id: Uuid, priority: PriorityLevel) -> Result<(), QueueError>;
    
    /// Pop a job from the queue (blocking)
    async fn pop_job(&self) -> Result<Option<Uuid>, QueueError>;
    
    /// Pop a job from the queue with timeout
    async fn pop_job_with_timeout(&self, timeout_seconds: u64) -> Result<Option<Uuid>, QueueError>;
    
    /// Get the number of jobs in the queue
    async fn queue_length(&self) -> Result<usize, QueueError>;
    
    /// Get the number of jobs in the queue by priority
    async fn queue_length_by_priority(&self, priority: PriorityLevel) -> Result<usize, QueueError>;
    
    /// Peek at the next job in the queue without removing it
    async fn peek_next_job(&self) -> Result<Option<Uuid>, QueueError>;
    
    /// Schedule a job for future execution
    async fn schedule_job(&self, job_id: Uuid, execute_at: chrono::DateTime<chrono::Utc>) -> Result<(), QueueError>;
    
    /// Get jobs that are scheduled for execution now
    async fn get_due_scheduled_jobs(&self) -> Result<Vec<Uuid>, QueueError>;
}
