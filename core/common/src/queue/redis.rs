use async_trait::async_trait;
use bb8_redis::{
    bb8::Pool,
    redis::{AsyncCommands, RedisResult},
    RedisConnectionManager,
};

use uuid::Uuid;

use crate::models::job::PriorityLevel;
use crate::queue::{JobQueue, JobQueueConfig, QueueError};

/// Redis implementation of the JobQueue trait
pub struct RedisJobQueue {
    pool: Pool<RedisConnectionManager>,
    config: JobQueueConfig,
}

impl RedisJobQueue {
    /// Create a new Redis job queue
    pub async fn new(config: JobQueueConfig) -> Result<Self, QueueError> {
        let manager = RedisConnectionManager::new(config.redis_url.clone())
            .map_err(|e| QueueError::Connection(format!("Failed to create Redis manager: {}", e)))?;

        let pool = Pool::builder()
            .max_size(config.pool_size)
            .build(manager)
            .await
            .map_err(|e| QueueError::Connection(format!("Failed to create Redis pool: {}", e)))?;

        Ok(Self { pool, config })
    }

    /// Get the Redis key for a priority queue
    fn priority_queue_key(&self, priority: PriorityLevel) -> String {
        format!("{}:p{}:pending", self.config.key_prefix, priority.as_i32())
    }

    /// Get the Redis key for the scheduled queue
    fn scheduled_queue_key(&self) -> String {
        format!("{}:scheduled", self.config.key_prefix)
    }
}

#[async_trait]
impl JobQueue for RedisJobQueue {
    async fn push_job(&self, job_id: Uuid, priority: PriorityLevel) -> Result<(), QueueError> {
        let mut conn = self.pool.get().await
            .map_err(|e| QueueError::Connection(format!("Failed to get Redis connection: {}", e)))?;

        let queue_key = self.priority_queue_key(priority);
        let job_id_str = job_id.to_string();

        // Push the job ID to the appropriate priority queue
        let _: () = conn.lpush(&queue_key, &job_id_str).await
            .map_err(|e| QueueError::Redis(e))?;

        Ok(())
    }

    async fn pop_job(&self) -> Result<Option<Uuid>, QueueError> {
        self.pop_job_with_timeout(self.config.timeout_seconds).await
    }

    async fn pop_job_with_timeout(&self, timeout_seconds: u64) -> Result<Option<Uuid>, QueueError> {
        let mut conn = self.pool.get().await
            .map_err(|e| QueueError::Connection(format!("Failed to get Redis connection: {}", e)))?;

        // Create keys for all priority queues, highest priority first
        let queue_keys: Vec<String> = vec![
            self.priority_queue_key(PriorityLevel::Critical),
            self.priority_queue_key(PriorityLevel::High),
            self.priority_queue_key(PriorityLevel::Medium),
            self.priority_queue_key(PriorityLevel::Low),
        ];

        // Try to pop a job from any queue in priority order with timeout
        let result: RedisResult<Option<(String, String)>> = conn
            .brpop(&queue_keys, timeout_seconds as f64)
            .await;

        match result {
            Ok(Some((_, job_id_str))) => {
                // Parse the job ID
                match Uuid::parse_str(&job_id_str) {
                    Ok(job_id) => Ok(Some(job_id)),
                    Err(_) => Err(QueueError::JobAcquisition(format!("Invalid job ID format: {}", job_id_str))),
                }
            }
            Ok(None) => Ok(None), // Timeout, no job available
            Err(e) => Err(QueueError::Redis(e)),
        }
    }

    async fn queue_length(&self) -> Result<usize, QueueError> {
        let mut total = 0;
        
        // Sum lengths of all priority queues
        for priority in [
            PriorityLevel::Critical,
            PriorityLevel::High,
            PriorityLevel::Medium,
            PriorityLevel::Low,
        ] {
            total += self.queue_length_by_priority(priority).await?;
        }
        
        Ok(total)
    }

    async fn queue_length_by_priority(&self, priority: PriorityLevel) -> Result<usize, QueueError> {
        let mut conn = self.pool.get().await
            .map_err(|e| QueueError::Connection(format!("Failed to get Redis connection: {}", e)))?;

        let queue_key = self.priority_queue_key(priority);
        
        let length: usize = conn.llen(&queue_key).await
            .map_err(|e| QueueError::Redis(e))?;
            
        Ok(length)
    }

    async fn peek_next_job(&self) -> Result<Option<Uuid>, QueueError> {
        let mut conn = self.pool.get().await
            .map_err(|e| QueueError::Connection(format!("Failed to get Redis connection: {}", e)))?;

        // Try to get the next job from each priority queue (in order)
        for priority in [
            PriorityLevel::Critical,
            PriorityLevel::High,
            PriorityLevel::Medium,
            PriorityLevel::Low,
        ] {
            let queue_key = self.priority_queue_key(priority);
            
            let result: Option<String> = conn.lindex(&queue_key, -1).await
                .map_err(|e| QueueError::Redis(e))?;
                
            if let Some(job_id_str) = result {
                return match Uuid::parse_str(&job_id_str) {
                    Ok(job_id) => Ok(Some(job_id)),
                    Err(_) => Err(QueueError::JobAcquisition(format!("Invalid job ID format: {}", job_id_str))),
                };
            }
        }
        
        Ok(None) // No jobs in any queue
    }

    async fn schedule_job(&self, job_id: Uuid, execute_at: chrono::DateTime<chrono::Utc>) -> Result<(), QueueError> {
        let mut conn = self.pool.get().await
            .map_err(|e| QueueError::Connection(format!("Failed to get Redis connection: {}", e)))?;

        let scheduled_key = self.scheduled_queue_key();
        let job_id_str = job_id.to_string();
        let score = execute_at.timestamp_millis() as f64;

        // Add job to sorted set with score as execution time
        let _: () = conn.zadd(&scheduled_key, &job_id_str, score).await
            .map_err(|e| QueueError::Redis(e))?;

        Ok(())
    }

    async fn get_due_scheduled_jobs(&self) -> Result<Vec<Uuid>, QueueError> {
        let mut conn = self.pool.get().await
            .map_err(|e| QueueError::Connection(format!("Failed to get Redis connection: {}", e)))?;

        let scheduled_key = self.scheduled_queue_key();
        let now = chrono::Utc::now().timestamp_millis() as f64;

        // Get all jobs with score (execution time) less than or equal to now
        let job_ids: Vec<String> = conn.zrangebyscore(&scheduled_key, 0.0, now).await
            .map_err(|e| QueueError::Redis(e))?;

        // Parse job IDs and return
        let mut result = Vec::with_capacity(job_ids.len());
        for job_id_str in job_ids {
            match Uuid::parse_str(&job_id_str) {
                Ok(job_id) => result.push(job_id),
                Err(_) => return Err(QueueError::JobAcquisition(format!("Invalid job ID format: {}", job_id_str))),
            }
        }

        // Remove the retrieved jobs from the scheduled queue
        if !result.is_empty() {
            let job_id_strs: Vec<String> = result.iter().map(|id| id.to_string()).collect();
            let _: () = conn.zrem(&scheduled_key, &job_id_strs).await
                .map_err(|e| QueueError::Redis(e))?;
        }

        Ok(result)
    }
}
