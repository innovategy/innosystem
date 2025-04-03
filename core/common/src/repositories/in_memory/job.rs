use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use async_trait::async_trait;
use uuid::Uuid;

use crate::errors::Error;
use crate::models::job::{Job, JobStatus, NewJob, PriorityLevel};
use crate::repositories::JobRepository;
use crate::Result;

/// In-memory implementation of JobRepository for Phase 1
pub struct InMemoryJobRepository {
    jobs: Arc<Mutex<HashMap<Uuid, Job>>>,
}

impl InMemoryJobRepository {
    pub fn new() -> Self {
        Self {
            jobs: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl JobRepository for InMemoryJobRepository {
    async fn create(&self, new_job: NewJob) -> Result<Job> {
        let job = Job {
            id: new_job.id,
            customer_id: new_job.customer_id,
            job_type_id: new_job.job_type_id,
            status: JobStatus::from_str(&new_job.status).ok_or_else(|| Error::InvalidInput(format!("Invalid job status: {}", new_job.status)))?,
            priority: PriorityLevel::from_i32(new_job.priority),
            input_data: new_job.input_data,
            output_data: None,
            error: None,
            estimated_cost_cents: new_job.estimated_cost_cents,
            cost_cents: None,
            created_at: Some(chrono::Utc::now().naive_utc()),
            started_at: None,
            completed_at: None,
        };
        
        let mut jobs = self.jobs.lock().map_err(|_| Error::Other(anyhow::anyhow!("Lock error")))?;
        jobs.insert(job.id, job.clone());
        
        Ok(job)
    }
    
    async fn find_by_id(&self, id: Uuid) -> Result<Job> {
        let jobs = self.jobs.lock().map_err(|_| Error::Other(anyhow::anyhow!("Lock error")))?;
        jobs.get(&id)
            .cloned()
            .ok_or_else(|| Error::NotFound(format!("Job not found: {}", id)))
    }
    
    async fn update_status(&self, id: Uuid, status: JobStatus) -> Result<Job> {
        let mut jobs = self.jobs.lock().map_err(|_| Error::Other(anyhow::anyhow!("Lock error")))?;
        
        let job = jobs.get_mut(&id)
            .ok_or_else(|| Error::NotFound(format!("Job not found: {}", id)))?;
            
        job.status = status;
        
        Ok(job.clone())
    }
    
    async fn set_started(&self, id: Uuid) -> Result<Job> {
        let mut jobs = self.jobs.lock().map_err(|_| Error::Other(anyhow::anyhow!("Lock error")))?;
        
        let job = jobs.get_mut(&id)
            .ok_or_else(|| Error::NotFound(format!("Job not found: {}", id)))?;
            
        job.status = JobStatus::Running;
        job.started_at = Some(chrono::Utc::now().naive_utc());
        
        Ok(job.clone())
    }
    
    async fn set_completed(
        &self, 
        id: Uuid, 
        success: bool, 
        output: Option<serde_json::Value>, 
        error: Option<String>, 
        cost_cents: Option<i32>
    ) -> Result<Job> {
        let mut jobs = self.jobs.lock().map_err(|_| Error::Other(anyhow::anyhow!("Lock error")))?;
        
        let job = jobs.get_mut(&id)
            .ok_or_else(|| Error::NotFound(format!("Job not found: {}", id)))?;
            
        job.status = if success { JobStatus::Succeeded } else { JobStatus::Failed };
        job.output_data = output;
        job.error = error;
        job.cost_cents = cost_cents;
        job.completed_at = Some(chrono::Utc::now().naive_utc());
        
        Ok(job.clone())
    }
    
    async fn find_by_customer_id(&self, customer_id: Uuid) -> Result<Vec<Job>> {
        let jobs = self.jobs.lock().map_err(|_| Error::Other(anyhow::anyhow!("Lock error")))?;
        
        Ok(jobs.values()
            .filter(|job| job.customer_id == customer_id)
            .cloned()
            .collect())
    }
    
    async fn find_by_status(&self, status: JobStatus) -> Result<Vec<Job>> {
        let jobs = self.jobs.lock().map_err(|_| Error::Other(anyhow::anyhow!("Lock error")))?;
        
        Ok(jobs.values()
            .filter(|job| job.status == status)
            .cloned()
            .collect())
    }
    
    async fn find_pending_jobs(&self, limit: i32) -> Result<Vec<Job>> {
        let jobs = self.jobs.lock().map_err(|_| Error::Other(anyhow::anyhow!("Lock error")))?;
        
        Ok(jobs.values()
            .filter(|job| job.status == JobStatus::Pending)
            .cloned()
            .take(limit as usize)
            .collect())
    }
}
