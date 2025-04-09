use axum::{extract::{Path, State}, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tracing::{info, error};

use innosystem_common::models::wallet::WalletTransaction;
use crate::state::AppState;

/// Request for depositing funds to a wallet
#[derive(Debug, Deserialize)]
pub struct DepositRequest {
    /// Amount to deposit in cents
    pub amount: i32,
    /// Optional description
    pub description: Option<String>,
}

/// Request for withdrawing funds from a wallet
#[derive(Debug, Deserialize)]
pub struct WithdrawRequest {
    /// Amount to withdraw in cents
    pub amount: i32,
    /// Optional description
    pub description: Option<String>,
}

/// Response data for wallet operations
#[derive(Debug, Serialize)]
pub struct WalletResponse {
    /// Wallet ID
    pub id: Uuid,
    /// Customer ID
    pub customer_id: Uuid,
    /// Current balance in cents
    pub balance_cents: i32,
    /// Creation timestamp
    pub created_at: Option<String>,
    /// Last update timestamp
    pub updated_at: Option<String>,
}

/// Response data for wallet transaction operations
#[derive(Debug, Serialize)]
pub struct WalletTransactionResponse {
    /// Transaction ID
    pub id: Uuid,
    /// Wallet ID
    pub wallet_id: Uuid,
    /// Transaction type
    pub transaction_type: String,
    /// Amount in cents
    pub amount_cents: i32,
    /// Previous balance
    pub previous_balance_cents: i32,
    /// New balance
    pub new_balance_cents: i32,
    /// Description
    pub description: Option<String>,
    /// Related job ID if applicable
    pub job_id: Option<Uuid>,
    /// Creation timestamp
    pub created_at: Option<String>,
}

/// Get a wallet by customer ID
#[allow(dead_code)]
pub async fn get_wallet(
    State(state): State<AppState>,
    Path(customer_id_str): Path<String>,
) -> Result<Json<WalletResponse>, StatusCode> {
    // Try to parse the customer_id as a UUID
    let customer_id = match Uuid::parse_str(&customer_id_str) {
        Ok(id) => id,
        Err(_) => {
            error!("Invalid customer ID format: {}", customer_id_str);
            return Err(StatusCode::BAD_REQUEST);
        }
    };
    
    // Fetch the wallet from the repository
    let wallet = state.wallet_repo.find_by_customer_id(customer_id)
        .await
        .map_err(|e| {
            error!("Failed to fetch wallet: {}", e);
            // If wallet not found, return 404
            if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;
    
    // Convert the timestamps to RFC3339 strings if they exist
    let created_at = wallet.created_at.map(|dt| dt.and_utc().to_rfc3339());
    let updated_at = wallet.updated_at.map(|dt| dt.and_utc().to_rfc3339());
    
    // Create the response
    let response = WalletResponse {
        id: wallet.id,
        customer_id: wallet.customer_id,
        balance_cents: wallet.balance_cents,
        created_at,
        updated_at,
    };
    
    info!("Retrieved wallet for customer ID: {}", customer_id);
    Ok(Json(response))
}

/// Deposit funds to a wallet
#[allow(dead_code)]
pub async fn deposit_funds(
    State(state): State<AppState>,
    Path(customer_id_str): Path<String>,
    Json(payload): Json<DepositRequest>,
) -> Result<Json<WalletResponse>, StatusCode> {
    // Validate the amount
    if payload.amount <= 0 {
        error!("Invalid deposit amount: {}", payload.amount);
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // Try to parse the customer_id as a UUID
    let customer_id = match Uuid::parse_str(&customer_id_str) {
        Ok(id) => id,
        Err(_) => {
            error!("Invalid customer ID format: {}", customer_id_str);
            return Err(StatusCode::BAD_REQUEST);
        }
    };
    
    // Fetch the wallet from the repository
    let wallet = state.wallet_repo.find_by_customer_id(customer_id)
        .await
        .map_err(|e| {
            error!("Failed to fetch wallet: {}", e);
            if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;
    
    // Deposit funds to the wallet
    let updated_wallet = state.wallet_repo.deposit(
        wallet.id,
        payload.amount,
        payload.description,
        None, // No job ID for manual deposits
    )
    .await
    .map_err(|e| {
        error!("Failed to deposit funds: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    
    // Convert the timestamps to RFC3339 strings if they exist
    let created_at = updated_wallet.created_at.map(|dt| dt.and_utc().to_rfc3339());
    let updated_at = updated_wallet.updated_at.map(|dt| dt.and_utc().to_rfc3339());
    
    // Create the response
    let response = WalletResponse {
        id: updated_wallet.id,
        customer_id: updated_wallet.customer_id,
        balance_cents: updated_wallet.balance_cents,
        created_at,
        updated_at,
    };
    
    info!("Deposited {} cents to wallet for customer ID: {}", payload.amount, customer_id);
    Ok(Json(response))
}

/// Get wallet transactions with pagination
#[allow(dead_code)]
pub async fn get_transactions(
    State(state): State<AppState>,
    Path((customer_id_str, limit_str, offset_str)): Path<(String, String, String)>,
) -> Result<Json<Vec<WalletTransactionResponse>>, StatusCode> {
    // Try to parse the customer_id as a UUID
    let customer_id = match Uuid::parse_str(&customer_id_str) {
        Ok(id) => id,
        Err(_) => {
            error!("Invalid customer ID format: {}", customer_id_str);
            return Err(StatusCode::BAD_REQUEST);
        }
    };
    
    // Parse limit and offset
    let limit = limit_str.parse::<i32>().unwrap_or(10);
    let offset = offset_str.parse::<i32>().unwrap_or(0);
    
    // Fetch the wallet from the repository
    let wallet = state.wallet_repo.find_by_customer_id(customer_id)
        .await
        .map_err(|e| {
            error!("Failed to fetch wallet: {}", e);
            if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;
    
    // Get transactions for the wallet
    let transactions = state.wallet_repo.get_transactions(wallet.id, limit, offset)
        .await
        .map_err(|e| {
            error!("Failed to fetch transactions: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    // Convert the transactions to the response format
    let transaction_responses: Vec<WalletTransactionResponse> = transactions.into_iter().map(|tx| {
        // Convert the timestamp to RFC3339 string if it exists
        let created_at = tx.created_at.map(|dt| dt.and_utc().to_rfc3339());
        
        // Transaction type is already a string in the model
        let transaction_type = tx.transaction_type;
        
        WalletTransactionResponse {
            id: tx.id,
            wallet_id: tx.wallet_id,
            transaction_type,
            amount_cents: tx.amount_cents,
            previous_balance_cents: 0, // Not stored in WalletTransaction
            new_balance_cents: 0,      // Not stored in WalletTransaction
            description: tx.description,
            job_id: tx.job_id,
            created_at,
        }
    }).collect();
    
    info!("Retrieved {} transactions for customer ID: {}", transaction_responses.len(), customer_id);
    Ok(Json(transaction_responses))
}

/// Get job-related transactions
#[allow(dead_code)]
pub async fn get_job_transactions(
    State(state): State<AppState>,
    Path(job_id_str): Path<String>,
) -> Result<Json<Vec<WalletTransactionResponse>>, StatusCode> {
    // Try to parse the job_id as a UUID
    let job_id = match Uuid::parse_str(&job_id_str) {
        Ok(id) => id,
        Err(_) => {
            error!("Invalid job ID format: {}", job_id_str);
            return Err(StatusCode::BAD_REQUEST);
        }
    };
    
    // Fetch the job to get the customer ID
    let job = state.job_repo.find_by_id(job_id)
        .await
        .map_err(|e| {
            error!("Failed to fetch job: {}", e);
            if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;
    
    // Fetch the wallet from the repository
    let wallet = state.wallet_repo.find_by_customer_id(job.customer_id)
        .await
        .map_err(|e| {
            error!("Failed to fetch wallet: {}", e);
            if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;
    
    // Get all transactions for the wallet
    let all_transactions = state.wallet_repo.get_transactions(wallet.id, 100, 0)
        .await
        .map_err(|e| {
            error!("Failed to fetch transactions: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    // Filter transactions for the specific job
    let job_transactions: Vec<WalletTransaction> = all_transactions
        .into_iter()
        .filter(|tx| tx.job_id == Some(job_id))
        .collect();
    
    // Convert the transactions to the response format
    let transaction_responses: Vec<WalletTransactionResponse> = job_transactions.into_iter().map(|tx| {
        // Convert the timestamp to RFC3339 string if it exists
        let created_at = tx.created_at.map(|dt| dt.and_utc().to_rfc3339());
        
        // Transaction type is already a string in the model
        let transaction_type = tx.transaction_type;
        
        WalletTransactionResponse {
            id: tx.id,
            wallet_id: tx.wallet_id,
            transaction_type,
            amount_cents: tx.amount_cents,
            previous_balance_cents: 0, // Not stored in WalletTransaction
            new_balance_cents: 0,      // Not stored in WalletTransaction
            description: tx.description,
            job_id: tx.job_id,
            created_at,
        }
    }).collect();
    
    info!("Retrieved {} job-related transactions for job ID: {}", transaction_responses.len(), job_id);
    Ok(Json(transaction_responses))
}
