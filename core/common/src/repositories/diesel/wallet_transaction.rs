use async_trait::async_trait;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use uuid::Uuid;
use anyhow::{Result, anyhow};
use chrono::NaiveDateTime;

use crate::models::wallet::{WalletTransaction, NewWalletTransaction, TransactionType};
use crate::repositories::WalletTransactionRepository;
use crate::diesel_schema::wallet_transactions;

/// Diesel implementation of the WalletTransactionRepository
pub struct DieselWalletTransactionRepository {
    pool: Pool<ConnectionManager<PgConnection>>,
}

impl DieselWalletTransactionRepository {
    /// Create a new DieselWalletTransactionRepository with the given connection pool
    pub fn new(pool: Pool<ConnectionManager<PgConnection>>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl WalletTransactionRepository for DieselWalletTransactionRepository {
    async fn create(&self, transaction: NewWalletTransaction) -> Result<WalletTransaction> {
        let mut conn = self.pool.get()?;
        
        // Insert the new transaction
        let transaction: WalletTransaction = tokio::task::spawn_blocking(move || {
            diesel::insert_into(wallet_transactions::table)
                .values(&transaction)
                .get_result(&mut conn)
        }).await??;
        
        Ok(transaction)
    }
    
    async fn find_by_id(&self, id: Uuid) -> Result<WalletTransaction> {
        let mut conn = self.pool.get()?;
        
        let transaction: WalletTransaction = tokio::task::spawn_blocking(move || {
            wallet_transactions::table
                .find(id)
                .first(&mut conn)
                .optional()
        }).await??
            .ok_or_else(|| anyhow!("Wallet transaction not found with ID: {}", id))?;
        
        Ok(transaction)
    }
    
    async fn find_by_wallet_id(&self, wallet_id: Uuid) -> Result<Vec<WalletTransaction>> {
        let mut conn = self.pool.get()?;
        
        let transactions: Vec<WalletTransaction> = tokio::task::spawn_blocking(move || {
            wallet_transactions::table
                .filter(wallet_transactions::wallet_id.eq(wallet_id))
                .order(wallet_transactions::created_at.desc())
                .load::<WalletTransaction>(&mut conn)
        }).await??;
        
        Ok(transactions)
    }
    
    async fn find_by_customer_id(&self, customer_id: Uuid) -> Result<Vec<WalletTransaction>> {
        let mut conn = self.pool.get()?;
        
        let transactions: Vec<WalletTransaction> = tokio::task::spawn_blocking(move || {
            wallet_transactions::table
                .filter(wallet_transactions::customer_id.eq(customer_id))
                .order(wallet_transactions::created_at.desc())
                .load::<WalletTransaction>(&mut conn)
        }).await??;
        
        Ok(transactions)
    }
    
    async fn find_in_time_range(&self, start_time: NaiveDateTime, end_time: NaiveDateTime) -> Result<Vec<WalletTransaction>> {
        let mut conn = self.pool.get()?;
        
        let transactions: Vec<WalletTransaction> = tokio::task::spawn_blocking(move || {
            wallet_transactions::table
                .filter(wallet_transactions::created_at.ge(start_time))
                .filter(wallet_transactions::created_at.le(end_time))
                .order(wallet_transactions::created_at.desc())
                .load::<WalletTransaction>(&mut conn)
        }).await??;
        
        Ok(transactions)
    }
    
    async fn find_by_transaction_type(&self, transaction_type: TransactionType) -> Result<Vec<WalletTransaction>> {
        let mut conn = self.pool.get()?;
        let transaction_type_str = transaction_type.to_string();
        
        let transactions: Vec<WalletTransaction> = tokio::task::spawn_blocking(move || {
            wallet_transactions::table
                .filter(wallet_transactions::transaction_type.eq(transaction_type_str))
                .order(wallet_transactions::created_at.desc())
                .load::<WalletTransaction>(&mut conn)
        }).await??;
        
        Ok(transactions)
    }
    
    async fn find_by_job_id(&self, job_id: Option<Uuid>) -> Result<Vec<WalletTransaction>> {
        let mut conn = self.pool.get()?;
        
        let transactions: Vec<WalletTransaction> = tokio::task::spawn_blocking(move || {
            wallet_transactions::table
                .filter(wallet_transactions::job_id.eq(job_id))
                .order(wallet_transactions::created_at.desc())
                .load::<WalletTransaction>(&mut conn)
        }).await??;
        
        Ok(transactions)
    }
}
