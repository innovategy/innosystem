use async_trait::async_trait;
use uuid::Uuid;
use anyhow::Result;

use crate::models::wallet::{Wallet, NewWallet, WalletTransaction, NewWalletTransaction, TransactionType};

#[async_trait]
pub trait WalletRepository: Send + Sync {
    /// Create a new wallet for a customer
    async fn create(&self, new_wallet: NewWallet) -> Result<Wallet>;
    
    /// Find a wallet by its ID
    async fn find_by_id(&self, id: Uuid) -> Result<Wallet>;
    
    /// Find a wallet by customer ID
    async fn find_by_customer_id(&self, customer_id: Uuid) -> Result<Wallet>;
    
    /// Update wallet balance and create a transaction record
    async fn update_balance(
        &self, 
        id: Uuid, 
        amount: i32, 
        transaction_type: TransactionType,
        description: Option<String>,
        job_id: Option<Uuid>
    ) -> Result<Wallet>;
    
    /// Add funds to wallet and create a deposit transaction record
    async fn deposit(
        &self,
        id: Uuid,
        amount: i32,
        description: Option<String>,
        job_id: Option<Uuid>
    ) -> Result<Wallet>;
    
    /// Remove funds from wallet and create a withdrawal transaction record
    async fn withdraw(
        &self,
        id: Uuid,
        amount: i32,
        description: Option<String>,
        job_id: Option<Uuid>
    ) -> Result<Wallet>;
    
    /// Reserve funds for a pending transaction
    async fn reserve_funds(
        &self, 
        id: Uuid, 
        amount: i32,
        description: Option<String>,
        job_id: Option<Uuid>
    ) -> Result<Wallet>;
    
    /// Release previously reserved funds
    async fn release_reservation(
        &self, 
        id: Uuid, 
        amount: i32,
        description: Option<String>,
        job_id: Option<Uuid>
    ) -> Result<Wallet>;
    
    /// Add a transaction record to the wallet
    async fn add_transaction(&self, new_transaction: NewWalletTransaction) -> Result<WalletTransaction>;
    
    /// Get transaction history for a wallet with pagination
    async fn get_transactions(&self, wallet_id: Uuid, limit: i32, offset: i32) -> Result<Vec<WalletTransaction>>;
    
    /// Get the current balance of a wallet
    async fn get_balance(&self, id: Uuid) -> Result<i32>;
}
