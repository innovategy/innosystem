use axum::{extract::{Path, State}, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use innosystem_common::models::job::{NewJob, PriorityLevel};

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
    State(_state): State<AppState>,
) -> Result<Json<Vec<JobResponse>>, StatusCode> {
    // In a real implementation, we would fetch jobs from the database
    // For now, we'll just return a mock list of jobs
    
    let now = chrono::Utc::now().to_rfc3339();
    
    // Create two mock jobs
    let jobs = vec![
        JobResponse {
            id: Uuid::new_v4(),
            customer_id: Uuid::new_v4(),
            job_type_id: Uuid::new_v4(),
            status: "Pending".to_string(),
            priority: 1,
            input_data: serde_json::json!({"data": "example"}),
            output_data: None,
            error: None,
            estimated_cost_cents: 1000,
            cost_cents: None,
            created_at: Some(now.clone()),
            started_at: None,
            completed_at: None,
        },
        JobResponse {
            id: Uuid::new_v4(),
            customer_id: Uuid::new_v4(),
            job_type_id: Uuid::new_v4(),
            status: "Completed".to_string(),
            priority: 2,
            input_data: serde_json::json!({"data": "another example"}),
            output_data: Some(serde_json::json!({"result": "success"})),
            error: None,
            estimated_cost_cents: 2000,
            cost_cents: Some(1950),
            created_at: Some(now.clone()),
            started_at: Some(now.clone()),
            completed_at: Some(now.clone()),
        },
    ];
    
    tracing::info!("Retrieved all jobs");
    Ok(Json(jobs))
}
