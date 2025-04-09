use axum::{extract::{Path, State, Extension}, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tracing::{error, info};

use crate::state::AppState;
use innosystem_common::models::project::NewProject;
use crate::middleware::auth::{AdminUser, CustomerUser};

/// Request data for creating a new project
#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: Option<String>,
}

/// Response data for a project
#[derive(Debug, Serialize)]
pub struct ProjectResponse {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

/// Create a new project for a customer
/// Access: Customer or Admin
pub async fn create_project(
    State(state): State<AppState>,
    Extension(customer): Extension<CustomerUser>,
    Json(request): Json<CreateProjectRequest>,
) -> Result<(StatusCode, Json<ProjectResponse>), StatusCode> {
    // Create a new project for the customer
    let new_project = NewProject {
        id: Uuid::new_v4(),
        customer_id: customer.id,
        name: request.name,
        description: request.description,
    };
    
    let project = state.project_repo.create(new_project).await
        .map_err(|e| {
            error!("Failed to create project: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    info!("Created new project: {}", project.id);
    
    // Return the created project
    Ok((StatusCode::CREATED, Json(ProjectResponse {
        id: project.id,
        customer_id: project.customer_id,
        name: project.name.clone(),
        description: project.description.clone(),
        created_at: project.created_at.map(|dt| dt.and_utc().to_rfc3339()),
        updated_at: project.updated_at.map(|dt| dt.and_utc().to_rfc3339()),
    })))
}

/// Get a project by ID
/// Access: Project's Customer or Admin
pub async fn get_project(
    State(state): State<AppState>,
    Extension(customer): Extension<CustomerUser>,
    Path(id): Path<Uuid>,
) -> Result<Json<ProjectResponse>, StatusCode> {
    // Retrieve the project from the database
    let project = state.project_repo.find_by_id(id).await
        .map_err(|e| {
            error!("Failed to find project {}: {}", id, e);
            StatusCode::NOT_FOUND
        })?;
    
    // Verify the customer is authorized to access this project
    if project.customer_id != customer.id {
        // Check if the customer is associated with a reseller
        if customer.reseller_id.is_none() {
            return Err(StatusCode::FORBIDDEN);
        }
    }
    
    // Return the project
    Ok(Json(ProjectResponse {
        id: project.id,
        customer_id: project.customer_id,
        name: project.name.clone(),
        description: project.description.clone(),
        created_at: project.created_at.map(|dt| dt.and_utc().to_rfc3339()),
        updated_at: project.updated_at.map(|dt| dt.and_utc().to_rfc3339()),
    }))
}

/// Update a project
/// Access: Project's Customer or Admin
pub async fn update_project(
    State(state): State<AppState>,
    Extension(customer): Extension<CustomerUser>,
    Path(id): Path<Uuid>,
    Json(request): Json<CreateProjectRequest>,
) -> Result<Json<ProjectResponse>, StatusCode> {
    // First retrieve the project
    let mut project = state.project_repo.find_by_id(id).await
        .map_err(|e| {
            error!("Failed to find project {}: {}", id, e);
            StatusCode::NOT_FOUND
        })?;
    
    // Verify the customer is authorized to update this project
    if project.customer_id != customer.id {
        // Check if the customer is associated with a reseller
        if customer.reseller_id.is_none() {
            return Err(StatusCode::FORBIDDEN);
        }
    }
    
    // Update the project fields
    project.name = request.name;
    project.description = request.description;
    
    // Save the updated project
    let updated_project = state.project_repo.update(&project).await
        .map_err(|e| {
            error!("Failed to update project {}: {}", id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    info!("Updated project: {}", updated_project.id);
    
    // Return the updated project
    Ok(Json(ProjectResponse {
        id: updated_project.id,
        customer_id: updated_project.customer_id,
        name: updated_project.name.clone(),
        description: updated_project.description.clone(),
        created_at: updated_project.created_at.map(|dt| dt.and_utc().to_rfc3339()),
        updated_at: updated_project.updated_at.map(|dt| dt.and_utc().to_rfc3339()),
    }))
}

/// Delete a project
/// Access: Project's Customer or Admin
pub async fn delete_project(
    State(state): State<AppState>,
    Extension(customer): Extension<CustomerUser>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    // First retrieve the project to check ownership
    let project = state.project_repo.find_by_id(id).await
        .map_err(|e| {
            error!("Failed to find project {}: {}", id, e);
            StatusCode::NOT_FOUND
        })?;
    
    // Verify the customer is authorized to delete this project
    if project.customer_id != customer.id {
        // Check if the customer is associated with a reseller
        if customer.reseller_id.is_none() {
            return Err(StatusCode::FORBIDDEN);
        }
    }
    
    // Delete the project
    state.project_repo.delete(id).await
        .map_err(|e| {
            error!("Failed to delete project {}: {}", id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    info!("Deleted project: {}", id);
    
    // Return success status
    Ok(StatusCode::NO_CONTENT)
}

/// List all projects for a customer
/// Access: Customer
pub async fn list_customer_projects(
    State(state): State<AppState>,
    Extension(customer): Extension<CustomerUser>,
) -> Result<Json<Vec<ProjectResponse>>, StatusCode> {
    // Retrieve all projects for the customer
    let projects = state.project_repo.find_by_customer_id(customer.id).await
        .map_err(|e| {
            error!("Failed to list projects for customer {}: {}", customer.id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    // Convert to response format
    let project_responses = projects.into_iter()
        .map(|project| ProjectResponse {
            id: project.id,
            customer_id: project.customer_id,
            name: project.name.clone(),
            description: project.description.clone(),
            created_at: project.created_at.map(|dt| dt.and_utc().to_rfc3339()),
            updated_at: project.updated_at.map(|dt| dt.and_utc().to_rfc3339()),
        })
        .collect();
    
    // Return the projects
    Ok(Json(project_responses))
}

/// List all projects (admin only)
/// Access: Admin
pub async fn list_all_projects(
    State(state): State<AppState>,
    Extension(_admin): Extension<AdminUser>,
) -> Result<Json<Vec<ProjectResponse>>, StatusCode> {
    // Retrieve all projects
    let projects = state.project_repo.list_all().await
        .map_err(|e| {
            error!("Failed to list all projects: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    // Convert to response format
    let project_responses = projects.into_iter()
        .map(|project| ProjectResponse {
            id: project.id,
            customer_id: project.customer_id,
            name: project.name.clone(),
            description: project.description.clone(),
            created_at: project.created_at.map(|dt| dt.and_utc().to_rfc3339()),
            updated_at: project.updated_at.map(|dt| dt.and_utc().to_rfc3339()),
        })
        .collect();
    
    // Return the projects
    Ok(Json(project_responses))
}
