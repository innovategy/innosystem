use async_trait::async_trait;
use chrono::Utc;
use diesel::prelude::*;
use diesel::dsl::{count_star, sum};
// No need to import private BoxedSelectStatement type
use uuid::Uuid;

use crate::database::{get_connection, PgPool, Transaction};
use crate::diesel_schema::jobs;
use crate::errors::Error;
use crate::models::job::{Job, JobDb, JobStatus, NewJob};
use crate::repositories::JobRepository;
use crate::repositories::job::{JobFilter, JobSortOrder, Pagination};
use crate::Result;

/// Diesel-backed implementation of JobRepository
pub struct DieselJobRepository {
    pool: PgPool,
}

impl DieselJobRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    
    // Helper function to apply filters to a query
    fn apply_filters<'a>(&self, mut query: jobs::BoxedQuery<'a, diesel::pg::Pg>, filter: &JobFilter) -> jobs::BoxedQuery<'a, diesel::pg::Pg> {
        
        // Apply customer_id filter if provided
        if let Some(customer_id) = filter.customer_id {
            query = query.filter(jobs::customer_id.eq(customer_id));
        }
        
        // Apply job_type_id filter if provided
        if let Some(job_type_id) = filter.job_type_id {
            query = query.filter(jobs::job_type_id.eq(job_type_id));
        }
        
        // Apply status filter if provided
        if let Some(status) = &filter.status {
            query = query.filter(jobs::status.eq(status.as_str()));
        }
        
        // Filter by created_after if provided
        if let Some(created_after) = filter.created_after {
            query = query.filter(jobs::created_at.ge(created_after));
        }
        
        // Filter by created_before if provided
        if let Some(created_before) = filter.created_before {
            query = query.filter(jobs::created_at.le(created_before));
        }
        
        // Filter by completed only
        if filter.completed_only {
            query = query.filter(jobs::completed_at.is_not_null());
        }
        
        // Filter by failed only
        if filter.failed_only {
            query = query.filter(jobs::status.eq(JobStatus::Failed.as_str()));
        }
        
        query
    }
    
    // Helper function to apply sorting to a query
    fn apply_sorting<'a>(&self, query: jobs::BoxedQuery<'a, diesel::pg::Pg>, sort: &Option<JobSortOrder>) -> jobs::BoxedQuery<'a, diesel::pg::Pg> {
        match sort {
            Some(JobSortOrder::CreatedDesc) => query.order(jobs::created_at.desc()),
            Some(JobSortOrder::CreatedAsc) => query.order(jobs::created_at.asc()),
            // Note: For PriorityDesc/Asc we'd ideally use the actual priority field,
            // but since it's not stored in database, we're using other fields as proxy
            // This is a limitation of our current model separation
            Some(JobSortOrder::PriorityDesc) => query.order(jobs::id.desc()), // Using ID as a proxy for now
            Some(JobSortOrder::PriorityAsc) => query.order(jobs::id.asc()),   // Using ID as a proxy for now
            None => query.order(jobs::created_at.desc()), // Default sort
        }
    }
    
    // Helper function to apply pagination
    fn apply_pagination<'a>(&self, query: jobs::BoxedQuery<'a, diesel::pg::Pg>, pagination: &Option<Pagination>) -> jobs::BoxedQuery<'a, diesel::pg::Pg> {
        if let Some(pagination) = pagination {
            let offset = pagination.page * pagination.per_page;
            query.offset(offset.into()).limit(pagination.per_page.into())
        } else {
            query
        }
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
            .first::<JobDb>(&mut conn)
            .map_err(|e| match e {
                diesel::result::Error::NotFound => Error::NotFound(format!("Job not found: {}", id)),
                e => Error::Database(e),
            })?;
        
        // Update the status
        let job_db = diesel::update(jobs::table)
            .filter(jobs::id.eq(id))
            .set((
                jobs::status.eq(status.as_str()),
                jobs::updated_at.eq(diesel::dsl::now),
            ))
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
        // Use transaction to ensure atomicity of job completion
        self.pool.run_in_transaction(|conn| {
            let status = if success { JobStatus::Succeeded } else { JobStatus::Failed };
            
            // Use the provided cost directly since it's now a required parameter
            
            // Update the job with completion data within transaction
            let job_db = diesel::update(jobs::table)
                .filter(jobs::id.eq(id))
                .set((
                    jobs::status.eq(status.as_str()),
                    jobs::cost_cents.eq(cost_cents),
                    jobs::completed_at.eq(diesel::dsl::now),
                    jobs::updated_at.eq(diesel::dsl::now),
                ))
                .returning(JobDb::as_select())
                .get_result(conn)?;
            
            // Create a Job from JobDb and add the non-DB fields
            let mut job = Job::from(job_db);
            
            // Set the fields that aren't in the database
            job.output_data = output;
            job.error = error;
            
            Ok(job)
        })
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
            .order(jobs::created_at.asc())
            .limit(limit.into())
            .select(JobDb::as_select())
            .load(&mut conn)
            .map_err(|e| Error::Database(e))?;
            
        // Convert all database models to application models
        let jobs = jobs_db.into_iter().map(Job::from).collect();
        Ok(jobs)
    }
    
    async fn query_jobs(&self, filter: JobFilter, sort: Option<JobSortOrder>, pagination: Option<Pagination>) -> Result<(Vec<Job>, u64)> {
        let mut conn = get_connection(&self.pool)?;
        
        // First, let's create a count query with the same filters
        let count_query = jobs::table.into_boxed();
        let filtered_count_query = self.apply_filters(count_query, &filter);
        
        // Execute the count query
        let total: i64 = filtered_count_query
            .count()
            .get_result(&mut conn)
            .map_err(|e| Error::Database(e))?;
        
        // Then let's create our data query
        let query = jobs::table.into_boxed();
        
        // Apply filters
        let filtered_query = self.apply_filters(query, &filter);
        
        // Apply sorting
        let sorted_query = self.apply_sorting(filtered_query, &sort);
        
        // Apply pagination
        let final_query = self.apply_pagination(sorted_query, &pagination);
        
        // Execute the query
        let jobs_db = final_query
            .select(JobDb::as_select())
            .load(&mut conn)
            .map_err(|e| Error::Database(e))?;
            
        // Convert database models to application models
        let jobs = jobs_db.into_iter().map(Job::from).collect();
        
        Ok((jobs, total as u64))
    }
    
    async fn get_job_stats_by_status(&self) -> Result<Vec<(String, i64)>> {
        let mut conn = get_connection(&self.pool)?;
        
        // Group by status and count jobs
        let results = jobs::table
            .group_by(jobs::status)
            .select((jobs::status, count_star()))
            .load::<(String, i64)>(&mut conn)
            .map_err(|e| Error::Database(e))?;
        
        Ok(results)
    }
    
    async fn get_job_stats_by_customer(&self) -> Result<Vec<(Uuid, i64)>> {
        let mut conn = get_connection(&self.pool)?;
        
        // Group by customer_id and count jobs
        let results = jobs::table
            .group_by(jobs::customer_id)
            .select((jobs::customer_id, count_star()))
            .load::<(Uuid, i64)>(&mut conn)
            .map_err(|e| Error::Database(e))?;
        
        Ok(results)
    }
    
    async fn get_cost_statistics(&self) -> Result<(i64, i64)> {
        let mut conn = get_connection(&self.pool)?;
        
        // Calculate sum of estimated cost and actual cost for completed jobs
        // Define the filter criteria for completed jobs
        let completed_filter = jobs::completed_at.is_not_null().and(jobs::status.eq(JobStatus::Succeeded.as_str()));
        
        // Query for sum - creating a separate query
        let total_cost: Option<i64> = jobs::table
            .filter(completed_filter.clone())
            .select(sum(jobs::cost_cents))
            .first(&mut conn)
            .map_err(|e| Error::Database(e))?;
        
        // Handle case with no completed jobs
        let total_cost = total_cost.unwrap_or(0);
            
        // Query for count - a separate query without needing to clone BoxedQuery
        let completed_count: i64 = jobs::table
            .filter(completed_filter)
            .count()
            .get_result(&mut conn)
            .map_err(|e| Error::Database(e))?;
        
        Ok((total_cost, completed_count))
    }
    
    async fn find_stalled_jobs(&self, running_threshold_minutes: i32) -> Result<Vec<Job>> {
        let mut conn = get_connection(&self.pool)?;
        
        // Define stalled jobs as those that have been in 'Running' state for longer than the threshold
        // For stalled jobs, we need to find jobs that have been running for too long
        // First we'll get all running jobs, then filter based on the running_threshold_minutes
        let running_jobs = jobs::table
            .filter(jobs::status.eq(JobStatus::Running.as_str()))
            .into_boxed();
        
        // Since we can't directly use interval arithmetic in a safe way with Diesel,
        // we'll fetch all running jobs and filter in Rust
        let jobs_db = running_jobs
            .select(JobDb::as_select())
            .load(&mut conn)
            .map_err(|e| Error::Database(e))?;
            
        // Filter jobs in Rust based on the threshold
        let now = Utc::now().naive_utc();
        let jobs_db: Vec<JobDb> = jobs_db
            .into_iter()
            .filter(|job| {
                if let Some(updated_at) = job.updated_at {
                    let duration = now.signed_duration_since(updated_at);
                    duration.num_minutes() >= running_threshold_minutes.into()
                } else {
                    false
                }
            })
            .collect();
        
        // Convert to application models
        let jobs = jobs_db.into_iter().map(Job::from).collect();
        
        Ok(jobs)
    }
    
    async fn bulk_update_status(&self, ids: Vec<Uuid>, status: JobStatus) -> Result<usize> {
        if ids.is_empty() {
            return Ok(0);
        }
        
        // Use transaction to ensure atomicity
        self.pool.run_in_transaction(|conn| {
            // Update all jobs with the given IDs to the new status
            let updated_count = diesel::update(jobs::table)
                .filter(jobs::id.eq_any(ids))
                .set((
                    jobs::status.eq(status.as_str()),
                    jobs::updated_at.eq(diesel::dsl::now),
                ))
                .execute(conn)?;
            
            Ok(updated_count)
        })
    }
}
