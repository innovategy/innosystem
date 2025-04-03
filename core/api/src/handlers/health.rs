use axum::{
    http::StatusCode,
    Json,
};
use serde::Serialize;

/// Response structure for health endpoint
#[derive(Serialize)]
pub struct HealthResponse {
    status: String,
}

/// Health check endpoint handler
#[allow(dead_code)]
pub async fn health_check() -> (StatusCode, Json<HealthResponse>) {
    (
        StatusCode::OK,
        Json(HealthResponse {
            status: "OK".to_string(),
        }),
    )
}
