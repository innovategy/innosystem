use async_trait::async_trait;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use uuid::Uuid;
use anyhow::{Result, anyhow};
use chrono::Utc;

use crate::models::project::{Project, NewProject};
use crate::repositories::ProjectRepository;
use crate::diesel_schema::projects;

/// Diesel implementation of the ProjectRepository
pub struct DieselProjectRepository {
    pool: Pool<ConnectionManager<PgConnection>>,
}

impl DieselProjectRepository {
    /// Create a new DieselProjectRepository with the given connection pool
    pub fn new(pool: Pool<ConnectionManager<PgConnection>>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProjectRepository for DieselProjectRepository {
    async fn create(&self, project: NewProject) -> Result<Project> {
        let mut conn = self.pool.get()?;
        
        // Insert the new project
        let project: Project = tokio::task::spawn_blocking(move || {
            diesel::insert_into(projects::table)
                .values(&project)
                .get_result::<Project>(&mut conn)
        }).await??;
        
        Ok(project)
    }
    
    async fn find_by_id(&self, id: Uuid) -> Result<Project> {
        let mut conn = self.pool.get()?;
        
        let project: Project = tokio::task::spawn_blocking(move || {
            projects::table
                .find(id)
                .first(&mut conn)
                .optional()
        }).await??
            .ok_or_else(|| anyhow!("Project not found with ID: {}", id))?;
        
        Ok(project)
    }
    
    async fn find_by_customer_id(&self, customer_id: Uuid) -> Result<Vec<Project>> {
        let mut conn = self.pool.get()?;
        
        let projects: Vec<Project> = tokio::task::spawn_blocking(move || {
            projects::table
                .filter(projects::customer_id.eq(customer_id))
                .load::<Project>(&mut conn)
        }).await??;
        
        Ok(projects)
    }
    
    async fn update(&self, project: &Project) -> Result<Project> {
        let project_clone = project.clone();
        let mut conn = self.pool.get()?;
        
        // Create an updated project with the current timestamp
        let mut updated_project = project_clone.clone();
        updated_project.updated_at = Some(Utc::now().naive_utc());
        
        let updated_project = tokio::task::spawn_blocking(move || {
            diesel::update(projects::table.find(project_clone.id))
                .set((
                    projects::name.eq(&updated_project.name),
                    projects::description.eq(&updated_project.description),
                    projects::updated_at.eq(updated_project.updated_at),
                ))
                .get_result::<Project>(&mut conn)
        }).await??;
        
        Ok(updated_project)
    }
    
    async fn list_all(&self) -> Result<Vec<Project>> {
        let mut conn = self.pool.get()?;
        
        let projects: Vec<Project> = tokio::task::spawn_blocking(move || {
            projects::table
                .load::<Project>(&mut conn)
        }).await??;
        
        Ok(projects)
    }
    
    async fn delete(&self, id: Uuid) -> Result<()> {
        let mut conn = self.pool.get()?;
        
        let count = tokio::task::spawn_blocking(move || {
            diesel::delete(projects::table.find(id))
                .execute(&mut conn)
        }).await??;
        
        if count == 0 {
            return Err(anyhow!("Project not found with ID: {}", id));
        }
        
        Ok(())
    }
}
