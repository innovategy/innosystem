use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::NaiveDateTime;

use crate::diesel_schema::customers;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = customers)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Customer {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub reseller_id: Option<Uuid>,
    pub api_key: Option<String>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

impl Customer {
    pub fn new(name: String, email: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            email,
            reseller_id: None,
            api_key: None,
            created_at: None,
            updated_at: None,
        }
    }
    
    pub fn with_reseller(name: String, email: String, reseller_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            email,
            reseller_id: Some(reseller_id),
            api_key: None,
            created_at: None,
            updated_at: None,
        }
    }
    
    pub fn generate_api_key() -> String {
        format!("cus_{}", Uuid::new_v4().to_string().replace("-", ""))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name = customers)]
#[diesel(check_for_backend(diesel::pg::Pg))]  
pub struct NewCustomer {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub reseller_id: Option<Uuid>,
    pub api_key: Option<String>,
}
