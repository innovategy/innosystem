use async_trait::async_trait;
use uuid::Uuid;
use anyhow::Result;
use chrono::NaiveDateTime;

use crate::models::runner::Runner;
use crate::models::runner::NewRunner;
use crate::models::job_type::JobType;

/// Repository trait for Runner operations
#[async_trait]
pub trait RunnerRepository: Send + Sync {
    /// Register a new runner
    async fn register(&self, runner: NewRunner) -> Result<Runner>;
    
    /// Update a runner's heartbeat timestamp
    async fn update_heartbeat(&self, id: Uuid, timestamp: NaiveDateTime) -> Result<Runner>;
    
    /// Find a runner by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Runner>;
    
    /// Update a runner's capabilities
    async fn update_capabilities(&self, id: Uuid, job_types: Vec<Uuid>) -> Result<Runner>;
    
    /// List all runners
    async fn list_all(&self) -> Result<Vec<Runner>>;
    
    /// List active runners (with recent heartbeat)
    async fn list_active(&self, since: NaiveDateTime) -> Result<Vec<Runner>>;
    
    /// Find runners compatible with a specific job type
    async fn find_compatible_with_job_type(&self, job_type: &JobType) -> Result<Vec<Runner>>;
    
    /// Set runner status (active/inactive)
    async fn set_status(&self, id: Uuid, active: bool) -> Result<Runner>;
}
