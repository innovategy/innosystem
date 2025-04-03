use async_trait::async_trait;
use uuid::Uuid;

use crate::models::job_type::{JobType, NewJobType};
use crate::Result;

#[async_trait]
pub trait JobTypeRepository: Send + Sync {
    async fn create(&self, new_job_type: NewJobType) -> Result<JobType>;
    async fn find_by_id(&self, id: Uuid) -> Result<JobType>;
    async fn update(&self, job_type: JobType) -> Result<JobType>;
    async fn list_all(&self) -> Result<Vec<JobType>>;
    async fn list_enabled(&self) -> Result<Vec<JobType>>;
}
