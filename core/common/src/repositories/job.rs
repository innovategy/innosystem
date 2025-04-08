use async_trait::async_trait;
use chrono::NaiveDateTime;
use uuid::Uuid;

use crate::models::job::{Job, JobStatus, NewJob, PriorityLevel};
use crate::Result;

/// Sorting options for job queries
pub enum JobSortOrder {
    /// Most recently created first
    CreatedDesc,
    /// Oldest created first
    CreatedAsc,
    /// Highest priority first
    PriorityDesc,
    /// Lowest priority first
    PriorityAsc,
}

/// Filter criteria for job queries
pub struct JobFilter {
    /// Filter by customer ID
    pub customer_id: Option<Uuid>,
    /// Filter by job type ID
    pub job_type_id: Option<Uuid>,
    /// Filter by job status
    pub status: Option<JobStatus>,
    /// Filter by priority level
    pub priority: Option<PriorityLevel>,
    /// Filter by jobs created after this timestamp
    pub created_after: Option<NaiveDateTime>,
    /// Filter by jobs created before this timestamp
    pub created_before: Option<NaiveDateTime>,
    /// Filter by completed jobs only
    pub completed_only: bool,
    /// Filter by failed jobs only
    pub failed_only: bool,
}

impl Default for JobFilter {
    fn default() -> Self {
        Self {
            customer_id: None,
            job_type_id: None,
            status: None,
            priority: None,
            created_after: None,
            created_before: None,
            completed_only: false,
            failed_only: false,
        }
    }
}

/// Pagination options for job queries
pub struct Pagination {
    /// Page number (0-based)
    pub page: u32,
    /// Items per page
    pub per_page: u32,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            page: 0,
            per_page: 10,
        }
    }
}

#[async_trait]
pub trait JobRepository: Send + Sync {
    // Basic CRUD operations
    async fn create(&self, new_job: NewJob) -> Result<Job>;
    async fn find_by_id(&self, id: Uuid) -> Result<Job>;
    async fn update_status(&self, id: Uuid, status: JobStatus) -> Result<Job>;
    async fn set_started(&self, id: Uuid) -> Result<Job>;
    async fn set_completed(&self, id: Uuid, success: bool, output: Option<serde_json::Value>, error: Option<String>, cost_cents: i32) -> Result<Job>;
    
    // Basic query operations (from original trait)
    async fn find_by_customer_id(&self, customer_id: Uuid) -> Result<Vec<Job>>;
    async fn find_by_status(&self, status: JobStatus) -> Result<Vec<Job>>;
    async fn find_pending_jobs(&self, limit: i32) -> Result<Vec<Job>>;
    
    // Advanced query operations (new methods for phase 2.2.2)
    /// Query jobs with advanced filtering, sorting and pagination
    async fn query_jobs(&self, filter: JobFilter, sort: Option<JobSortOrder>, pagination: Option<Pagination>) -> Result<(Vec<Job>, u64)>;
    
    /// Get job statistics grouped by status
    async fn get_job_stats_by_status(&self) -> Result<Vec<(String, i64)>>;
    
    /// Get job statistics grouped by customer
    async fn get_job_stats_by_customer(&self) -> Result<Vec<(Uuid, i64)>>;
    
    /// Get estimated vs actual cost statistics for completed jobs
    async fn get_cost_statistics(&self) -> Result<(i64, i64)>;
    
    /// Find jobs that have been in running state for too long (possibly stalled)
    async fn find_stalled_jobs(&self, running_threshold_minutes: i32) -> Result<Vec<Job>>;
    
    /// Update multiple jobs with the same status in a single operation
    async fn bulk_update_status(&self, ids: Vec<Uuid>, status: JobStatus) -> Result<usize>;
}
