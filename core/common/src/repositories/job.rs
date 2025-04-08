use async_trait::async_trait;
use uuid::Uuid;

use crate::models::job::{Job, JobStatus, NewJob};
use crate::Result;

#[async_trait]
pub trait JobRepository: Send + Sync {
    async fn create(&self, new_job: NewJob) -> Result<Job>;
    async fn find_by_id(&self, id: Uuid) -> Result<Job>;
    async fn update_status(&self, id: Uuid, status: JobStatus) -> Result<Job>;
    async fn set_started(&self, id: Uuid) -> Result<Job>;
    async fn set_completed(&self, id: Uuid, success: bool, output: Option<serde_json::Value>, error: Option<String>, cost_cents: i32) -> Result<Job>;
    async fn find_by_customer_id(&self, customer_id: Uuid) -> Result<Vec<Job>>;
    async fn find_by_status(&self, status: JobStatus) -> Result<Vec<Job>>;
    async fn find_pending_jobs(&self, limit: i32) -> Result<Vec<Job>>;
}
