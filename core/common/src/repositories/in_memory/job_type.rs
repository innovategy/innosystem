use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use async_trait::async_trait;
use uuid::Uuid;

use crate::errors::Error;
use crate::models::job_type::{JobType, NewJobType, ProcessorType};
use crate::repositories::JobTypeRepository;
use crate::Result;

/// In-memory implementation of JobTypeRepository for Phase 1
pub struct InMemoryJobTypeRepository {
    job_types: Arc<Mutex<HashMap<Uuid, JobType>>>,
}

impl InMemoryJobTypeRepository {
    pub fn new() -> Self {
        Self {
            job_types: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl JobTypeRepository for InMemoryJobTypeRepository {
    async fn create(&self, new_job_type: NewJobType) -> Result<JobType> {
        let processor_type = ProcessorType::from_str(&new_job_type.processor_type)
            .ok_or_else(|| Error::InvalidInput(format!("Invalid processor type: {}", new_job_type.processor_type)))?;
            
        let job_type = JobType {
            id: new_job_type.id,
            name: new_job_type.name,
            description: new_job_type.description,
            processing_logic_id: new_job_type.processing_logic_id,
            processor_type,
            standard_cost_cents: new_job_type.standard_cost_cents,
            enabled: new_job_type.enabled,
            created_at: Some(chrono::Utc::now().naive_utc()),
            updated_at: Some(chrono::Utc::now().naive_utc()),
        };
        
        let mut job_types = self.job_types.lock().map_err(|_| Error::Other(anyhow::anyhow!("Lock error")))?;
        job_types.insert(job_type.id, job_type.clone());
        
        Ok(job_type)
    }
    
    async fn find_by_id(&self, id: Uuid) -> Result<JobType> {
        let job_types = self.job_types.lock().map_err(|_| Error::Other(anyhow::anyhow!("Lock error")))?;
        
        job_types.get(&id)
            .cloned()
            .ok_or_else(|| Error::NotFound(format!("JobType not found: {}", id)))
    }
    
    async fn update(&self, job_type: JobType) -> Result<JobType> {
        let mut job_types = self.job_types.lock().map_err(|_| Error::Other(anyhow::anyhow!("Lock error")))?;
        
        if !job_types.contains_key(&job_type.id) {
            return Err(Error::NotFound(format!("JobType not found: {}", job_type.id)));
        }
        
        let updated_job_type = JobType {
            updated_at: Some(chrono::Utc::now().naive_utc()),
            ..job_type
        };
        
        job_types.insert(updated_job_type.id, updated_job_type.clone());
        
        Ok(updated_job_type)
    }
    
    async fn list_all(&self) -> Result<Vec<JobType>> {
        let job_types = self.job_types.lock().map_err(|_| Error::Other(anyhow::anyhow!("Lock error")))?;
        
        Ok(job_types.values().cloned().collect())
    }
    
    async fn list_enabled(&self) -> Result<Vec<JobType>> {
        let job_types = self.job_types.lock().map_err(|_| Error::Other(anyhow::anyhow!("Lock error")))?;
        
        Ok(job_types.values()
            .filter(|job_type| job_type.enabled)
            .cloned()
            .collect())
    }
}
