use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::NaiveDateTime;

use crate::diesel_schema::{wallets, wallet_transactions};

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Wallet {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub balance_cents: i32,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

impl Wallet {
    pub fn new(customer_id: Uuid, initial_balance_cents: i32) -> Self {
        Self {
            id: Uuid::new_v4(),
            customer_id,
            balance_cents: initial_balance_cents,
            created_at: None,
            updated_at: None,
        }
    }

    pub fn available_balance(&self) -> i32 {
        self.balance_cents
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = wallets)]
#[diesel(check_for_backend(diesel::pg::Pg))]  
pub struct NewWallet {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub balance_cents: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct WalletTransaction {
    pub id: Uuid,
    pub wallet_id: Uuid,
    pub amount_cents: i32,
    pub transaction_type: String,
    pub reference_id: Option<Uuid>,
    pub created_at: Option<NaiveDateTime>,
}

impl WalletTransaction {
    pub fn new(
        wallet_id: Uuid,
        amount_cents: i32,
        transaction_type: String,
        reference_id: Option<Uuid>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            wallet_id,
            amount_cents,
            transaction_type,
            reference_id,
            created_at: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = wallet_transactions)]
#[diesel(check_for_backend(diesel::pg::Pg))]  
pub struct NewWalletTransaction {
    pub id: Uuid,
    pub wallet_id: Uuid,
    pub amount_cents: i32,
    pub transaction_type: String,
    pub reference_id: Option<Uuid>,
}
