use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::pg::Pg;
use diesel::sql_types::Text;
use diesel::serialize::{self, Output, ToSql};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum JobStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
    Cancelled,
    Scheduled,
}

// Implement Queryable for JobStatus
impl Queryable<Text, Pg> for JobStatus {
    type Row = String;
    
    fn build(row: Self::Row) -> diesel::deserialize::Result<Self> {
        match JobStatus::from_str(&row) {
            Some(status) => Ok(status),
            None => {
                let error_message = format!("Unrecognized job status: {}", row);
                Err(Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, error_message)))
            }
        }
    }
}

// Implement ToSql for JobStatus (convert from Rust type to SQL type)
impl ToSql<Text, Pg> for JobStatus {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        // Since as_str() returns a static str, we can directly use it with to_sql
        match *self {
            JobStatus::Pending => ToSql::<Text, Pg>::to_sql("pending", out),
            JobStatus::Running => ToSql::<Text, Pg>::to_sql("running", out),
            JobStatus::Succeeded => ToSql::<Text, Pg>::to_sql("succeeded", out),
            JobStatus::Failed => ToSql::<Text, Pg>::to_sql("failed", out),
            JobStatus::Cancelled => ToSql::<Text, Pg>::to_sql("cancelled", out),
            JobStatus::Scheduled => ToSql::<Text, Pg>::to_sql("scheduled", out),
        }
    }
}

impl JobStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            JobStatus::Pending => "pending",
            JobStatus::Running => "running",
            JobStatus::Succeeded => "succeeded",
            JobStatus::Failed => "failed",
            JobStatus::Cancelled => "cancelled",
            JobStatus::Scheduled => "scheduled",
        }
    }
    
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "pending" => Some(JobStatus::Pending),
            "running" => Some(JobStatus::Running),
            "succeeded" => Some(JobStatus::Succeeded),
            "failed" => Some(JobStatus::Failed),
            "cancelled" => Some(JobStatus::Cancelled),
            "scheduled" => Some(JobStatus::Scheduled),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum PriorityLevel {
    Low = 0,
    Medium = 1,
    High = 2,
    Critical = 3,
}



impl PriorityLevel {
    pub fn as_i32(&self) -> i32 {
        match self {
            PriorityLevel::Low => 0,
            PriorityLevel::Medium => 1,
            PriorityLevel::High => 2, 
            PriorityLevel::Critical => 3,
        }
    }
    
    pub fn from_i32(value: i32) -> Self {
        match value {
            0 => PriorityLevel::Low,
            1 => PriorityLevel::Medium,
            2 => PriorityLevel::High,
            3 => PriorityLevel::Critical,
            _ => PriorityLevel::Low, // Default to Low for unknown values
        }
    }
}

// Database representation of a Job
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Identifiable, Selectable)]
#[diesel(table_name = crate::diesel_schema::jobs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct JobDb {
    pub id: Uuid,
    pub job_type_id: Uuid,
    pub customer_id: Uuid,
    pub status: String,  // Store as String in DB representation
    pub cost_cents: i32,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub completed_at: Option<NaiveDateTime>,
}

// Full Job model with all fields used in application logic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub job_type_id: Uuid,
    pub status: JobStatus,
    pub priority: PriorityLevel,
    pub input_data: serde_json::Value,
    pub output_data: Option<serde_json::Value>,
    pub error: Option<String>,
    pub estimated_cost_cents: i32,
    pub cost_cents: i32,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
    pub completed_at: Option<NaiveDateTime>,
}

// Conversion from database model to application model
impl From<JobDb> for Job {
    fn from(db_job: JobDb) -> Self {
        Job {
            id: db_job.id,
            customer_id: db_job.customer_id,
            job_type_id: db_job.job_type_id,
            status: JobStatus::from_str(&db_job.status).unwrap_or(JobStatus::Pending),
            priority: PriorityLevel::Medium, // Default value since not stored in DB
            input_data: serde_json::Value::Null, // Default value since not stored in DB
            output_data: None,
            error: None,
            estimated_cost_cents: db_job.cost_cents, // Use cost_cents as estimate
            cost_cents: db_job.cost_cents,
            created_at: db_job.created_at,
            updated_at: db_job.updated_at,
            completed_at: db_job.completed_at,
        }
    }
}

impl Job {
    pub fn new(
        customer_id: Uuid,
        job_type_id: Uuid,
        input_data: serde_json::Value,
        priority: PriorityLevel,
        estimated_cost_cents: i32,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            customer_id,
            job_type_id,
            status: JobStatus::Pending,
            priority,
            input_data,
            output_data: None,
            error: None,
            estimated_cost_cents,
            cost_cents: estimated_cost_cents,  // Initialize with estimated cost
            created_at: Some(chrono::Utc::now().naive_utc()),
            updated_at: None,
            completed_at: None,
        }
    }
}

// For DB insertion with Diesel
#[derive(Debug, Clone, Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::diesel_schema::jobs)]
pub struct NewJob {
    pub id: Uuid,
    pub job_type_id: Uuid,
    pub customer_id: Uuid,
    pub status: String,
    pub cost_cents: i32,
}

// Conversion from application model to database insert model
impl From<Job> for NewJob {
    fn from(job: Job) -> Self {
        NewJob {
            id: job.id,
            job_type_id: job.job_type_id,
            customer_id: job.customer_id,
            status: job.status.as_str().to_string(),
            cost_cents: job.cost_cents,
        }
    }
}
