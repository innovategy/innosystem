use axum::{extract::{Path, State}, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tracing::{info, error, warn};

use innosystem_common::models::job::{NewJob, PriorityLevel, JobStatus};

use crate::state::AppState;

/// Request data for creating a new job
#[derive(Debug, Deserialize)]
pub struct CreateJobRequest {
    /// Customer ID
    pub customer_id: Uuid,
    /// Job type ID
    pub job_type_id: Uuid,
    /// Priority level (optional, defaults to 1)
    #[serde(default = "default_priority")]
    pub priority: i32,
    /// Input data for the job
    pub input_data: serde_json::Value,
}

/// Default priority function
fn default_priority() -> i32 {
    1
}

/// Response data for job operations
#[derive(Debug, Serialize)]
pub struct JobResponse {
    /// Job ID
    pub id: Uuid,
    /// Customer ID
    pub customer_id: Uuid,
    /// Job type ID
    pub job_type_id: Uuid,
    /// Current status
    pub status: String,
    /// Priority level
    pub priority: i32,
    /// Input data
    pub input_data: serde_json::Value,
    /// Output data (if completed)
    pub output_data: Option<serde_json::Value>,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Estimated cost in cents
    pub estimated_cost_cents: i32,
    /// Actual cost in cents (if completed)
    pub cost_cents: Option<i32>,
    /// Creation timestamp
    pub created_at: Option<String>,
    /// Start timestamp
    pub started_at: Option<String>,
    /// Completion timestamp
    pub completed_at: Option<String>,
}

/// Request to calculate job cost
#[derive(Debug, Deserialize)]
pub struct CalculateJobCostRequest {
    /// Job ID to calculate cost for
    pub job_id: Uuid,
}

/// Response with job cost calculation
#[derive(Debug, Serialize)]
pub struct JobCostResponse {
    /// Job ID
    pub job_id: Uuid,
    /// Estimated cost in cents
    pub estimated_cost_cents: i32,
    /// Calculated actual cost in cents
    pub calculated_cost_cents: i32,
}

/// Request to complete a job
#[derive(Debug, Deserialize)]
pub struct CompleteJobRequest {
    /// Job ID to mark as completed
    pub job_id: Uuid,
    /// Whether the job was successful
    pub success: bool,
    /// Output data from the job
    pub output_data: Option<serde_json::Value>,
    /// Error message if job failed
    pub error: Option<String>,
}

/// Create a new job
#[allow(dead_code)]
pub async fn create_job(
    State(state): State<AppState>,
    Json(payload): Json<CreateJobRequest>,
) -> Result<(StatusCode, Json<JobResponse>), StatusCode> {
    // Convert the priority from i32 to PriorityLevel
    let priority = PriorityLevel::from_i32(payload.priority);
    
    // First create a full Job with all application-level fields
    let job = innosystem_common::models::job::Job::new(
        payload.customer_id,
        payload.job_type_id,
        payload.input_data.clone(),
        priority,
        1000, // $10.00 default estimated cost for now
    );
    
    // Convert to NewJob for repository storage
    let new_job = NewJob::from(job.clone());
    
    // Save the job to the repository
    let created_job = state.job_repo.create(new_job)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create job: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    // Push the job to the queue for processing
    // Clone priority to avoid ownership issues
    let job_priority = created_job.priority.clone();
    match state.job_queue.push_job(created_job.id, job_priority).await {
        Ok(_) => tracing::info!("Job {} added to queue for processing", created_job.id),
        Err(e) => {
            tracing::error!("Failed to queue job {}: {}", created_job.id, e);
            // We don't fail the request here - the job is still created, just not queued
            // The runner will periodically scan for unqueued jobs
        }
    }
    
    // Convert the timestamps to RFC3339 strings if they exist
    let created_at = created_job.created_at.map(|dt| dt.and_utc().to_rfc3339());
    let updated_at = created_job.updated_at.map(|dt| dt.and_utc().to_rfc3339()); // Changed to updated_at
    let completed_at = created_job.completed_at.map(|dt| dt.and_utc().to_rfc3339());
    
    // Create the response
    let response = JobResponse {
        id: created_job.id,
        customer_id: created_job.customer_id,
        job_type_id: created_job.job_type_id,
        status: created_job.status.as_str().to_string(),
        priority: created_job.priority.as_i32(), // This should work now
        input_data: created_job.input_data,
        output_data: created_job.output_data,
        error: created_job.error,
        estimated_cost_cents: created_job.estimated_cost_cents,
        cost_cents: Some(created_job.cost_cents), // Now cost_cents is i32, not Option<i32>
        created_at,
        started_at: updated_at, // Use updated_at instead of started_at
        completed_at,
    };
    
    tracing::info!("Created new job with ID: {}", created_job.id);
    Ok((StatusCode::CREATED, Json(response)))
}

/// Get a job by ID
#[allow(dead_code)]
pub async fn get_job(
    State(state): State<AppState>,
    Path(job_id_str): Path<String>,
) -> Result<Json<JobResponse>, StatusCode> {
    // Try to parse the job_id as a UUID
    let job_id = match Uuid::parse_str(&job_id_str) {
        Ok(id) => id,
        Err(_) => {
            tracing::error!("Invalid job ID format: {}", job_id_str);
            return Err(StatusCode::BAD_REQUEST);
        }
    };
    
    // Fetch the job from the repository
    let job = state.job_repo.find_by_id(job_id)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch job: {}", e);
            // If job not found, return 404
            if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;
    
    // Convert the timestamps to RFC3339 strings if they exist
    let created_at = job.created_at.map(|dt| dt.and_utc().to_rfc3339());
    let updated_at = job.updated_at.map(|dt| dt.and_utc().to_rfc3339()); // Changed to updated_at
    let completed_at = job.completed_at.map(|dt| dt.and_utc().to_rfc3339());
    
    // Create the response
    let response = JobResponse {
        id: job.id,
        customer_id: job.customer_id,
        job_type_id: job.job_type_id,
        status: job.status.as_str().to_string(),
        priority: job.priority.as_i32(),
        input_data: job.input_data,
        output_data: job.output_data,
        error: job.error,
        estimated_cost_cents: job.estimated_cost_cents,
        cost_cents: Some(job.cost_cents), // Now cost_cents is i32, not Option<i32>
        created_at,
        started_at: updated_at, // Use updated_at instead of started_at
        completed_at,
    };
    
    tracing::info!("Retrieved job with ID: {}", job_id);
    Ok(Json(response))
}

/// Get all jobs
#[allow(dead_code)]
pub async fn get_all_jobs(
    State(state): State<AppState>,
) -> Result<Json<Vec<JobResponse>>, StatusCode> {
    // Create default filter and pagination
    let filter = innosystem_common::repositories::job::JobFilter::default();
    let sort = Some(innosystem_common::repositories::job::JobSortOrder::CreatedDesc);
    let pagination = None; // Get all jobs without pagination
    
    // Fetch all jobs from the repository using query_jobs
    let (jobs, _total_count) = state.job_repo.query_jobs(filter, sort, pagination).await
        .map_err(|e| {
            tracing::error!("Failed to fetch jobs: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    // Convert the jobs to the response format
    let job_responses = jobs.into_iter().map(|job| {
        // Convert the timestamps to RFC3339 strings if they exist
        let created_at = job.created_at.map(|dt| dt.and_utc().to_rfc3339());
        let updated_at = job.updated_at.map(|dt| dt.and_utc().to_rfc3339());
        let completed_at = job.completed_at.map(|dt| dt.and_utc().to_rfc3339());
        
        JobResponse {
            id: job.id,
            customer_id: job.customer_id,
            job_type_id: job.job_type_id,
            status: job.status.as_str().to_string(),
            priority: job.priority.as_i32(),
            input_data: job.input_data,
            output_data: job.output_data,
            error: job.error,
            estimated_cost_cents: job.estimated_cost_cents,
            cost_cents: Some(job.cost_cents),
            created_at,
            started_at: updated_at,
            completed_at,
        }
    }).collect();
    
    tracing::info!("Retrieved all jobs from database");
    Ok(Json(job_responses))
}

/// Calculate the cost of a job
#[allow(dead_code)]
pub async fn calculate_job_cost(
    State(state): State<AppState>,
    Json(payload): Json<CalculateJobCostRequest>,
) -> Result<Json<JobCostResponse>, StatusCode> {
    // Fetch the job to ensure it exists
    let job = state.job_repo.find_by_id(payload.job_id)
        .await
        .map_err(|e| {
            error!("Failed to fetch job: {}", e);
            if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;
    
    // Calculate the cost using the billing service
    let calculated_cost = state.billing_service.calculate_job_cost(payload.job_id)
        .await
        .map_err(|e| {
            error!("Failed to calculate job cost: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    // Create the response
    let response = JobCostResponse {
        job_id: job.id,
        estimated_cost_cents: job.estimated_cost_cents,
        calculated_cost_cents: calculated_cost,
    };
    
    info!("Calculated cost for job {}: {} cents", job.id, calculated_cost);
    Ok(Json(response))
}

/// Complete a job and process billing
#[allow(dead_code)]
pub async fn complete_job(
    State(state): State<AppState>,
    Json(payload): Json<CompleteJobRequest>,
) -> Result<Json<JobResponse>, StatusCode> {
    // Fetch the job to ensure it exists and check its current status
    let job = state.job_repo.find_by_id(payload.job_id)
        .await
        .map_err(|e| {
            error!("Failed to fetch job for completion: {}", e);
            if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;
    
    // Check if job can be completed (must be in Running or Pending status)
    if job.status != JobStatus::Running && job.status != JobStatus::Pending {
        error!("Cannot complete job {} with status {}", job.id, job.status.as_str());
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // Process billing for the job
    if let Err(e) = state.billing_service.process_job_billing(payload.job_id, payload.success).await {
        error!("Failed to process billing for job {}: {}", payload.job_id, e);
        // Continue with job completion even if billing fails, but log the error
        warn!("Job {} will be marked as completed but billing failed", payload.job_id);
    }
    
    // Update the job status and other fields
    let updated_job = state.job_repo.set_completed(
        payload.job_id,
        payload.success,
        payload.output_data.clone(),
        payload.error.clone(),
        job.cost_cents, // Pass current cost_cents as this was updated by the billing service
    )
    .await
    .map_err(|e| {
        error!("Failed to update job status: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    // Convert the timestamps to RFC3339 strings if they exist
    let created_at = updated_job.created_at.map(|dt| dt.and_utc().to_rfc3339());
    let updated_at = updated_job.updated_at.map(|dt| dt.and_utc().to_rfc3339());
    let completed_at = updated_job.completed_at.map(|dt| dt.and_utc().to_rfc3339());
    
    // Create the response
    let response = JobResponse {
        id: updated_job.id,
        customer_id: updated_job.customer_id,
        job_type_id: updated_job.job_type_id,
        status: updated_job.status.as_str().to_string(),
        priority: updated_job.priority.as_i32(),
        input_data: updated_job.input_data,
        output_data: updated_job.output_data,
        error: updated_job.error,
        estimated_cost_cents: updated_job.estimated_cost_cents,
        cost_cents: Some(updated_job.cost_cents),
        created_at,
        started_at: updated_at,
        completed_at,
    };
    
    info!("Job {} completed with status: {}", payload.job_id, if payload.success { "SUCCESS" } else { "FAILURE" });
    Ok(Json(response))
}
