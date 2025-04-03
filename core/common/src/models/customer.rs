use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::NaiveDateTime;

// In-memory version for Phase 1
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Customer {
    pub id: Uuid,
    pub name: String,
    pub email: Option<String>,
    pub api_key: Option<String>,
    pub active: bool,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

impl Customer {
    pub fn new(name: String, email: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            email,
            api_key: None,
            active: true,
            created_at: None,
            updated_at: None,
        }
    }
}

// Will be used in Phase 2 for DB insertion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewCustomer {
    pub id: Uuid,
    pub name: String,
    pub email: Option<String>,
    pub api_key: Option<String>,
    pub active: bool,
}
