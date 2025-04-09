use async_trait::async_trait;
use uuid::Uuid;
use anyhow::Result;

use crate::models::project::{Project, NewProject};

/// Repository trait for Project operations
#[async_trait]
pub trait ProjectRepository: Send + Sync {
    /// Create a new project
    async fn create(&self, project: NewProject) -> Result<Project>;
    
    /// Find a project by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Project>;
    
    /// Find projects for a specific customer
    async fn find_by_customer_id(&self, customer_id: Uuid) -> Result<Vec<Project>>;
    
    /// Update a project
    async fn update(&self, project: &Project) -> Result<Project>;
    
    /// List all projects
    async fn list_all(&self) -> Result<Vec<Project>>;
    
    /// Delete a project
    async fn delete(&self, id: Uuid) -> Result<()>;
}
