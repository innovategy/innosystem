use axum::{extract::{Path, State}, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::state::AppState;

/// Request data for creating a new customer
#[derive(Debug, Deserialize)]
pub struct CreateCustomerRequest {
    /// Customer name
    pub name: String,
    /// Customer email
    pub email: String,
    /// Initial balance in cents (optional)
    pub initial_balance_cents: Option<i64>,
}

/// Response data for customer operations
#[derive(Debug, Serialize)]
pub struct CustomerResponse {
    /// Customer ID
    pub id: Uuid,
    /// Customer name
    pub name: String,
    /// Customer email
    pub email: String,
    /// API key
    pub api_key: String,
    /// Wallet ID
    pub wallet_id: Option<Uuid>,
    /// Wallet balance in cents
    pub balance_cents: Option<i64>,
    /// Creation timestamp
    pub created_at: Option<String>,
    /// Last update timestamp
    pub updated_at: Option<String>,
}

/// Create a new customer
pub async fn create_customer(
    State(_state): State<AppState>,
    Json(payload): Json<CreateCustomerRequest>,
) -> (StatusCode, Json<CustomerResponse>) {
    // In a real implementation, we would validate and save to a database
    // For now, we'll just create a mock response
    let customer_id = Uuid::new_v4();
    let wallet_id = Uuid::new_v4();
    let api_key = format!("api_{}", Uuid::new_v4().to_string().replace("-", ""));
    
    let now = chrono::Utc::now().to_rfc3339();
    let response = CustomerResponse {
        id: customer_id,
        name: payload.name,
        email: payload.email,
        api_key,
        wallet_id: Some(wallet_id),
        balance_cents: payload.initial_balance_cents.or(Some(0)),
        created_at: Some(now.clone()),
        updated_at: Some(now),
    };
    
    tracing::info!("Created new customer with ID: {}", customer_id);
    (StatusCode::CREATED, Json(response))
}

/// Get a customer by ID
pub async fn get_customer(
    State(_state): State<AppState>,
    Path(customer_id): Path<String>,
) -> Result<Json<CustomerResponse>, StatusCode> {
    // In a real implementation, we would fetch from a database
    // For now, we'll return a mock response if the ID looks valid
    
    // Try to parse the customer_id as a UUID
    let customer_id = match Uuid::parse_str(&customer_id) {
        Ok(id) => id,
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    };
    
    let wallet_id = Uuid::new_v4();
    let api_key = format!("api_{}", Uuid::new_v4().to_string().replace("-", ""));
    let now = chrono::Utc::now().to_rfc3339();
    
    // Mock a customer response
    let response = CustomerResponse {
        id: customer_id,
        name: "Example Customer".to_string(),
        email: "customer@example.com".to_string(),
        api_key,
        wallet_id: Some(wallet_id),
        balance_cents: Some(10000), // $100.00
        created_at: Some(now.clone()),
        updated_at: Some(now),
    };
    
    tracing::info!("Retrieved customer with ID: {}", customer_id);
    Ok(Json(response))
}

/// Get all customers
#[allow(dead_code)]
pub async fn get_all_customers(
    State(_state): State<AppState>,
) -> Result<Json<Vec<CustomerResponse>>, StatusCode> {
    // In a real implementation, we would fetch from a database
    // For now, we'll just return a mock list of customers
    
    let now = chrono::Utc::now().to_rfc3339();
    
    // Create two mock customers
    let customers = vec![
        CustomerResponse {
            id: Uuid::new_v4(),
            name: "Example Customer 1".to_string(),
            email: "customer1@example.com".to_string(),
            api_key: format!("api_{}", Uuid::new_v4().to_string().replace("-", "")),
            wallet_id: Some(Uuid::new_v4()),
            balance_cents: Some(10000), // $100.00
            created_at: Some(now.clone()),
            updated_at: Some(now.clone()),
        },
        CustomerResponse {
            id: Uuid::new_v4(),
            name: "Example Customer 2".to_string(),
            email: "customer2@example.com".to_string(),
            api_key: format!("api_{}", Uuid::new_v4().to_string().replace("-", "")),
            wallet_id: Some(Uuid::new_v4()),
            balance_cents: Some(5000), // $50.00
            created_at: Some(now.clone()),
            updated_at: Some(now),
        },
    ];
    
    tracing::info!("Retrieved all customers");
    Ok(Json(customers))
}
