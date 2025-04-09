use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::pg::Pg;
use diesel::serialize::{self, IsNull, Output, ToSql};
use diesel::deserialize::{self, FromSql};
use diesel::sql_types::Text;
use std::io::Write;

use crate::diesel_schema::{runners, runner_job_type_compatibility};
use crate::models::job_type::JobType;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RunnerStatus {
    Active,
    Inactive,
    Maintenance,
}

impl RunnerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            RunnerStatus::Active => "active",
            RunnerStatus::Inactive => "inactive",
            RunnerStatus::Maintenance => "maintenance",
        }
    }
    
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "active" => Some(RunnerStatus::Active),
            "inactive" => Some(RunnerStatus::Inactive),
            "maintenance" => Some(RunnerStatus::Maintenance),
            _ => None,
        }
    }
}

// Implement Queryable for RunnerStatus
impl Queryable<Text, Pg> for RunnerStatus {
    type Row = String;
    
    fn build(row: Self::Row) -> diesel::deserialize::Result<Self> {
        match RunnerStatus::from_str(&row) {
            Some(status) => Ok(status),
            None => {
                let error_message = format!("Unrecognized runner status: {}", row);
                Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, error_message)))
            }
        }
    }
}

// Implement ToSql for RunnerStatus
impl ToSql<Text, Pg> for RunnerStatus {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        let s = self.as_str();
        out.write_all(s.as_bytes())?;
        Ok(IsNull::No)
    }
}

// Implement FromSql for RunnerStatus
impl FromSql<Text, Pg> for RunnerStatus {
    fn from_sql(bytes: diesel::pg::PgValue) -> deserialize::Result<Self> {
        let string_value = <String as FromSql<Text, Pg>>::from_sql(bytes)?;
        match RunnerStatus::from_str(&string_value) {
            Some(status) => Ok(status),
            None => {
                let error_message = format!("Unrecognized runner status: {}", string_value);
                let io_error = std::io::Error::new(std::io::ErrorKind::InvalidData, error_message);
                Err(Box::new(io_error))
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable, Selectable)]
#[diesel(table_name = runners)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Runner {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub status: RunnerStatus,
    pub compatible_job_types: Vec<String>,
    pub last_heartbeat: Option<NaiveDateTime>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

impl Runner {
    pub fn new(
        name: String,
        description: Option<String>,
        compatible_job_types: Vec<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            status: RunnerStatus::Inactive,
            compatible_job_types,
            last_heartbeat: None,
            created_at: None,
            updated_at: None,
        }
    }
    
    pub fn update_heartbeat(&mut self, time: NaiveDateTime) {
        self.last_heartbeat = Some(time);
    }
    
    pub fn set_status(&mut self, status: RunnerStatus) {
        self.status = status;
    }
    
    pub fn add_compatible_job_type(&mut self, job_type: String) {
        if !self.compatible_job_types.contains(&job_type) {
            self.compatible_job_types.push(job_type);
        }
    }
    
    pub fn remove_compatible_job_type(&mut self, job_type: &str) {
        self.compatible_job_types.retain(|t| t != job_type);
    }
}

// For DB insertion with Diesel
#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = runners)]
pub struct NewRunner {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub compatible_job_types: Vec<String>,
}

impl From<Runner> for NewRunner {
    fn from(runner: Runner) -> Self {
        Self {
            id: runner.id,
            name: runner.name,
            description: runner.description,
            status: runner.status.as_str().to_string(),
            compatible_job_types: runner.compatible_job_types,
        }
    }
}

// For joining runners and job types
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable, Associations)]
#[diesel(belongs_to(Runner))]
#[diesel(belongs_to(JobType))]
#[diesel(table_name = runner_job_type_compatibility)]
#[diesel(primary_key(runner_id, job_type_id))]
pub struct JobTypeCompatibility {
    pub runner_id: Uuid,
    pub job_type_id: Uuid,
    pub created_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = runner_job_type_compatibility)]
pub struct NewJobTypeCompatibility {
    pub runner_id: Uuid,
    pub job_type_id: Uuid,
}
