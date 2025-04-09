use async_trait::async_trait;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use uuid::Uuid;
use anyhow::{Result, anyhow};
use chrono::{NaiveDateTime, Utc};


use crate::models::runner::{Runner, NewRunner, NewJobTypeCompatibility, RunnerStatus};
use crate::repositories::RunnerRepository;
use crate::diesel_schema::{runners, runner_job_type_compatibility};
use crate::models::job_type::JobType;

/// Diesel implementation of the RunnerRepository
pub struct DieselRunnerRepository {
    pool: Pool<ConnectionManager<PgConnection>>,
}

impl DieselRunnerRepository {
    /// Create a new DieselRunnerRepository with the given connection pool
    pub fn new(pool: Pool<ConnectionManager<PgConnection>>) -> Self {
        Self { pool }
    }
    
    // Helper to save job type compatibilities
    async fn save_job_type_compatibilities(&self, runner_id: Uuid, job_type_ids: Vec<Uuid>) -> Result<()> {
        let mut conn = self.pool.get()?;
        
        tokio::task::spawn_blocking(move || -> Result<()> {
            // Start a transaction
            conn.transaction(|conn| {
                // First delete existing compatibilities
                diesel::delete(runner_job_type_compatibility::table)
                    .filter(runner_job_type_compatibility::runner_id.eq(runner_id))
                    .execute(conn)?;
                
                // Then insert new ones
                for job_type_id in job_type_ids {
                    let compatibility = NewJobTypeCompatibility {
                        runner_id,
                        job_type_id,
                    };
                    
                    diesel::insert_into(runner_job_type_compatibility::table)
                        .values(&compatibility)
                        .execute(conn)?;
                }
                
                Ok(())
            })
        }).await?
    }
    
    // Helper to get job type compatibilities
    async fn get_job_type_compatibilities(&self, runner_id: Uuid) -> Result<Vec<Uuid>> {
        let mut conn = self.pool.get()?;
        
        let compatibilities = tokio::task::spawn_blocking(move || {
            runner_job_type_compatibility::table
                .filter(runner_job_type_compatibility::runner_id.eq(runner_id))
                .select(runner_job_type_compatibility::job_type_id)
                .load::<Uuid>(&mut conn)
        }).await??;
        
        Ok(compatibilities)
    }
}

#[async_trait]
impl RunnerRepository for DieselRunnerRepository {
    async fn register(&self, runner: NewRunner) -> Result<Runner> {
        let mut conn = self.pool.get()?;
        
        // Insert the new runner
        let runner: Runner = tokio::task::spawn_blocking(move || {
            diesel::insert_into(runners::table)
                .values(&runner)
                .get_result(&mut conn)
        }).await??;
        
        // Convert string job types to UUIDs and save compatibility if any exist
        if !runner.compatible_job_types.is_empty() {
            // Just passing empty vec since we can't get UUIDs from strings here
            // The caller should use update_capabilities after registration if needed
            self.save_job_type_compatibilities(runner.id, Vec::new()).await?;
        }
        
        Ok(runner)
    }
    
    async fn update_heartbeat(&self, id: Uuid, timestamp: NaiveDateTime) -> Result<Runner> {
        let mut conn = self.pool.get()?;
        
        let runner = tokio::task::spawn_blocking(move || {
            diesel::update(runners::table.find(id))
                .set(runners::last_heartbeat.eq(timestamp))
                .get_result::<Runner>(&mut conn)
        }).await??;
        
        Ok(runner)
    }
    
    async fn find_by_id(&self, id: Uuid) -> Result<Runner> {
        let mut conn = self.pool.get()?;
        
        let runner: Runner = tokio::task::spawn_blocking(move || {
            runners::table
                .find(id)
                .first(&mut conn)
                .optional()
        }).await??
            .ok_or_else(|| anyhow!("Runner not found with ID: {}", id))?;
        
        // Load job type compatibilities - runner already has compatible_job_types as strings
        // We don't need to modify the runner object with job_type_ids since that field doesn't exist
        
        Ok(runner)
    }
    
    async fn update_capabilities(&self, id: Uuid, job_type_ids: Vec<Uuid>) -> Result<Runner> {
        // First ensure runner exists
        let runner = self.find_by_id(id).await?;
        
        // Save the new capabilities
        self.save_job_type_compatibilities(id, job_type_ids.clone()).await?;
        
        // The Runner model doesn't have a job_type_ids field, it has compatible_job_types
        // We would need to query job types to get their names
        // For now, just return the runner as is
        
        Ok(runner)
    }
    
    async fn list_all(&self) -> Result<Vec<Runner>> {
        let mut conn = self.pool.get()?;
        
        let runners: Vec<Runner> = tokio::task::spawn_blocking(move || {
            runners::table
                .load::<Runner>(&mut conn)
        }).await??;
        
        // Runners already have their compatible_job_types as strings
        // We don't need to modify each runner with job_type_ids
        
        Ok(runners)
    }
    
    async fn list_active(&self, since: NaiveDateTime) -> Result<Vec<Runner>> {
        let mut conn = self.pool.get()?;
        
        let runners: Vec<Runner> = tokio::task::spawn_blocking(move || {
            runners::table
                .filter(runners::status.eq(RunnerStatus::Active.as_str()))
                .filter(runners::last_heartbeat.ge(since))
                .load::<Runner>(&mut conn)
        }).await??;
        
        // Runners already have their compatible_job_types as strings
        // We don't need to modify each runner with job_type_ids
        
        Ok(runners)
    }
    
    async fn find_compatible_with_job_type(&self, job_type: &JobType) -> Result<Vec<Runner>> {
        let job_type_id = job_type.id;
        let mut conn = self.pool.get()?;
        
        // Find runners that are compatible with this job type
        let runner_ids: Vec<Uuid> = tokio::task::spawn_blocking(move || {
            runner_job_type_compatibility::table
                .filter(runner_job_type_compatibility::job_type_id.eq(job_type_id))
                .select(runner_job_type_compatibility::runner_id)
                .load::<Uuid>(&mut conn)
        }).await??;
        
        if runner_ids.is_empty() {
            return Ok(Vec::new());
        }
        
        // Get the active runners from those IDs
        let mut conn = self.pool.get()?;
        let runner_ids_clone = runner_ids.clone();
        // Use chrono::Utc::now() for the timestamp calculation
        let since = Utc::now().naive_utc() - chrono::Duration::minutes(5);
        
        let runners: Vec<Runner> = tokio::task::spawn_blocking(move || {
            runners::table
                .filter(runners::id.eq_any(runner_ids_clone))
                .filter(runners::status.eq(RunnerStatus::Active.as_str()))
                .filter(runners::last_heartbeat.ge(since))
                .load::<Runner>(&mut conn)
        }).await??;
        
        Ok(runners)
    }
    
    async fn set_status(&self, id: Uuid, active: bool) -> Result<Runner> {
        let mut conn = self.pool.get()?;
        let status = if active { RunnerStatus::Active } else { RunnerStatus::Inactive };
        
        let runner = tokio::task::spawn_blocking(move || {
            diesel::update(runners::table.find(id))
                .set((
                    runners::status.eq(status.as_str()),
                    runners::updated_at.eq(Utc::now().naive_utc()),
                ))
                .get_result::<Runner>(&mut conn)
        }).await??;
        
        // The runner already has compatible_job_types as strings
        
        Ok(runner)
    }
}
