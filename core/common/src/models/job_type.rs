use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::pg::Pg;
use diesel::serialize::{self, IsNull, Output, ToSql};
use diesel::deserialize::{self, FromSql};
use diesel::sql_types::Text;
use std::io::Write;

use crate::diesel_schema::job_types;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessorType {
    Sync,
    Async,
    ExternalApi,
    Batch,
    Webhook,
}



// Implement ToSql for ProcessorType (convert from Rust type to SQL type)
impl ToSql<Text, Pg> for ProcessorType {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        let s = self.as_str();
        out.write_all(s.as_bytes())?;
        Ok(IsNull::No)
    }
}

// Implement FromSql for ProcessorType (convert from SQL type to Rust type)
impl FromSql<Text, Pg> for ProcessorType {
    fn from_sql(bytes: diesel::pg::PgValue) -> deserialize::Result<Self> {
        let string_value = <String as FromSql<Text, Pg>>::from_sql(bytes)?;
        match ProcessorType::from_str(&string_value) {
            Some(processor_type) => Ok(processor_type),
            None => {
                let error_message = format!("Unrecognized processor type: {}", string_value);
                let io_error = std::io::Error::new(std::io::ErrorKind::InvalidData, error_message);
                Err(Box::new(io_error))
            }
        }
    }
}

impl ProcessorType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ProcessorType::Sync => "sync",
            ProcessorType::Async => "async",
            ProcessorType::ExternalApi => "external_api",
            ProcessorType::Batch => "batch",
            ProcessorType::Webhook => "webhook",
        }
    }
    
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "sync" => Some(ProcessorType::Sync),
            "async" => Some(ProcessorType::Async),
            "external_api" => Some(ProcessorType::ExternalApi),
            "batch" => Some(ProcessorType::Batch),
            "webhook" => Some(ProcessorType::Webhook),
            _ => None,
        }
    }
}

// Updated for Phase 2 with Diesel support
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable)]
#[diesel(table_name = job_types)]
pub struct JobType {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub processing_logic_id: String,
    pub processor_type: ProcessorType,
    pub standard_cost_cents: i32,
    pub enabled: bool,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

impl JobType {
    pub fn new(
        name: String,
        processing_logic_id: String,
        processor_type: ProcessorType,
        standard_cost_cents: i32,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            description: None,
            processing_logic_id,
            processor_type,
            standard_cost_cents,
            enabled: true,
            created_at: None,
            updated_at: None,
        }
    }
}

// For DB insertion with Diesel
#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = job_types)]
pub struct NewJobType {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub processing_logic_id: String,
    pub processor_type: String,
    pub standard_cost_cents: i32,
    pub enabled: bool,
}
