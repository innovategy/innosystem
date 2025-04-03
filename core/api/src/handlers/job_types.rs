use axum::{extract::{Path, State}, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::state::AppState;

/// Request data for creating a new job type
#[derive(Debug, Deserialize)]
pub struct CreateJobTypeRequest {
    /// Job type name
    pub name: String,
    /// Job type description
    pub description: String,
    /// Processor type
    pub processor_type: String,
    /// Custom processing logic ID (if applicable)
    pub processing_logic_id: Option<Uuid>,
    /// Standard cost in cents
    pub standard_cost_cents: i32,
    /// Whether the job type is enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

/// Default enabled status
fn default_enabled() -> bool {
    true
}

/// Response data for job type operations
#[derive(Debug, Serialize)]
pub struct JobTypeResponse {
    /// Job type ID
    pub id: Uuid,
    /// Job type name
    pub name: String,
    /// Job type description
    pub description: String,
    /// Processor type
    pub processor_type: String,
    /// Custom processing logic ID (if applicable)
    pub processing_logic_id: Option<Uuid>,
    /// Standard cost in cents
    pub standard_cost_cents: i32,
    /// Whether the job type is enabled
    pub enabled: bool,
    /// Creation timestamp
    pub created_at: Option<String>,
    /// Last update timestamp
    pub updated_at: Option<String>,
}

/// Create a new job type
pub async fn create_job_type(
    State(_state): State<AppState>,
    Json(payload): Json<CreateJobTypeRequest>,
) -> (StatusCode, Json<JobTypeResponse>) {
    // In a real implementation, we would validate and save to a database
    // For now, we'll just create a mock response
    let job_type_id = Uuid::new_v4();
    
    let now = chrono::Utc::now().to_rfc3339();
    let response = JobTypeResponse {
        id: job_type_id,
        name: payload.name,
        description: payload.description,
        processor_type: payload.processor_type,
        processing_logic_id: payload.processing_logic_id,
        standard_cost_cents: payload.standard_cost_cents,
        enabled: payload.enabled,
        created_at: Some(now.clone()),
        updated_at: Some(now),
    };
    
    tracing::info!("Created new job type with ID: {}", job_type_id);
    (StatusCode::CREATED, Json(response))
}

/// Get a job type by ID
pub async fn get_job_type(
    State(_state): State<AppState>,
    Path(job_type_id): Path<String>,
) -> Result<Json<JobTypeResponse>, StatusCode> {
    // In a real implementation, we would fetch from a database
    // For now, we'll return a mock response if the ID looks valid
    
    // Try to parse the job_type_id as a UUID
    let job_type_id = match Uuid::parse_str(&job_type_id) {
        Ok(id) => id,
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    };
    
    let now = chrono::Utc::now().to_rfc3339();
    // Mock a job type response
    let response = JobTypeResponse {
        id: job_type_id,
        name: "Image Processing".to_string(),
        description: "Process and analyze images".to_string(),
        processor_type: "standard".to_string(),
        processing_logic_id: None,
        standard_cost_cents: 5000, // $50.00
        enabled: true,
        created_at: Some(now.clone()),
        updated_at: Some(now),
    };
    
    tracing::info!("Retrieved job type with ID: {}", job_type_id);
    Ok(Json(response))
}

/// Get all job types
#[allow(dead_code)]
pub async fn get_all_job_types(
    State(_state): State<AppState>,
) -> Result<Json<Vec<JobTypeResponse>>, StatusCode> {
    // In a real implementation, we would fetch from a database
    // For now, we'll just return a mock list of job types
    
    let now = chrono::Utc::now().to_rfc3339();
    
    // Create two mock job types
    let job_types = vec![
        JobTypeResponse {
            id: Uuid::new_v4(),
            name: "Image Processing".to_string(),
            description: "Process and analyze images".to_string(),
            processor_type: "standard".to_string(),
            processing_logic_id: None,
            standard_cost_cents: 5000, // $50.00
            enabled: true,
            created_at: Some(now.clone()),
            updated_at: Some(now.clone()),
        },
        JobTypeResponse {
            id: Uuid::new_v4(),
            name: "Webhook Processing".to_string(),
            description: "Sends data to a webhook".to_string(),
            processor_type: "webhook".to_string(),
            processing_logic_id: None,
            standard_cost_cents: 1000, // $10.00
            enabled: true,
            created_at: Some(now.clone()),
            updated_at: Some(now),
        },
    ];
    
    tracing::info!("Retrieved all job types");
    Ok(Json(job_types))
}
