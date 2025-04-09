use axum::{extract::{Path, State, Extension}, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tracing::{error, info};
use chrono::{Utc, Duration};

use crate::state::AppState;
use innosystem_common::models::runner::{NewRunner, RunnerStatus};
use crate::middleware::auth::AdminUser;

/// Request data for registering a new runner
#[derive(Debug, Deserialize)]
pub struct RegisterRunnerRequest {
    pub name: String,
    pub description: Option<String>,
    pub compatible_job_types: Vec<String>,
}

/// Request for updating runner capabilities
#[derive(Debug, Deserialize)]
pub struct UpdateRunnerCapabilitiesRequest {
    pub job_type_ids: Vec<Uuid>,
}

/// Response data for a runner
#[derive(Debug, Serialize)]
pub struct RunnerResponse {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub compatible_job_types: Vec<String>,
    pub last_heartbeat: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

/// Register a new runner
/// Access: Admin
pub async fn register_runner(
    State(state): State<AppState>,
    Extension(_admin): Extension<AdminUser>,
    Json(request): Json<RegisterRunnerRequest>,
) -> Result<(StatusCode, Json<RunnerResponse>), StatusCode> {
    // Create a new runner
    let new_runner = NewRunner {
        id: Uuid::new_v4(),
        name: request.name,
        description: request.description,
        status: RunnerStatus::Inactive.as_str().to_string(),
        compatible_job_types: request.compatible_job_types,
    };
    
    let runner = state.runner_repo.register(new_runner).await
        .map_err(|e| {
            error!("Failed to register runner: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    info!("Registered new runner: {}", runner.id);
    
    // Return the created runner
    Ok((StatusCode::CREATED, Json(RunnerResponse {
        id: runner.id,
        name: runner.name.clone(),
        description: runner.description.clone(),
        status: runner.status.as_str().to_string(),
        compatible_job_types: runner.compatible_job_types.clone(),
        last_heartbeat: runner.last_heartbeat.map(|dt| dt.and_utc().to_rfc3339()),
        created_at: runner.created_at.map(|dt| dt.and_utc().to_rfc3339()),
        updated_at: runner.updated_at.map(|dt| dt.and_utc().to_rfc3339()),
    })))
}

/// Update runner heartbeat
/// Access: Public (runner itself)
pub async fn update_heartbeat(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    // Update the runner's heartbeat with the current timestamp
    let now = Utc::now().naive_utc();
    state.runner_repo.update_heartbeat(id, now).await
        .map_err(|e| {
            error!("Failed to update runner heartbeat for {}: {}", id, e);
            StatusCode::NOT_FOUND
        })?;
    
    // Return success status
    Ok(StatusCode::OK)
}

/// Get a runner by ID
/// Access: Admin
pub async fn get_runner(
    State(state): State<AppState>,
    Extension(_admin): Extension<AdminUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<RunnerResponse>, StatusCode> {
    // Retrieve the runner from the database
    let runner = state.runner_repo.find_by_id(id).await
        .map_err(|e| {
            error!("Failed to find runner {}: {}", id, e);
            StatusCode::NOT_FOUND
        })?;
    
    // Return the runner
    Ok(Json(RunnerResponse {
        id: runner.id,
        name: runner.name.clone(),
        description: runner.description.clone(),
        status: runner.status.as_str().to_string(),
        compatible_job_types: runner.compatible_job_types.clone(),
        last_heartbeat: runner.last_heartbeat.map(|dt| dt.and_utc().to_rfc3339()),
        created_at: runner.created_at.map(|dt| dt.and_utc().to_rfc3339()),
        updated_at: runner.updated_at.map(|dt| dt.and_utc().to_rfc3339()),
    }))
}

/// Update runner capabilities
/// Access: Admin
pub async fn update_capabilities(
    State(state): State<AppState>,
    Extension(_admin): Extension<AdminUser>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateRunnerCapabilitiesRequest>,
) -> Result<Json<RunnerResponse>, StatusCode> {
    // Update the runner's capabilities
    let runner = state.runner_repo.update_capabilities(id, request.job_type_ids).await
        .map_err(|e| {
            error!("Failed to update runner capabilities for {}: {}", id, e);
            StatusCode::NOT_FOUND
        })?;
    
    info!("Updated capabilities for runner: {}", id);
    
    // Return the updated runner
    Ok(Json(RunnerResponse {
        id: runner.id,
        name: runner.name.clone(),
        description: runner.description.clone(),
        status: runner.status.as_str().to_string(),
        compatible_job_types: runner.compatible_job_types.clone(),
        last_heartbeat: runner.last_heartbeat.map(|dt| dt.and_utc().to_rfc3339()),
        created_at: runner.created_at.map(|dt| dt.and_utc().to_rfc3339()),
        updated_at: runner.updated_at.map(|dt| dt.and_utc().to_rfc3339()),
    }))
}

/// List all runners
/// Access: Admin
pub async fn list_all_runners(
    State(state): State<AppState>,
    Extension(_admin): Extension<AdminUser>,
) -> Result<Json<Vec<RunnerResponse>>, StatusCode> {
    // Retrieve all runners
    let runners = state.runner_repo.list_all().await
        .map_err(|e| {
            error!("Failed to list all runners: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    // Convert to response format
    let runner_responses = runners.into_iter()
        .map(|runner| RunnerResponse {
            id: runner.id,
            name: runner.name.clone(),
            description: runner.description.clone(),
            status: runner.status.as_str().to_string(),
            compatible_job_types: runner.compatible_job_types.clone(),
            last_heartbeat: runner.last_heartbeat.map(|dt| dt.and_utc().to_rfc3339()),
            created_at: runner.created_at.map(|dt| dt.and_utc().to_rfc3339()),
            updated_at: runner.updated_at.map(|dt| dt.and_utc().to_rfc3339()),
        })
        .collect();
    
    // Return the runners
    Ok(Json(runner_responses))
}

/// List active runners
/// Access: Admin
pub async fn list_active_runners(
    State(state): State<AppState>,
    Extension(_admin): Extension<AdminUser>,
) -> Result<Json<Vec<RunnerResponse>>, StatusCode> {
    // Define what "active" means (heartbeat within last 5 minutes)
    let since = (Utc::now() - Duration::minutes(5)).naive_utc();
    
    // Retrieve active runners
    let runners = state.runner_repo.list_active(since).await
        .map_err(|e| {
            error!("Failed to list active runners: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    // Convert to response format
    let runner_responses = runners.into_iter()
        .map(|runner| RunnerResponse {
            id: runner.id,
            name: runner.name.clone(),
            description: runner.description.clone(),
            status: runner.status.as_str().to_string(),
            compatible_job_types: runner.compatible_job_types.clone(),
            last_heartbeat: runner.last_heartbeat.map(|dt| dt.and_utc().to_rfc3339()),
            created_at: runner.created_at.map(|dt| dt.and_utc().to_rfc3339()),
            updated_at: runner.updated_at.map(|dt| dt.and_utc().to_rfc3339()),
        })
        .collect();
    
    // Return the runners
    Ok(Json(runner_responses))
}

/// Set runner status (active/inactive)
/// Access: Admin
pub async fn set_runner_status(
    State(state): State<AppState>,
    Extension(_admin): Extension<AdminUser>,
    Path(id): Path<Uuid>,
    Json(active): Json<bool>,
) -> Result<Json<RunnerResponse>, StatusCode> {
    // Update the runner's status
    let runner = state.runner_repo.set_status(id, active).await
        .map_err(|e| {
            error!("Failed to set runner status for {}: {}", id, e);
            StatusCode::NOT_FOUND
        })?;
    
    info!("Set runner {} status to {}", id, if active { "active" } else { "inactive" });
    
    // Return the updated runner
    Ok(Json(RunnerResponse {
        id: runner.id,
        name: runner.name.clone(),
        description: runner.description.clone(),
        status: runner.status.as_str().to_string(),
        compatible_job_types: runner.compatible_job_types.clone(),
        last_heartbeat: runner.last_heartbeat.map(|dt| dt.and_utc().to_rfc3339()),
        created_at: runner.created_at.map(|dt| dt.and_utc().to_rfc3339()),
        updated_at: runner.updated_at.map(|dt| dt.and_utc().to_rfc3339()),
    }))
}
