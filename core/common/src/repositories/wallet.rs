use async_trait::async_trait;
use uuid::Uuid;

use crate::models::wallet::{Wallet, NewWallet, WalletTransaction, NewWalletTransaction};
use crate::Result;

#[async_trait]
pub trait WalletRepository: Send + Sync {
    async fn create(&self, new_wallet: NewWallet) -> Result<Wallet>;
    async fn find_by_id(&self, id: Uuid) -> Result<Wallet>;
    async fn find_by_customer_id(&self, customer_id: Uuid) -> Result<Wallet>;
    async fn update_balance(&self, id: Uuid, new_balance: i32) -> Result<Wallet>;
    async fn reserve_funds(&self, id: Uuid, amount: i32) -> Result<Wallet>;
    async fn release_reservation(&self, id: Uuid, amount: i32) -> Result<Wallet>;
    async fn add_transaction(&self, new_transaction: NewWalletTransaction) -> Result<WalletTransaction>;
    async fn get_transactions(&self, wallet_id: Uuid, limit: i32, offset: i32) -> Result<Vec<WalletTransaction>>;
}
