use axum::{extract::{Path, State, Extension}, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tracing::error;

use crate::state::AppState;
// Customer model is imported via NewCustomer

/// Request data for creating a new customer
#[derive(Debug, Deserialize)]
pub struct CreateCustomerRequest {
    /// Customer name
    pub name: String,
    /// Customer email
    pub email: String,
    /// Initial balance in cents (optional)
    pub initial_balance_cents: Option<i64>,
    /// Reseller ID (optional, will be set from context if not provided)
    pub reseller_id: Option<Uuid>,
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
    /// API key (if available)
    pub api_key: Option<String>,
    /// Reseller ID (if the customer belongs to a reseller)
    pub reseller_id: Option<Uuid>,
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
    State(state): State<AppState>,
    Extension(api_key): Extension<String>,
    Json(payload): Json<CreateCustomerRequest>,
) -> (StatusCode, Json<CustomerResponse>) {
    // Determine the reseller_id based on API key
    let reseller_id = match payload.reseller_id {
        // If specified in payload, use that (for admin operations)
        Some(id) => Some(id),
        // Otherwise, lookup reseller by API key (for reseller operations)
        None => match state.reseller_repo.find_by_api_key(&api_key).await {
            Ok(reseller) => Some(reseller.id),
            Err(e) => {
                // If it's an admin key, then customer has no reseller
                if &api_key == &state.config.admin_api_key {
                    None
                } else {
                    error!("Failed to find reseller by API key: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, Json(CustomerResponse {
                        id: Uuid::nil(),
                        name: "".to_string(),
                        email: "".to_string(),
                        api_key: None,
                        reseller_id: None,
                        wallet_id: None,
                        balance_cents: None,
                        created_at: None,
                        updated_at: None,
                    }));
                }
            }
        }
    };
    
    // Generate API key if needed
    let api_key = if reseller_id.is_some() {
        // Customers under a reseller get their own API key
        Some(format!("cust_{}", Uuid::new_v4().simple()))
    } else {
        None
    };
    
    // Create the customer model with a new UUID
    let new_customer = innosystem_common::models::customer::NewCustomer {
        id: Uuid::new_v4(),
        name: payload.name.clone(),
        email: payload.email.clone(),
        api_key,
        reseller_id,
    };
    
    // Insert the customer into the database
    let customer = match state.customer_repo.create(new_customer).await {
        Ok(customer) => customer,
        Err(e) => {
            tracing::error!("Failed to create customer: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(CustomerResponse {
                id: Uuid::nil(),
                name: "".to_string(),
                email: "".to_string(),
                api_key: None,
                reseller_id: None,
                wallet_id: None,
                balance_cents: None,
                created_at: None,
                updated_at: None,
            }));
        }
    };
    
    // Create a wallet for the customer
    let initial_balance = payload.initial_balance_cents.unwrap_or(0) as i32; // Convert i64 to i32
    let new_wallet = innosystem_common::models::wallet::NewWallet {
        id: Uuid::new_v4(),
        customer_id: customer.id,
        balance_cents: initial_balance,
    };
    
    let wallet = match state.wallet_repo.create(new_wallet).await {
        Ok(wallet) => wallet,
        Err(e) => {
            tracing::error!("Failed to create wallet for customer {}: {}", customer.id, e);
            // Continue with customer creation even if wallet creation fails
            return (StatusCode::CREATED, Json(CustomerResponse {
                id: customer.id,
                name: customer.name,
                email: customer.email,
                api_key: customer.api_key.clone(),
                reseller_id: customer.reseller_id,
                wallet_id: None,
                balance_cents: None,
                created_at: customer.created_at.map(|dt| dt.and_utc().to_rfc3339()),
                updated_at: customer.updated_at.map(|dt| dt.and_utc().to_rfc3339()),
            }));
        }
    };
    
    // Create the response
    let response = CustomerResponse {
        id: customer.id,
        name: customer.name,
        email: customer.email,
        api_key: customer.api_key.clone(),
        reseller_id: customer.reseller_id,
        wallet_id: Some(wallet.id),
        balance_cents: Some(wallet.balance_cents as i64), // Convert i32 to i64
        created_at: customer.created_at.map(|dt| dt.and_utc().to_rfc3339()),
        updated_at: customer.updated_at.map(|dt| dt.and_utc().to_rfc3339()),
    };
    
    tracing::info!("Created new customer with ID: {}", customer.id);
    (StatusCode::CREATED, Json(response))
}

/// Get a customer by ID
pub async fn get_customer(
    State(state): State<AppState>,
    Path(customer_id_str): Path<String>,
) -> Result<Json<CustomerResponse>, StatusCode> {
    // Try to parse the customer_id as a UUID
    let customer_id = match Uuid::parse_str(&customer_id_str) {
        Ok(id) => id,
        Err(_) => {
            tracing::error!("Invalid customer ID format: {}", customer_id_str);
            return Err(StatusCode::BAD_REQUEST);
        }
    };
    
    // Fetch the customer from the repository
    let customer = state.customer_repo.find_by_id(customer_id).await
        .map_err(|e| {
            tracing::error!("Failed to fetch customer: {}", e);
            // If customer not found, return 404
            if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;
    
    // Fetch the customer's wallet
    let wallet = state.wallet_repo.find_by_customer_id(customer.id).await;
    
    let (wallet_id, balance_cents) = match wallet {
        Ok(wallet) => (Some(wallet.id), Some(wallet.balance_cents)),
        Err(e) => {
            tracing::warn!("Failed to fetch wallet for customer {}: {}", customer.id, e);
            (None, None)
        }
    };
    
    // Create the response
    let response = CustomerResponse {
        id: customer.id,
        name: customer.name,
        email: customer.email,
        api_key: customer.api_key.clone(),
        reseller_id: customer.reseller_id,
        wallet_id,
        balance_cents: balance_cents.map(|b| b as i64), // Convert from i32 to i64
        created_at: customer.created_at.map(|dt| dt.and_utc().to_rfc3339()),
        updated_at: customer.updated_at.map(|dt| dt.and_utc().to_rfc3339()),
    };
    
    tracing::info!("Retrieved customer with ID: {}", customer.id);
    Ok(Json(response))
}

/// Get all customers
pub async fn get_all_customers(
    State(state): State<AppState>,
    Extension(api_key): Extension<String>,
) -> Result<Json<Vec<CustomerResponse>>, StatusCode> {
    // Determine if this is an admin or reseller request
    let customers = if api_key == state.config.admin_api_key {
        // Admin sees all customers
        state.customer_repo.list_all().await
            .map_err(|e| {
                error!("Failed to fetch customers: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?
    } else {
        // Reseller sees only their customers
        let reseller = state.reseller_repo.find_by_api_key(&api_key).await
            .map_err(|e| {
                error!("Failed to find reseller by API key: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
        
        state.customer_repo.find_by_reseller_id(reseller.id).await
            .map_err(|e| {
                error!("Failed to fetch customers for reseller {}: {}", reseller.id, e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?
    };
    
    // Convert to response format with wallet information where available
    let mut customer_responses = Vec::with_capacity(customers.len());
    
    for customer in customers {
        // Try to fetch the wallet for this customer, but don't fail if not found
        let wallet = state.wallet_repo.find_by_customer_id(customer.id).await;
        
        let (wallet_id, balance_cents) = match wallet {
            Ok(wallet) => (Some(wallet.id), Some(wallet.balance_cents)),
            Err(_) => (None, None),
        };
        
        customer_responses.push(CustomerResponse {
            id: customer.id,
            name: customer.name,
            email: customer.email,
            api_key: customer.api_key.clone(),
            reseller_id: customer.reseller_id,
            wallet_id,
            balance_cents: balance_cents.map(|b| b as i64), // Convert from i32 to i64
            created_at: customer.created_at.map(|dt| dt.and_utc().to_rfc3339()),
            updated_at: customer.updated_at.map(|dt| dt.and_utc().to_rfc3339()),
        });
    }
    
    tracing::info!("Retrieved all customers from database");
    Ok(Json(customer_responses))
}
