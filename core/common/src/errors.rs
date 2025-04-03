use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Database error: {0}")]
    Database(#[from] diesel::result::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("Job queue error: {0}")]
    JobQueue(String),

    #[error("Entity not found: {0}")]
    NotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Insufficient funds for wallet: {0}")]
    InsufficientFunds(String),

    #[error("Unauthorized access: {0}")]
    Unauthorized(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
