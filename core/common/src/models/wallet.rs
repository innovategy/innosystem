use diesel::prelude::*;
use diesel::sql_types::Text;
use diesel::deserialize::{self, FromSql};
use diesel::serialize::{self, ToSql, Output};
use diesel::pg::{Pg, PgValue};
use diesel::{AsExpression, FromSqlRow};
use serde::{Deserialize, Serialize};
// Removed unused import
use std::io::Write;
use uuid::Uuid;
use chrono::NaiveDateTime;

use crate::diesel_schema::{wallets, wallet_transactions};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, AsExpression, FromSqlRow)]
#[diesel(sql_type = diesel::sql_types::Text)]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Reserved,
    Released,
    JobCredit,
    JobDebit,
    RefundCredit
}

impl TransactionType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "DEPOSIT" => Some(TransactionType::Deposit),
            "WITHDRAWAL" => Some(TransactionType::Withdrawal),
            "RESERVED" => Some(TransactionType::Reserved),
            "RELEASED" => Some(TransactionType::Released),
            "JOB_CREDIT" => Some(TransactionType::JobCredit),
            "JOB_DEBIT" => Some(TransactionType::JobDebit),
            "REFUND_CREDIT" => Some(TransactionType::RefundCredit),
            _ => None,
        }
    }
    
    pub fn as_str(&self) -> &'static str {
        match self {
            TransactionType::Deposit => "DEPOSIT",
            TransactionType::Withdrawal => "WITHDRAWAL",
            TransactionType::Reserved => "RESERVED",
            TransactionType::Released => "RELEASED",
            TransactionType::JobCredit => "JOB_CREDIT",
            TransactionType::JobDebit => "JOB_DEBIT",
            TransactionType::RefundCredit => "REFUND_CREDIT",
        }
    }
}

impl ToString for TransactionType {
    fn to_string(&self) -> String {
        self.as_str().to_string()
    }
}

impl TryFrom<String> for TransactionType {
    type Error = String;
    
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_str(&value).ok_or_else(|| format!("Invalid transaction type: {}", value))
    }
}

impl ToSql<Text, Pg> for TransactionType {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        let s = self.as_str();
        out.write_all(s.as_bytes())?;
        Ok(serialize::IsNull::No)
    }
}

impl FromSql<Text, Pg> for TransactionType {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        // Use explicit cast to get the string value from bytes
        let s = <String as FromSql<Text, Pg>>::from_sql(bytes)?;
        TransactionType::from_str(&s)
            .ok_or_else(|| format!("Unrecognized TransactionType variant: {}", s).into())
    }
}

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

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = wallet_transactions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct WalletTransaction {
    pub id: Uuid,
    pub wallet_id: Uuid,
    pub amount_cents: i32,
    pub transaction_type: String,
    pub customer_id: Uuid,
    pub reference_id: Option<Uuid>,
    pub description: Option<String>,
    pub job_id: Option<Uuid>,
    pub created_at: Option<NaiveDateTime>,
}

impl WalletTransaction {
    pub fn new(
        wallet_id: Uuid,
        amount_cents: i32,
        transaction_type: String,
        customer_id: Uuid,
        reference_id: Option<Uuid>,
        description: Option<String>,
        job_id: Option<Uuid>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            wallet_id,
            amount_cents,
            transaction_type,
            customer_id,
            reference_id,
            description,
            job_id,
            created_at: None,
        }
    }
    
    // Helper for job-related transactions
    pub fn for_job(
        wallet_id: Uuid,
        amount_cents: i32,
        transaction_type: String,
        customer_id: Uuid,
        job_id: Uuid,
        description: Option<String>,
    ) -> Self {
        Self::new(
            wallet_id,
            amount_cents,
            transaction_type,
            customer_id,
            None,
            description,
            Some(job_id),
        )
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
    pub customer_id: Uuid,
    pub reference_id: Option<Uuid>,
    pub description: Option<String>,
    pub job_id: Option<Uuid>,
    pub created_at: Option<NaiveDateTime>,
}
