use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum QueueError {
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
    
    #[error("Failed to serialize job: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Job with ID {0} not found in queue")]
    JobNotFound(Uuid),
    
    #[error("Failed to acquire job: {0}")]
    JobAcquisition(String),
    
    #[error("Queue connection error: {0}")]
    Connection(String),
    
    #[error("Invalid queue configuration: {0}")]
    Configuration(String),
    
    #[error("Queue operation timeout")]
    Timeout,
}
