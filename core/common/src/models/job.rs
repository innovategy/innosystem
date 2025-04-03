use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::NaiveDateTime;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum JobStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
    Cancelled,
    Scheduled,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

// In-memory version for Phase 1
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
    pub cost_cents: Option<i32>,
    pub created_at: Option<NaiveDateTime>,
    pub started_at: Option<NaiveDateTime>,
    pub completed_at: Option<NaiveDateTime>,
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
            cost_cents: None,
            created_at: None,
            started_at: None,
            completed_at: None,
        }
    }
}

// Will be used in Phase 2 for DB insertion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewJob {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub job_type_id: Uuid,
    pub status: String,
    pub priority: i32,
    pub input_data: serde_json::Value,
    pub estimated_cost_cents: i32,
}
