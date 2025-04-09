use axum::{extract::{Path, State, Extension}, http::StatusCode, Json};
use serde::Serialize;
use uuid::Uuid;
use tracing::{error, info};

use crate::state::AppState;
use crate::middleware::auth::AdminUser;
// RunnerHealthStatus is used internally in the service

/// Response for runner health status
#[derive(Debug, Serialize)]
pub struct RunnerHealthResponse {
    pub runner_id: Uuid,
    pub status: String,
    pub compatible_job_types: Vec<String>,
    pub last_heartbeat: Option<String>,
}

/// Response for runner compatibility check
#[derive(Debug, Serialize)]
pub struct CompatibilityResponse {
    pub runner_id: Uuid,
    pub job_type_id: Uuid,
    pub is_compatible: bool,
}

/// Response for compatible runners
#[derive(Debug, Serialize)]
pub struct CompatibleRunnersResponse {
    pub job_type_id: Uuid,
    pub compatible_runners: Vec<RunnerHealthInfo>,
}

/// Information about a runner's health
#[derive(Debug, Serialize)]
pub struct RunnerHealthInfo {
    pub runner_id: Uuid,
    pub health_status: String,
}

/// Check the health status of a runner
/// Access: Admin
pub async fn check_runner_health(
    State(state): State<AppState>,
    Extension(_admin): Extension<AdminUser>,
    Path(runner_id): Path<Uuid>,
) -> Result<Json<RunnerHealthResponse>, StatusCode> {
    // Get the runner
    let runner = state.runner_repo.find_by_id(runner_id).await
        .map_err(|e| {
            error!("Failed to find runner {}: {}", runner_id, e);
            StatusCode::NOT_FOUND
        })?;
    
    // Check health status
    let health_status = state.runner_health_service.check_runner_health(runner_id).await
        .map_err(|e| {
            error!("Failed to check health for runner {}: {}", runner_id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    // Return runner health info
    Ok(Json(RunnerHealthResponse {
        runner_id: runner.id,
        status: health_status.as_str().to_string(),
        compatible_job_types: runner.compatible_job_types.clone(),
        last_heartbeat: runner.last_heartbeat.map(|dt| dt.and_utc().to_rfc3339()),
    }))
}

/// Check if a runner is compatible with a job type
/// Access: Admin
pub async fn check_compatibility(
    State(state): State<AppState>,
    Extension(_admin): Extension<AdminUser>,
    Path((runner_id, job_type_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<CompatibilityResponse>, StatusCode> {
    // Check compatibility
    let is_compatible = state.runner_health_service.is_compatible_with_job_type(runner_id, job_type_id).await
        .map_err(|e| {
            error!("Failed to check compatibility for runner {} and job type {}: {}", 
                   runner_id, job_type_id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    // Return compatibility response
    Ok(Json(CompatibilityResponse {
        runner_id,
        job_type_id,
        is_compatible,
    }))
}

/// Find compatible runners for a job type
/// Access: Admin
pub async fn find_compatible_runners(
    State(state): State<AppState>,
    Extension(_admin): Extension<AdminUser>,
    Path(job_type_id): Path<Uuid>,
) -> Result<Json<CompatibleRunnersResponse>, StatusCode> {
    // Find compatible runners
    let compatible_runners = state.runner_health_service.find_compatible_runners(job_type_id).await
        .map_err(|e| {
            error!("Failed to find compatible runners for job type {}: {}", job_type_id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    // Convert to response format
    let runner_infos = compatible_runners.into_iter()
        .map(|(runner_id, health_status)| RunnerHealthInfo {
            runner_id,
            health_status: health_status.as_str().to_string(),
        })
        .collect();
    
    // Return compatible runners response
    Ok(Json(CompatibleRunnersResponse {
        job_type_id,
        compatible_runners: runner_infos,
    }))
}

/// Force check and reassign jobs from unhealthy runners
/// Access: Admin
pub async fn check_and_reassign_jobs(
    State(state): State<AppState>,
    Extension(_admin): Extension<AdminUser>,
) -> Result<Json<u32>, StatusCode> {
    // Check and reassign jobs
    let reassigned_count = state.runner_health_service.check_and_reassign_jobs().await
        .map_err(|e| {
            error!("Failed to check and reassign jobs: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    info!("Reassigned {} jobs from unhealthy runners", reassigned_count);
    
    // Return number of reassigned jobs
    Ok(Json(reassigned_count))
}
