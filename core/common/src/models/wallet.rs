use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::NaiveDateTime;

// In-memory version for Phase 1
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub balance_cents: i64,
    pub pending_charges_cents: i64,
    pub currency: String,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

impl Wallet {
    pub fn new(customer_id: Uuid, initial_balance_cents: i64) -> Self {
        Self {
            id: Uuid::new_v4(),
            customer_id,
            balance_cents: initial_balance_cents,
            pending_charges_cents: 0,
            currency: "EUR".to_string(),
            created_at: None,
            updated_at: None,
        }
    }

    pub fn available_balance(&self) -> i64 {
        self.balance_cents - self.pending_charges_cents
    }
}

// Will be used in Phase 2 for DB insertion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewWallet {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub balance_cents: i64,
    pub pending_charges_cents: i64,
    pub currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletTransaction {
    pub id: Uuid,
    pub wallet_id: Uuid,
    pub customer_id: Uuid,
    pub amount_cents: i64,
    pub job_id: Option<Uuid>,
    pub description: String,
    pub created_at: Option<NaiveDateTime>,
}

impl WalletTransaction {
    pub fn new(
        wallet_id: Uuid,
        customer_id: Uuid,
        amount_cents: i64,
        job_id: Option<Uuid>,
        description: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            wallet_id,
            customer_id,
            amount_cents,
            job_id,
            description,
            created_at: None,
        }
    }
}

// Will be used in Phase 2 for DB insertion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewWalletTransaction {
    pub id: Uuid,
    pub wallet_id: Uuid,
    pub customer_id: Uuid,
    pub amount_cents: i64,
    pub job_id: Option<Uuid>,
    pub description: String,
}
