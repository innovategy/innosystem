use async_trait::async_trait;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use uuid::Uuid;
use anyhow::{Result, anyhow};
use chrono::Utc;

use crate::diesel_schema::{wallets, wallet_transactions};
use crate::models::wallet::{Wallet, NewWallet, WalletTransaction, NewWalletTransaction, TransactionType};
use crate::repositories::WalletRepository;

/// Diesel-backed implementation of WalletRepository
pub struct DieselWalletRepository {
    pool: Pool<ConnectionManager<PgConnection>>,
}

impl DieselWalletRepository {
    pub fn new(pool: Pool<ConnectionManager<PgConnection>>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl WalletRepository for DieselWalletRepository {
    async fn create(&self, new_wallet: NewWallet) -> Result<Wallet> {
        let mut conn = self.pool.get()?;
        
        // Execute the insert and return the new record
        let wallet: Wallet = tokio::task::spawn_blocking(move || {
            let result = diesel::insert_into(wallets::table)
                .values(&new_wallet)
                .get_result::<Wallet>(&mut conn);
                
            match result {
                Ok(wallet) => Ok(wallet),
                Err(e) => Err(anyhow!("Failed to create wallet: {}", e))
            }
        }).await??;
        
        Ok(wallet)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Wallet> {
        let mut conn = self.pool.get()?;
        
        let wallet: Wallet = tokio::task::spawn_blocking(move || {
            wallets::table
                .find(id)
                .first(&mut conn)
                .optional()
        }).await??
            .ok_or_else(|| anyhow!("Wallet not found with ID: {}", id))?;
        
        Ok(wallet)
    }

    async fn find_by_customer_id(&self, customer_id: Uuid) -> Result<Wallet> {
        let mut conn = self.pool.get()?;
        
        let wallet: Wallet = tokio::task::spawn_blocking(move || {
            wallets::table
                .filter(wallets::customer_id.eq(customer_id))
                .first(&mut conn)
                .optional()
        }).await??
            .ok_or_else(|| anyhow!("Wallet not found for customer: {}", customer_id))?;
        
        Ok(wallet)
    }

    async fn update_balance(
        &self, 
        id: Uuid, 
        amount: i32, 
        transaction_type: TransactionType,
        description: Option<String>,
        job_id: Option<Uuid>
    ) -> Result<Wallet> {
        let mut conn = self.pool.get()?;
        
        // Create a transaction to ensure atomicity
        let (wallet, _) = tokio::task::spawn_blocking(move || -> Result<(Wallet, WalletTransaction)> {
            conn.transaction(|conn| {
                // First get the wallet to check available balance
                let wallet = wallets::table
                    .find(id)
                    .first::<Wallet>(conn)?;
                
                // Calculate new balance
                let new_balance = wallet.balance_cents + amount;
                
                // Create a transaction record
                let transaction = NewWalletTransaction {
                    id: Uuid::new_v4(),
                    wallet_id: id,
                    amount_cents: amount,
                    transaction_type: transaction_type.to_string(),
                    customer_id: wallet.customer_id,
                    reference_id: None,
                    description,
                    job_id,
                    created_at: None,
                };
                
                // Insert the transaction record
                let transaction_record = diesel::insert_into(wallet_transactions::table)
                    .values(&transaction)
                    .get_result::<WalletTransaction>(conn)?;
                
                // Update the wallet balance
                let updated_wallet = diesel::update(wallets::table.find(id))
                    .set((
                        wallets::balance_cents.eq(new_balance),
                        wallets::updated_at.eq(Utc::now().naive_utc()),
                    ))
                    .get_result::<Wallet>(conn)?;
                
                Ok((updated_wallet, transaction_record))
            })
        }).await??;
        
        Ok(wallet)
    }
    
    async fn deposit(
        &self,
        id: Uuid,
        amount: i32,
        description: Option<String>,
        job_id: Option<Uuid>
    ) -> Result<Wallet> {
        if amount <= 0 {
            return Err(anyhow!("Deposit amount must be positive"));
        }
        
        self.update_balance(
            id, 
            amount, 
            TransactionType::Deposit,
            description.or_else(|| Some(format!("Deposit of {} cents", amount))),
            job_id
        ).await
    }
    
    async fn withdraw(
        &self,
        id: Uuid,
        amount: i32,
        description: Option<String>,
        job_id: Option<Uuid>
    ) -> Result<Wallet> {
        if amount <= 0 {
            return Err(anyhow!("Withdrawal amount must be positive"));
        }
        
        // Check if there are sufficient funds
        let wallet = self.find_by_id(id).await?;
        if wallet.balance_cents < amount {
            return Err(anyhow!("Insufficient funds for withdrawal"));
        }
        
        self.update_balance(
            id, 
            -amount, // Negative for withdrawal
            TransactionType::Withdrawal,
            description.or_else(|| Some(format!("Withdrawal of {} cents", amount))),
            job_id
        ).await
    }

    async fn reserve_funds(
        &self, 
        id: Uuid, 
        amount: i32,
        description: Option<String>,
        job_id: Option<Uuid>
    ) -> Result<Wallet> {
        if amount <= 0 {
            return Err(anyhow!("Reservation amount must be positive"));
        }
        
        // Check if there are sufficient funds
        let wallet = self.find_by_id(id).await?;
        if wallet.balance_cents < amount {
            return Err(anyhow!("Insufficient funds for reservation"));
        }
        
        self.update_balance(
            id, 
            -amount, // Negative for reservation
            TransactionType::Reserved,
            description.or_else(|| Some(format!("Reservation of {} cents", amount))),
            job_id
        ).await
    }

    async fn release_reservation(
        &self, 
        id: Uuid, 
        amount: i32,
        description: Option<String>,
        job_id: Option<Uuid>
    ) -> Result<Wallet> {
        if amount <= 0 {
            return Err(anyhow!("Release amount must be positive"));
        }
        
        self.update_balance(
            id, 
            amount, // Positive for releasing
            TransactionType::Released,
            description.or_else(|| Some(format!("Release of reservation of {} cents", amount))),
            job_id
        ).await
    }

    async fn add_transaction(&self, new_transaction: NewWalletTransaction) -> Result<WalletTransaction> {
        let mut conn = self.pool.get()?;
        let wallet_id = new_transaction.wallet_id;
        
        // Use a transaction to ensure atomicity
        let (transaction, _) = tokio::task::spawn_blocking(move || -> Result<(WalletTransaction, Wallet)> {
            conn.transaction(|conn| {
                // First check if the wallet exists
                let wallet = wallets::table
                    .find(wallet_id)
                    .first::<Wallet>(conn)?;
                
                // Insert the transaction record
                let transaction_record = diesel::insert_into(wallet_transactions::table)
                    .values(&new_transaction)
                    .get_result::<WalletTransaction>(conn)?;
                
                // Update the wallet balance
                let new_balance = wallet.balance_cents + new_transaction.amount_cents;
                let updated_wallet = diesel::update(wallets::table.find(wallet_id))
                    .set((
                        wallets::balance_cents.eq(new_balance),
                        wallets::updated_at.eq(Utc::now().naive_utc()),
                    ))
                    .get_result::<Wallet>(conn)?;
                
                Ok((transaction_record, updated_wallet))
            })
        }).await??;
        
        Ok(transaction)
    }

    async fn get_transactions(&self, wallet_id: Uuid, limit: i32, offset: i32) -> Result<Vec<WalletTransaction>> {
        let mut conn = self.pool.get()?;
        
        let transactions = tokio::task::spawn_blocking(move || {
            let result = wallet_transactions::table
                .filter(wallet_transactions::wallet_id.eq(wallet_id))
                .order(wallet_transactions::created_at.desc())
                .limit(limit.into())
                .offset(offset.into())
                .load::<WalletTransaction>(&mut conn);
                
            match result {
                Ok(transactions) => Ok(transactions),
                Err(e) => Err(anyhow!("Failed to get transactions: {}", e))
            }
        }).await??;
        
        Ok(transactions)
    }
    
    async fn get_balance(&self, id: Uuid) -> Result<i32> {
        let wallet = self.find_by_id(id).await?;
        Ok(wallet.balance_cents)
    }
}
