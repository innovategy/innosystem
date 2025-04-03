use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::NaiveDateTime;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessorType {
    Sync,
    Async,
    ExternalApi,
    Batch,
    Webhook,
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

// In-memory version for Phase 1
#[derive(Debug, Clone, Serialize, Deserialize)]
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

// Will be used in Phase 2 for DB insertion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewJobType {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub processing_logic_id: String,
    pub processor_type: String,
    pub standard_cost_cents: i32,
    pub enabled: bool,
}
