use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use async_trait::async_trait;
use uuid::Uuid;

use crate::errors::Error;
use crate::models::wallet::{Wallet, NewWallet, WalletTransaction, NewWalletTransaction};
use crate::repositories::WalletRepository;
use crate::Result;

/// In-memory implementation of WalletRepository for Phase 1
pub struct InMemoryWalletRepository {
    wallets: Arc<Mutex<HashMap<Uuid, Wallet>>>,
    customer_wallets: Arc<Mutex<HashMap<Uuid, Uuid>>>,
    transactions: Arc<Mutex<HashMap<Uuid, WalletTransaction>>>,
    wallet_transactions: Arc<Mutex<HashMap<Uuid, Vec<Uuid>>>>,
}

impl InMemoryWalletRepository {
    pub fn new() -> Self {
        Self {
            wallets: Arc::new(Mutex::new(HashMap::new())),
            customer_wallets: Arc::new(Mutex::new(HashMap::new())),
            transactions: Arc::new(Mutex::new(HashMap::new())),
            wallet_transactions: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl WalletRepository for InMemoryWalletRepository {
    async fn create(&self, new_wallet: NewWallet) -> Result<Wallet> {
        let wallet = Wallet {
            id: new_wallet.id,
            customer_id: new_wallet.customer_id,
            balance_cents: new_wallet.balance_cents,
            pending_charges_cents: new_wallet.pending_charges_cents,
            currency: new_wallet.currency,
            created_at: Some(chrono::Utc::now().naive_utc()),
            updated_at: Some(chrono::Utc::now().naive_utc()),
        };
        
        let mut wallets = self.wallets.lock().map_err(|_| Error::Other(anyhow::anyhow!("Lock error")))?;
        wallets.insert(wallet.id, wallet.clone());
        
        let mut customer_wallets = self.customer_wallets.lock().map_err(|_| Error::Other(anyhow::anyhow!("Lock error")))?;
        customer_wallets.insert(wallet.customer_id, wallet.id);
        
        let mut wallet_transactions = self.wallet_transactions.lock().map_err(|_| Error::Other(anyhow::anyhow!("Lock error")))?;
        wallet_transactions.insert(wallet.id, Vec::new());
        
        Ok(wallet)
    }
    
    async fn find_by_id(&self, id: Uuid) -> Result<Wallet> {
        let wallets = self.wallets.lock().map_err(|_| Error::Other(anyhow::anyhow!("Lock error")))?;
        
        wallets.get(&id)
            .cloned()
            .ok_or_else(|| Error::NotFound(format!("Wallet not found: {}", id)))
    }
    
    async fn find_by_customer_id(&self, customer_id: Uuid) -> Result<Wallet> {
        // Get wallet ID from customer wallets map, but drop the lock before the await
        let wallet_id = {
            let customer_wallets = self.customer_wallets.lock().map_err(|_| Error::Other(anyhow::anyhow!("Lock error")))?;
            
            customer_wallets.get(&customer_id)
                .cloned()
                .ok_or_else(|| Error::NotFound(format!("Wallet not found for customer: {}", customer_id)))?
        };
            
        // Now find the wallet with the ID (no lock held across await)
        self.find_by_id(wallet_id).await
    }
    
    async fn update_balance(&self, id: Uuid, new_balance: i64) -> Result<Wallet> {
        let mut wallets = self.wallets.lock().map_err(|_| Error::Other(anyhow::anyhow!("Lock error")))?;
        
        let wallet = wallets.get_mut(&id)
            .ok_or_else(|| Error::NotFound(format!("Wallet not found: {}", id)))?;
            
        wallet.balance_cents = new_balance;
        wallet.updated_at = Some(chrono::Utc::now().naive_utc());
        
        Ok(wallet.clone())
    }
    
    async fn reserve_funds(&self, id: Uuid, amount: i64) -> Result<Wallet> {
        let mut wallets = self.wallets.lock().map_err(|_| Error::Other(anyhow::anyhow!("Lock error")))?;
        
        let wallet = wallets.get_mut(&id)
            .ok_or_else(|| Error::NotFound(format!("Wallet not found: {}", id)))?;
            
        if wallet.available_balance() < amount {
            return Err(Error::InsufficientFunds(format!("Insufficient funds. Available: {}, Requested: {}", wallet.available_balance(), amount)));
        }
        
        wallet.pending_charges_cents += amount;
        wallet.updated_at = Some(chrono::Utc::now().naive_utc());
        
        Ok(wallet.clone())
    }
    
    async fn release_reservation(&self, id: Uuid, amount: i64) -> Result<Wallet> {
        let mut wallets = self.wallets.lock().map_err(|_| Error::Other(anyhow::anyhow!("Lock error")))?;
        
        let wallet = wallets.get_mut(&id)
            .ok_or_else(|| Error::NotFound(format!("Wallet not found: {}", id)))?;
            
        wallet.pending_charges_cents -= amount.min(wallet.pending_charges_cents);
        wallet.updated_at = Some(chrono::Utc::now().naive_utc());
        
        Ok(wallet.clone())
    }
    
    async fn add_transaction(&self, new_transaction: NewWalletTransaction) -> Result<WalletTransaction> {
        let transaction = WalletTransaction {
            id: new_transaction.id,
            wallet_id: new_transaction.wallet_id,
            customer_id: new_transaction.customer_id,
            amount_cents: new_transaction.amount_cents,
            job_id: new_transaction.job_id,
            description: new_transaction.description,
            created_at: Some(chrono::Utc::now().naive_utc()),
        };
        
        // Validate wallet exists
        self.find_by_id(transaction.wallet_id).await?;
        
        // Update wallet balance based on transaction
        let mut wallets = self.wallets.lock().map_err(|_| Error::Other(anyhow::anyhow!("Lock error")))?;
        
        let wallet = wallets.get_mut(&transaction.wallet_id)
            .ok_or_else(|| Error::NotFound(format!("Wallet not found: {}", transaction.wallet_id)))?;
            
        wallet.balance_cents += transaction.amount_cents;
        wallet.updated_at = Some(chrono::Utc::now().naive_utc());
        
        // Store transaction
        let mut transactions = self.transactions.lock().map_err(|_| Error::Other(anyhow::anyhow!("Lock error")))?;
        transactions.insert(transaction.id, transaction.clone());
        
        // Associate transaction with wallet
        let mut wallet_transactions = self.wallet_transactions.lock().map_err(|_| Error::Other(anyhow::anyhow!("Lock error")))?;
        
        if let Some(txs) = wallet_transactions.get_mut(&transaction.wallet_id) {
            txs.push(transaction.id);
        } else {
            wallet_transactions.insert(transaction.wallet_id, vec![transaction.id]);
        }
        
        Ok(transaction)
    }
    
    async fn get_transactions(&self, wallet_id: Uuid, limit: i32, offset: i32) -> Result<Vec<WalletTransaction>> {
        // Check if wallet exists
        self.find_by_id(wallet_id).await?;
        
        let wallet_transactions = self.wallet_transactions.lock().map_err(|_| Error::Other(anyhow::anyhow!("Lock error")))?;
        let transactions = self.transactions.lock().map_err(|_| Error::Other(anyhow::anyhow!("Lock error")))?;
        
        let transaction_ids = wallet_transactions.get(&wallet_id)
            .cloned()
            .unwrap_or_default();
            
        let result: Vec<WalletTransaction> = transaction_ids.iter()
            .skip(offset as usize)
            .take(limit as usize)
            .filter_map(|id| transactions.get(id).cloned())
            .collect();
            
        Ok(result)
    }
}
