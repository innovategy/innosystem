mod default;

pub use default::DefaultJobProcessor;
use innosystem_common::models::job::Job;

/// Trait for job processors
#[async_trait::async_trait]
pub trait JobProcessor: Send + Sync {
    /// Process a job and return the output data and cost if successful
    async fn process_job(&self, job: Job) -> anyhow::Result<(serde_json::Value, i32)>;
}
