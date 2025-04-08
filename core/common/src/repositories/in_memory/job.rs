use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use async_trait::async_trait;
use uuid::Uuid;
use chrono::Utc;

use crate::errors::Error;
use crate::models::job::{Job, JobStatus, NewJob, PriorityLevel};
use crate::repositories::JobRepository;
use crate::repositories::job::{JobFilter, JobSortOrder, Pagination};
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
            priority: PriorityLevel::Medium, // Default value since not stored in DB
            input_data: serde_json::Value::Null, // Default value since not stored in DB
            output_data: None,
            error: None,
            estimated_cost_cents: new_job.cost_cents, // Use cost as estimate
            cost_cents: new_job.cost_cents,
            created_at: Some(chrono::Utc::now().naive_utc()),
            updated_at: None,
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
        job.updated_at = Some(chrono::Utc::now().naive_utc()); // Use updated_at instead of started_at
        
        Ok(job.clone())
    }
    
    async fn set_completed(
        &self, 
        id: Uuid, 
        success: bool, 
        output: Option<serde_json::Value>, 
        error: Option<String>, 
        cost_cents: i32
    ) -> Result<Job> {
        let mut jobs = self.jobs.lock().map_err(|_| Error::Other(anyhow::anyhow!("Lock error")))?;
        
        let job = jobs.get_mut(&id)
            .ok_or_else(|| Error::NotFound(format!("Job not found: {}", id)))?;
            
        job.status = if success { JobStatus::Succeeded } else { JobStatus::Failed };
        job.output_data = output;
        job.error = error;
        job.cost_cents = cost_cents;
        job.updated_at = Some(chrono::Utc::now().naive_utc());
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
    
    async fn query_jobs(&self, filter: JobFilter, sort: Option<JobSortOrder>, pagination: Option<Pagination>) -> Result<(Vec<Job>, u64)> {
        let jobs = self.jobs.lock().map_err(|_| Error::Other(anyhow::anyhow!("Lock error")))?;
        
        // Start with all jobs and apply filters
        let mut filtered_jobs: Vec<Job> = jobs.values().cloned().collect();
        
        // Apply filters
        if let Some(customer_id) = filter.customer_id {
            filtered_jobs.retain(|job| job.customer_id == customer_id);
        }
        
        if let Some(job_type_id) = filter.job_type_id {
            filtered_jobs.retain(|job| job.job_type_id == job_type_id);
        }
        
        if let Some(status) = filter.status {
            filtered_jobs.retain(|job| job.status == status);
        }
        
        if let Some(priority) = filter.priority {
            filtered_jobs.retain(|job| job.priority == priority);
        }
        
        if let Some(created_after) = filter.created_after {
            filtered_jobs.retain(|job| job.created_at.map_or(false, |created_at| created_at >= created_after));
        }
        
        if let Some(created_before) = filter.created_before {
            filtered_jobs.retain(|job| job.created_at.map_or(false, |created_at| created_at <= created_before));
        }
        
        if filter.completed_only {
            filtered_jobs.retain(|job| job.completed_at.is_some());
        }
        
        if filter.failed_only {
            filtered_jobs.retain(|job| job.status == JobStatus::Failed);
        }
        
        // Get total count before pagination
        let total_count = filtered_jobs.len() as u64;
        
        // Apply sorting
        match sort {
            Some(JobSortOrder::CreatedDesc) => {
                filtered_jobs.sort_by(|a, b| b.created_at.cmp(&a.created_at));
            },
            Some(JobSortOrder::CreatedAsc) => {
                filtered_jobs.sort_by(|a, b| a.created_at.cmp(&b.created_at));
            },
            Some(JobSortOrder::PriorityDesc) => {
                filtered_jobs.sort_by(|a, b| b.priority.cmp(&a.priority));
            },
            Some(JobSortOrder::PriorityAsc) => {
                filtered_jobs.sort_by(|a, b| a.priority.cmp(&b.priority));
            },
            None => {
                // Default sort by created_at descending
                filtered_jobs.sort_by(|a, b| b.created_at.cmp(&a.created_at));
            }
        }
        
        // Apply pagination
        if let Some(pagination) = pagination {
            let start = (pagination.page * pagination.per_page) as usize;
            let end = start + pagination.per_page as usize;
            filtered_jobs = filtered_jobs.into_iter().skip(start).take(end - start).collect();
        }
        
        Ok((filtered_jobs, total_count))
    }
    
    async fn get_job_stats_by_status(&self) -> Result<Vec<(String, i64)>> {
        let jobs = self.jobs.lock().map_err(|_| Error::Other(anyhow::anyhow!("Lock error")))?;
        
        // Count jobs by status
        let mut stats: HashMap<String, i64> = HashMap::new();
        
        for job in jobs.values() {
            let status_str = job.status.as_str().to_string();
            *stats.entry(status_str).or_insert(0) += 1;
        }
        
        // Convert HashMap to Vec<(String, i64)>
        Ok(stats.into_iter().collect())
    }
    
    async fn get_job_stats_by_customer(&self) -> Result<Vec<(Uuid, i64)>> {
        let jobs = self.jobs.lock().map_err(|_| Error::Other(anyhow::anyhow!("Lock error")))?;
        
        // Count jobs by customer
        let mut stats: HashMap<Uuid, i64> = HashMap::new();
        
        for job in jobs.values() {
            *stats.entry(job.customer_id).or_insert(0) += 1;
        }
        
        // Convert HashMap to Vec<(Uuid, i64)>
        Ok(stats.into_iter().collect())
    }
    
    async fn get_cost_statistics(&self) -> Result<(i64, i64)> {
        let jobs = self.jobs.lock().map_err(|_| Error::Other(anyhow::anyhow!("Lock error")))?;
        
        // Initialize counters
        let mut total_cost: i64 = 0;
        let mut completed_count: i64 = 0;
        
        // Calculate sum of costs for completed jobs
        for job in jobs.values() {
            if job.status == JobStatus::Succeeded && job.completed_at.is_some() {
                total_cost += job.cost_cents as i64;
                completed_count += 1;
            }
        }
        
        Ok((total_cost, completed_count))
    }
    
    async fn find_stalled_jobs(&self, running_threshold_minutes: i32) -> Result<Vec<Job>> {
        let jobs = self.jobs.lock().map_err(|_| Error::Other(anyhow::anyhow!("Lock error")))?;
        
        // Get current time
        let now = Utc::now().naive_utc();
        
        // Filter jobs that are in running state for too long
        let stalled_jobs = jobs.values()
            .filter(|job| {
                job.status == JobStatus::Running && 
                job.updated_at.map_or(false, |updated_at| {
                    let duration = now.signed_duration_since(updated_at);
                    duration.num_minutes() >= running_threshold_minutes.into()
                })
            })
            .cloned()
            .collect();
        
        Ok(stalled_jobs)
    }
    
    async fn bulk_update_status(&self, ids: Vec<Uuid>, status: JobStatus) -> Result<usize> {
        if ids.is_empty() {
            return Ok(0);
        }
        
        let mut jobs = self.jobs.lock().map_err(|_| Error::Other(anyhow::anyhow!("Lock error")))?;
        
        let now = Utc::now().naive_utc();
        let mut updated_count = 0;
        
        // Update each job that matches an ID in the list
        for id in ids {
            if let Some(job) = jobs.get_mut(&id) {
                job.status = status.clone();
                job.updated_at = Some(now);
                updated_count += 1;
            }
        }
        
        Ok(updated_count)
    }
}
