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
    State(state): State<AppState>,
    Json(payload): Json<CreateJobTypeRequest>,
) -> (StatusCode, Json<JobTypeResponse>) {
    tracing::info!("Received job type creation request: name={}, processor_type={}", payload.name, payload.processor_type);
    // Parse processor type from string
    let processor_type = match innosystem_common::models::job_type::ProcessorType::from_str(&payload.processor_type) {
        Some(pt) => pt,
        None => {
            tracing::error!("Invalid processor type: {}", payload.processor_type);
            tracing::error!("Valid processor types are: sync, async, external_api, batch, webhook");
            return (StatusCode::BAD_REQUEST, Json(JobTypeResponse {
                id: Uuid::nil(),
                name: "".to_string(),
                description: "".to_string(),
                processor_type: "".to_string(),
                processing_logic_id: None,
                standard_cost_cents: 0,
                enabled: false,
                created_at: None,
                updated_at: None,
            }));
        }
    };
    
    // Create the job type model for database insertion
    let new_job_type = innosystem_common::models::job_type::NewJobType {
        id: Uuid::new_v4(),
        name: payload.name.clone(),
        description: Some(payload.description.clone()),
        processor_type: processor_type.as_str().to_string(),
        processing_logic_id: payload.processing_logic_id
            .map(|uuid| uuid.to_string())
            .unwrap_or_else(|| Uuid::new_v4().to_string()),
        standard_cost_cents: payload.standard_cost_cents,
        enabled: payload.enabled,
    };
    
    tracing::debug!("Creating job type with processor_type: {}", processor_type.as_str());
    
    // Insert the job type into the database
    tracing::debug!("Inserting job type into database with processor_type={}", processor_type.as_str());
    let job_type = match state.job_type_repo.create(new_job_type).await {
        Ok(jt) => jt,
        Err(e) => {
            tracing::error!("Failed to create job type: {}", e);
            tracing::error!("Error details: {:?}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(JobTypeResponse {
                id: Uuid::nil(),
                name: "".to_string(),
                description: "".to_string(),
                processor_type: "".to_string(),
                processing_logic_id: None,
                standard_cost_cents: 0,
                enabled: false,
                created_at: None,
                updated_at: None,
            }));
        }
    };
    
    // Create the response
    let response = JobTypeResponse {
        id: job_type.id,
        name: job_type.name,
        description: job_type.description.unwrap_or_default(),
        processor_type: job_type.processor_type.as_str().to_string(),
        processing_logic_id: Uuid::parse_str(&job_type.processing_logic_id).ok(),
        standard_cost_cents: job_type.standard_cost_cents,
        enabled: job_type.enabled,
        created_at: job_type.created_at.map(|dt| dt.and_utc().to_rfc3339()),
        updated_at: job_type.updated_at.map(|dt| dt.and_utc().to_rfc3339()),
    };
    
    tracing::info!("Created new job type with ID: {}", job_type.id);
    (StatusCode::CREATED, Json(response))
}

/// Get a job type by ID
pub async fn get_job_type(
    State(state): State<AppState>,
    Path(job_type_id_str): Path<String>,
) -> Result<Json<JobTypeResponse>, StatusCode> {
    // Try to parse the job_type_id as a UUID
    let job_type_id = match Uuid::parse_str(&job_type_id_str) {
        Ok(id) => id,
        Err(_) => {
            tracing::error!("Invalid job type ID format: {}", job_type_id_str);
            return Err(StatusCode::BAD_REQUEST);
        }
    };
    
    // Fetch the job type from the repository
    let job_type = state.job_type_repo.find_by_id(job_type_id).await
        .map_err(|e| {
            tracing::error!("Failed to fetch job type: {}", e);
            // If job type not found, return 404
            if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;
    
    // Create the response
    let response = JobTypeResponse {
        id: job_type.id,
        name: job_type.name,
        description: job_type.description.unwrap_or_default(),
        processor_type: job_type.processor_type.as_str().to_string(),
        processing_logic_id: Uuid::parse_str(&job_type.processing_logic_id).ok(),
        standard_cost_cents: job_type.standard_cost_cents,
        enabled: job_type.enabled,
        created_at: job_type.created_at.map(|dt| dt.and_utc().to_rfc3339()),
        updated_at: job_type.updated_at.map(|dt| dt.and_utc().to_rfc3339()),
    };
    
    tracing::info!("Retrieved job type with ID: {}", job_type.id);
    Ok(Json(response))
}

/// Get all job types
#[allow(dead_code)]
pub async fn get_all_job_types(
    State(state): State<AppState>,
) -> Result<Json<Vec<JobTypeResponse>>, StatusCode> {
    // Fetch all job types from the repository
    let job_types = state.job_type_repo.list_all().await
        .map_err(|e| {
            tracing::error!("Failed to fetch job types: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    // Convert to response format
    let job_type_responses = job_types.into_iter().map(|jt| {
        JobTypeResponse {
            id: jt.id,
            name: jt.name,
            description: jt.description.unwrap_or_default(),
            processor_type: jt.processor_type.as_str().to_string(),
            processing_logic_id: Uuid::parse_str(&jt.processing_logic_id).ok(),
            standard_cost_cents: jt.standard_cost_cents,
            enabled: jt.enabled,
            created_at: jt.created_at.map(|dt| dt.and_utc().to_rfc3339()),
            updated_at: jt.updated_at.map(|dt| dt.and_utc().to_rfc3339()),
        }
    }).collect();
    
    tracing::info!("Retrieved all job types from database");
    Ok(Json(job_type_responses))
}
