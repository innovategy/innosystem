use axum::{
    extract::{Path, State, Extension},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tracing::{info, error};

use crate::state::AppState;
use innosystem_common::models::reseller::{Reseller, NewReseller};

/// Request data for creating a new reseller
#[derive(Debug, Deserialize)]
pub struct CreateResellerRequest {
    /// Reseller name
    pub name: String,
    /// Reseller email
    pub email: String,
    /// Commission rate as a percentage (e.g., 10.5 for 10.5%)
    pub commission_rate_percentage: f64,
}

/// Request data for updating a reseller
#[derive(Debug, Deserialize)]
pub struct UpdateResellerRequest {
    /// Reseller name
    pub name: Option<String>,
    /// Reseller email
    pub email: Option<String>,
    /// Commission rate as a percentage
    pub commission_rate_percentage: Option<f64>,
    /// Whether the reseller is active
    pub active: Option<bool>,
}

/// Response data for reseller operations
#[derive(Debug, Serialize)]
pub struct ResellerResponse {
    /// Reseller ID
    pub id: Uuid,
    /// Reseller name
    pub name: String,
    /// Reseller email
    pub email: String,
    /// Reseller API key
    pub api_key: String,
    /// Whether the reseller is active
    pub active: bool,
    /// Commission rate as a percentage (e.g., 10.5 for 10.5%)
    pub commission_rate_percentage: f64,
    /// Creation timestamp
    pub created_at: Option<String>,
    /// Last update timestamp
    pub updated_at: Option<String>,
}

/// Create a new reseller
pub async fn create_reseller(
    State(state): State<AppState>,
    Json(payload): Json<CreateResellerRequest>,
) -> Result<(StatusCode, Json<ResellerResponse>), StatusCode> {
    // Generate a new API key for the reseller
    let api_key = Reseller::generate_api_key();
    
    // Create the reseller model with a new UUID
    let mut new_reseller = Reseller::new(
        payload.name.clone(),
        payload.email.clone(),
        api_key,
        0, // Temporary commission rate, will be set from percentage below
    );
    
    // Set commission rate from percentage
    new_reseller.set_commission_rate_from_percentage(payload.commission_rate_percentage);
    
    // Convert to NewReseller for database insertion
    let new_reseller_db = NewReseller::from(new_reseller.clone());
    
    // Insert the reseller into the database
    let reseller = match state.reseller_repo.create(new_reseller_db).await {
        Ok(reseller) => reseller,
        Err(e) => {
            error!("Failed to create reseller: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    
    // Create the response
    let response = ResellerResponse {
        id: reseller.id,
        name: reseller.name.clone(),
        email: reseller.email.clone(),
        api_key: reseller.api_key.clone(),
        active: reseller.active,
        commission_rate_percentage: reseller.commission_rate_percentage(),
        created_at: reseller.created_at.map(|dt| dt.and_utc().to_rfc3339()),
        updated_at: reseller.updated_at.map(|dt| dt.and_utc().to_rfc3339()),
    };
    
    info!("Created new reseller with ID: {}", reseller.id);
    Ok((StatusCode::CREATED, Json(response)))
}

/// Get a reseller by ID
pub async fn get_reseller(
    State(state): State<AppState>,
    Path(reseller_id_str): Path<String>,
) -> Result<Json<ResellerResponse>, StatusCode> {
    // Try to parse the reseller_id as a UUID
    let reseller_id = match Uuid::parse_str(&reseller_id_str) {
        Ok(id) => id,
        Err(_) => {
            error!("Invalid reseller ID format: {}", reseller_id_str);
            return Err(StatusCode::BAD_REQUEST);
        }
    };
    
    // Fetch the reseller from the repository
    let reseller = state.reseller_repo.find_by_id(reseller_id).await
        .map_err(|e| {
            error!("Failed to fetch reseller: {}", e);
            // If reseller not found, return 404
            if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;
    
    // Create the response
    let response = ResellerResponse {
        id: reseller.id,
        name: reseller.name.clone(),
        email: reseller.email.clone(),
        api_key: reseller.api_key.clone(),
        active: reseller.active,
        commission_rate_percentage: reseller.commission_rate_percentage(),
        created_at: reseller.created_at.map(|dt| dt.and_utc().to_rfc3339()),
        updated_at: reseller.updated_at.map(|dt| dt.and_utc().to_rfc3339()),
    };
    
    info!("Retrieved reseller with ID: {}", reseller.id);
    Ok(Json(response))
}

/// Update a reseller
pub async fn update_reseller(
    State(state): State<AppState>,
    Path(reseller_id_str): Path<String>,
    Json(payload): Json<UpdateResellerRequest>,
) -> Result<Json<ResellerResponse>, StatusCode> {
    // Try to parse the reseller_id as a UUID
    let reseller_id = match Uuid::parse_str(&reseller_id_str) {
        Ok(id) => id,
        Err(_) => {
            error!("Invalid reseller ID format: {}", reseller_id_str);
            return Err(StatusCode::BAD_REQUEST);
        }
    };
    
    // Fetch the reseller from the repository
    let mut reseller = state.reseller_repo.find_by_id(reseller_id).await
        .map_err(|e| {
            error!("Failed to fetch reseller: {}", e);
            // If reseller not found, return 404
            if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;
    
    // Update fields if provided
    if let Some(name) = payload.name {
        reseller.name = name;
    }
    
    if let Some(email) = payload.email {
        reseller.email = email;
    }
    
    if let Some(commission_rate) = payload.commission_rate_percentage {
        reseller.set_commission_rate_from_percentage(commission_rate);
    }
    
    if let Some(active) = payload.active {
        reseller.active = active;
    }
    
    // Update the reseller in the database
    let updated_reseller = state.reseller_repo.update(&reseller).await
        .map_err(|e| {
            error!("Failed to update reseller: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    // Create the response
    let response = ResellerResponse {
        id: updated_reseller.id,
        name: updated_reseller.name.clone(),
        email: updated_reseller.email.clone(),
        api_key: updated_reseller.api_key.clone(),
        active: updated_reseller.active,
        commission_rate_percentage: updated_reseller.commission_rate_percentage(),
        created_at: updated_reseller.created_at.map(|dt| dt.and_utc().to_rfc3339()),
        updated_at: updated_reseller.updated_at.map(|dt| dt.and_utc().to_rfc3339()),
    };
    
    info!("Updated reseller with ID: {}", updated_reseller.id);
    Ok(Json(response))
}

/// Get current reseller profile based on API key
pub async fn get_current_reseller_profile(
    State(state): State<AppState>,
    Extension(api_key): Extension<String>,
) -> Result<Json<ResellerResponse>, StatusCode> {
    // Fetch the reseller from the repository using the API key from the request extension
    let reseller = state.reseller_repo.find_by_api_key(&api_key).await
        .map_err(|e| {
            error!("Failed to fetch reseller by API key: {}", e);
            // If reseller not found, return 404
            if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;
    
    // Create the response
    let response = ResellerResponse {
        id: reseller.id,
        name: reseller.name.clone(),
        email: reseller.email.clone(),
        api_key: reseller.api_key.clone(),
        active: reseller.active,
        commission_rate_percentage: reseller.commission_rate_percentage(),
        created_at: reseller.created_at.map(|dt| dt.and_utc().to_rfc3339()),
        updated_at: reseller.updated_at.map(|dt| dt.and_utc().to_rfc3339()),
    };
    
    info!("Retrieved current reseller profile with ID: {}", reseller.id);
    Ok(Json(response))
}

/// Get all resellers
pub async fn get_all_resellers(
    State(state): State<AppState>,
) -> Result<Json<Vec<ResellerResponse>>, StatusCode> {
    // Fetch all resellers from the repository
    let resellers = state.reseller_repo.list_all().await
        .map_err(|e| {
            error!("Failed to fetch resellers: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    // Convert to response format
    let reseller_responses: Vec<ResellerResponse> = resellers
        .into_iter()
        .map(|reseller| ResellerResponse {
            id: reseller.id,
            name: reseller.name.clone(),
            email: reseller.email.clone(),
            api_key: reseller.api_key.clone(),
            active: reseller.active,
            commission_rate_percentage: reseller.commission_rate_percentage(),
            created_at: reseller.created_at.map(|dt| dt.and_utc().to_rfc3339()),
            updated_at: reseller.updated_at.map(|dt| dt.and_utc().to_rfc3339()),
        })
        .collect();
    
    info!("Retrieved all resellers from database");
    Ok(Json(reseller_responses))
}

/// Get active resellers only
pub async fn get_active_resellers(
    State(state): State<AppState>,
) -> Result<Json<Vec<ResellerResponse>>, StatusCode> {
    // Fetch active resellers from the repository
    let resellers = state.reseller_repo.list_active().await
        .map_err(|e| {
            error!("Failed to fetch active resellers: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    // Convert to response format
    let reseller_responses: Vec<ResellerResponse> = resellers
        .into_iter()
        .map(|reseller| ResellerResponse {
            id: reseller.id,
            name: reseller.name.clone(),
            email: reseller.email.clone(),
            api_key: reseller.api_key.clone(),
            active: reseller.active,
            commission_rate_percentage: reseller.commission_rate_percentage(),
            created_at: reseller.created_at.map(|dt| dt.and_utc().to_rfc3339()),
            updated_at: reseller.updated_at.map(|dt| dt.and_utc().to_rfc3339()),
        })
        .collect();
    
    info!("Retrieved active resellers from database");
    Ok(Json(reseller_responses))
}

/// Generate a new API key for a reseller
pub async fn regenerate_api_key(
    State(state): State<AppState>,
    Path(reseller_id_str): Path<String>,
) -> Result<Json<ResellerResponse>, StatusCode> {
    // Try to parse the reseller_id as a UUID
    let reseller_id = match Uuid::parse_str(&reseller_id_str) {
        Ok(id) => id,
        Err(_) => {
            error!("Invalid reseller ID format: {}", reseller_id_str);
            return Err(StatusCode::BAD_REQUEST);
        }
    };
    
    // Fetch the reseller from the repository
    let mut reseller = state.reseller_repo.find_by_id(reseller_id).await
        .map_err(|e| {
            error!("Failed to fetch reseller: {}", e);
            // If reseller not found, return 404
            if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;
    
    // Generate a new API key
    reseller.api_key = Reseller::generate_api_key();
    
    // Update the reseller in the database
    let updated_reseller = state.reseller_repo.update(&reseller).await
        .map_err(|e| {
            error!("Failed to update reseller API key: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    // Create the response
    let response = ResellerResponse {
        id: updated_reseller.id,
        name: updated_reseller.name.clone(),
        email: updated_reseller.email.clone(),
        api_key: updated_reseller.api_key.clone(),
        active: updated_reseller.active,
        commission_rate_percentage: updated_reseller.commission_rate_percentage(),
        created_at: updated_reseller.created_at.map(|dt| dt.and_utc().to_rfc3339()),
        updated_at: updated_reseller.updated_at.map(|dt| dt.and_utc().to_rfc3339()),
    };
    
    info!("Regenerated API key for reseller with ID: {}", updated_reseller.id);
    Ok(Json(response))
}
