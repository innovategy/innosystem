use async_trait::async_trait;
use uuid::Uuid;
use anyhow::Result;
use chrono::NaiveDateTime;

use crate::models::wallet::{WalletTransaction, NewWalletTransaction, TransactionType};

/// Repository trait for Wallet Transaction operations
#[async_trait]
pub trait WalletTransactionRepository: Send + Sync {
    /// Create a new wallet transaction
    async fn create(&self, transaction: NewWalletTransaction) -> Result<WalletTransaction>;
    
    /// Find a wallet transaction by ID
    async fn find_by_id(&self, id: Uuid) -> Result<WalletTransaction>;
    
    /// Get transactions for a specific wallet
    async fn find_by_wallet_id(&self, wallet_id: Uuid) -> Result<Vec<WalletTransaction>>;
    
    /// Get transactions for a specific customer
    async fn find_by_customer_id(&self, customer_id: Uuid) -> Result<Vec<WalletTransaction>>;
    
    /// Get transactions within a time range
    async fn find_in_time_range(&self, start_time: NaiveDateTime, end_time: NaiveDateTime) -> Result<Vec<WalletTransaction>>;
    
    /// Get transactions by transaction type
    async fn find_by_transaction_type(&self, transaction_type: TransactionType) -> Result<Vec<WalletTransaction>>;
    
    /// Get transactions for a specific job
    async fn find_by_job_id(&self, job_id: Option<Uuid>) -> Result<Vec<WalletTransaction>>;
}
