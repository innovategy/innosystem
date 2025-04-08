use async_trait::async_trait;
use diesel::prelude::*;
use uuid::Uuid;

use crate::database::{PgPool, get_connection};
use crate::diesel_schema::{wallets, wallet_transactions};
use crate::errors::Error;
use crate::models::wallet::{Wallet, NewWallet, WalletTransaction, NewWalletTransaction};
use crate::repositories::WalletRepository;
use crate::Result;

/// Diesel-backed implementation of WalletRepository
pub struct DieselWalletRepository {
    pool: PgPool,
}

impl DieselWalletRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl WalletRepository for DieselWalletRepository {
    async fn create(&self, new_wallet: NewWallet) -> Result<Wallet> {
        let mut conn = get_connection(&self.pool)?;
        
        // Execute the insert and return the new record
        diesel::insert_into(wallets::table)
            .values(&new_wallet)
            .returning(Wallet::as_select())
            .get_result(&mut conn)
            .map_err(|e| Error::Database(e))
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Wallet> {
        let mut conn = get_connection(&self.pool)?;
        
        wallets::table
            .find(id)
            .select(Wallet::as_select())
            .first(&mut conn)
            .map_err(|e| match e {
                diesel::result::Error::NotFound => Error::NotFound(format!("Wallet not found: {}", id)),
                e => Error::Database(e),
            })
    }

    async fn find_by_customer_id(&self, customer_id: Uuid) -> Result<Wallet> {
        let mut conn = get_connection(&self.pool)?;
        
        wallets::table
            .filter(wallets::customer_id.eq(customer_id))
            .select(Wallet::as_select())
            .first(&mut conn)
            .map_err(|e| match e {
                diesel::result::Error::NotFound => Error::NotFound(format!("Wallet not found for customer: {}", customer_id)),
                e => Error::Database(e),
            })
    }

    async fn update_balance(&self, id: Uuid, new_balance: i32) -> Result<Wallet> {
        let mut conn = get_connection(&self.pool)?;
        
        // First check if the entity exists
        let _wallet = wallets::table
            .find(id)
            .select(Wallet::as_select())
            .first(&mut conn)
            .map_err(|e| match e {
                diesel::result::Error::NotFound => Error::NotFound(format!("Wallet not found: {}", id)),
                e => Error::Database(e),
            })?;
            
        // Update the balance
        diesel::update(wallets::table)
            .filter(wallets::id.eq(id))
            .set((
                wallets::balance_cents.eq(new_balance),
                wallets::updated_at.eq(diesel::dsl::now),
            ))
            .returning(Wallet::as_select())
            .get_result(&mut conn)
            .map_err(|e| Error::Database(e))
    }

    async fn reserve_funds(&self, id: Uuid, amount: i32) -> Result<Wallet> {
        let mut conn = get_connection(&self.pool)?;
        
        // First get the wallet to check available balance
        let wallet = wallets::table
            .find(id)
            .select(Wallet::as_select())
            .first(&mut conn)
            .map_err(|e| match e {
                diesel::result::Error::NotFound => Error::NotFound(format!("Wallet not found: {}", id)),
                e => Error::Database(e),
            })?;
        
        // Ensure there are enough funds available
        if wallet.balance_cents < amount {
            return Err(Error::InsufficientFunds(format!(
                "Insufficient funds available in wallet {}: needed {}, available {}", 
                id, amount, wallet.balance_cents
            )));
        }
        
        // Create a reservation record via a transaction
        let transaction = NewWalletTransaction {
            id: Uuid::new_v4(),
            wallet_id: id,
            amount_cents: -amount, // Negative amount for reservation
            transaction_type: "RESERVE".to_string(),
            reference_id: None,
        };
        
        self.add_transaction(transaction).await?;
        
        // Return updated wallet
        self.find_by_id(id).await
    }

    async fn release_reservation(&self, id: Uuid, amount: i32) -> Result<Wallet> {
        // Create a release transaction
        let transaction = NewWalletTransaction {
            id: Uuid::new_v4(),
            wallet_id: id,
            amount_cents: amount, // Positive amount for releasing reservation
            transaction_type: "RELEASE".to_string(),
            reference_id: None,
        };
        
        self.add_transaction(transaction).await?;
        
        // Return updated wallet
        self.find_by_id(id).await
    }

    async fn add_transaction(&self, new_transaction: NewWalletTransaction) -> Result<WalletTransaction> {
        let mut conn = get_connection(&self.pool)?;
        
        // First check if the wallet exists
        let wallet = wallets::table
            .find(new_transaction.wallet_id)
            .select(Wallet::as_select())
            .first(&mut conn)
            .map_err(|e| match e {
                diesel::result::Error::NotFound => Error::NotFound(format!("Wallet not found: {}", new_transaction.wallet_id)),
                e => Error::Database(e),
            })?;
        
        // Use Diesel's connection.transaction method to handle transactions
        let wallet_transaction = conn.transaction::<_, Error, _>(|conn| {
            // Insert the transaction record
            let wallet_transaction = diesel::insert_into(wallet_transactions::table)
                .values(&new_transaction)
                .returning(WalletTransaction::as_select())
                .get_result(conn)
                .map_err(|e| Error::Database(e))?;
            
            // Update the wallet balance
            let new_balance = wallet.balance_cents + new_transaction.amount_cents;
            diesel::update(wallets::table)
                .filter(wallets::id.eq(wallet.id))
                .set((
                    wallets::balance_cents.eq(new_balance),
                    wallets::updated_at.eq(diesel::dsl::now),
                ))
                .execute(conn)
                .map_err(|e| Error::Database(e))?;
            
            Ok(wallet_transaction)
        })?;
            
        Ok(wallet_transaction)
    }

    async fn get_transactions(&self, wallet_id: Uuid, limit: i32, offset: i32) -> Result<Vec<WalletTransaction>> {
        let mut conn = get_connection(&self.pool)?;
        
        wallet_transactions::table
            .filter(wallet_transactions::wallet_id.eq(wallet_id))
            .order(wallet_transactions::created_at.desc())
            .limit(limit.into())
            .offset(offset.into())
            .select(WalletTransaction::as_select())
            .load(&mut conn)
            .map_err(|e| Error::Database(e))
    }
}
