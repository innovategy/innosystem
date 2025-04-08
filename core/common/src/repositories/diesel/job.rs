use async_trait::async_trait;
use diesel::prelude::*;
use uuid::Uuid;

use crate::database::{get_connection, PgPool};
use crate::diesel_schema::jobs;
use crate::errors::Error;
use crate::models::job::{Job, JobDb, JobStatus, NewJob};
use crate::repositories::JobRepository;
use crate::Result;

/// Diesel-backed implementation of JobRepository
pub struct DieselJobRepository {
    pool: PgPool,
}

impl DieselJobRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl JobRepository for DieselJobRepository {
    async fn create(&self, new_job: NewJob) -> Result<Job> {
        let mut conn = get_connection(&self.pool)?;
        
        // Execute the insert and return the new record
        let job_db = diesel::insert_into(jobs::table)
            .values(&new_job)
            .returning(JobDb::as_select())
            .get_result(&mut conn)
            .map_err(|e| Error::Database(e))?;
            
        // Convert to application model
        Ok(Job::from(job_db))
    }
    
    async fn find_by_id(&self, id: Uuid) -> Result<Job> {
        let mut conn = get_connection(&self.pool)?;
        
        let job_db = jobs::table
            .find(id)
            .select(JobDb::as_select())
            .first(&mut conn)
            .map_err(|e| match e {
                diesel::result::Error::NotFound => Error::NotFound(format!("Job not found: {}", id)),
                e => Error::Database(e),
            })?;
            
        // Convert to application model
        Ok(Job::from(job_db))
    }
    
    async fn update_status(&self, id: Uuid, status: JobStatus) -> Result<Job> {
        let mut conn = get_connection(&self.pool)?;
        
        // First check if the entity exists
        let _ = jobs::table
            .find(id)
            .select(JobDb::as_select())
            .first(&mut conn)
            .map_err(|e| match e {
                diesel::result::Error::NotFound => Error::NotFound(format!("Job not found: {}", id)),
                e => Error::Database(e),
            })?;
        
        // Update the status
        let job_db = diesel::update(jobs::table)
            .filter(jobs::id.eq(id))
            .set(jobs::status.eq(status.as_str()))
            .returning(JobDb::as_select())
            .get_result(&mut conn)
            .map_err(|e| Error::Database(e))?;
            
        // Convert to application model
        Ok(Job::from(job_db))
    }
    
    async fn set_started(&self, id: Uuid) -> Result<Job> {
        let mut conn = get_connection(&self.pool)?;
        
        // Update the status to running and set the updated_at timestamp
        let job_db = diesel::update(jobs::table)
            .filter(jobs::id.eq(id))
            .set((
                jobs::status.eq(JobStatus::Running.as_str()),
                jobs::updated_at.eq(diesel::dsl::now),
            ))
            .returning(JobDb::as_select())
            .get_result(&mut conn)
            .map_err(|e| match e {
                diesel::result::Error::NotFound => Error::NotFound(format!("Job not found: {}", id)),
                e => Error::Database(e),
            })?;
            
        // Convert to application model
        Ok(Job::from(job_db))
    }
    
    async fn set_completed(
        &self, 
        id: Uuid, 
        success: bool, 
        output: Option<serde_json::Value>, 
        error: Option<String>, 
        cost_cents: i32
    ) -> Result<Job> {
        let mut conn = get_connection(&self.pool)?;
        
        let status = if success { JobStatus::Succeeded } else { JobStatus::Failed };
        
        // Use the provided cost directly since it's now a required parameter
        
        // Update the job with completion data
        let job_db = diesel::update(jobs::table)
            .filter(jobs::id.eq(id))
            .set((
                jobs::status.eq(status.as_str()),
                jobs::cost_cents.eq(cost_cents),
                jobs::completed_at.eq(diesel::dsl::now),
                jobs::updated_at.eq(diesel::dsl::now),
            ))
            .returning(JobDb::as_select())
            .get_result(&mut conn)
            .map_err(|e| match e {
                diesel::result::Error::NotFound => Error::NotFound(format!("Job not found: {}", id)),
                e => Error::Database(e),
            })?;
        
        // Create a Job from JobDb and add the non-DB fields
        let mut job = Job::from(job_db);
        
        // Set the fields that aren't in the database
        job.output_data = output;
        job.error = error;
        
        Ok(job)
    }
    
    async fn find_by_customer_id(&self, customer_id: Uuid) -> Result<Vec<Job>> {
        let mut conn = get_connection(&self.pool)?;
        
        let jobs_db = jobs::table
            .filter(jobs::customer_id.eq(customer_id))
            .select(JobDb::as_select())
            .load(&mut conn)
            .map_err(|e| Error::Database(e))?;
            
        // Convert all database models to application models
        let jobs = jobs_db.into_iter().map(Job::from).collect();
        Ok(jobs)
    }
    
    async fn find_by_status(&self, status: JobStatus) -> Result<Vec<Job>> {
        let mut conn = get_connection(&self.pool)?;
        
        let jobs_db = jobs::table
            .filter(jobs::status.eq(status.as_str()))
            .select(JobDb::as_select())
            .load(&mut conn)
            .map_err(|e| Error::Database(e))?;
            
        // Convert all database models to application models
        let jobs = jobs_db.into_iter().map(Job::from).collect();
        Ok(jobs)
    }
    
    async fn find_pending_jobs(&self, limit: i32) -> Result<Vec<Job>> {
        let mut conn = get_connection(&self.pool)?;
        
        let jobs_db = jobs::table
            .filter(jobs::status.eq(JobStatus::Pending.as_str()))
            .select(JobDb::as_select())
            .limit(limit.into())
            .load(&mut conn)
            .map_err(|e| Error::Database(e))?;
            
        // Convert all database models to application models
        let jobs = jobs_db.into_iter().map(Job::from).collect();
        Ok(jobs)
    }
}
