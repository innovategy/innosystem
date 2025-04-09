use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::NaiveDateTime;
use diesel::prelude::*;

use crate::diesel_schema::resellers;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable, Selectable)]
#[diesel(table_name = resellers)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Reseller {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub api_key: String,
    pub active: bool,
    /// Commission rate in basis points (1/100 of a percent)
    /// Example: 1000 = 10.00%, 2500 = 25.00%
    pub commission_rate: i32,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

impl Reseller {
    pub fn new(
        name: String,
        email: String,
        api_key: String,
        commission_rate: i32,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            email,
            api_key,
            active: true,
            commission_rate,
            created_at: None,
            updated_at: None,
        }
    }

    pub fn generate_api_key() -> String {
        format!("rs_{}", Uuid::new_v4().to_string().replace("-", ""))
    }
}

// For DB insertion with Diesel
#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = resellers)]
pub struct NewReseller {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub api_key: String,
    pub active: bool,
    pub commission_rate: i32,
}

impl From<Reseller> for NewReseller {
    fn from(reseller: Reseller) -> Self {
        Self {
            id: reseller.id,
            name: reseller.name,
            email: reseller.email,
            api_key: reseller.api_key,
            active: reseller.active,
            commission_rate: reseller.commission_rate,
        }
    }
}

impl Reseller {
    /// Get the commission rate as a percentage value
    pub fn commission_rate_percentage(&self) -> f64 {
        self.commission_rate as f64 / 100.0
    }
    
    /// Set the commission rate using a percentage value
    pub fn set_commission_rate_from_percentage(&mut self, percentage: f64) {
        self.commission_rate = (percentage * 100.0).round() as i32;
    }
}
