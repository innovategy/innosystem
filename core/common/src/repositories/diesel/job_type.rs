use async_trait::async_trait;
use diesel::prelude::*;
use uuid::Uuid;

use crate::database::{PgPool, get_connection};
use crate::diesel_schema::job_types;
use crate::errors::Error;
use crate::models::job_type::{JobType, NewJobType};
use crate::repositories::JobTypeRepository;
use crate::Result;

/// Diesel-backed implementation of JobTypeRepository
pub struct DieselJobTypeRepository {
    pool: PgPool,
}

impl DieselJobTypeRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl JobTypeRepository for DieselJobTypeRepository {
    async fn create(&self, new_job_type: NewJobType) -> Result<JobType> {
        let mut conn = get_connection(&self.pool)?;
        
        // Execute the insert and return the new record
        diesel::insert_into(job_types::table)
            .values(&new_job_type)
            .returning(JobType::as_select())
            .get_result(&mut conn)
            .map_err(|e| Error::Database(e))
    }

    async fn find_by_id(&self, id: Uuid) -> Result<JobType> {
        let mut conn = get_connection(&self.pool)?;
        
        job_types::table
            .find(id)
            .select(JobType::as_select())
            .first(&mut conn)
            .map_err(|e| match e {
                diesel::result::Error::NotFound => Error::NotFound(format!("JobType not found: {}", id)),
                e => Error::Database(e),
            })
    }

    async fn update(&self, job_type: JobType) -> Result<JobType> {
        let mut conn = get_connection(&self.pool)?;
        
        // First check if the entity exists
        let _ = job_types::table
            .find(job_type.id)
            .select(JobType::as_select())
            .first(&mut conn)
            .map_err(|e| match e {
                diesel::result::Error::NotFound => Error::NotFound(format!("JobType not found: {}", job_type.id)),
                e => Error::Database(e),
            })?;
            
        // Update the record
        diesel::update(job_types::table)
            .filter(job_types::id.eq(job_type.id))
            .set((
                job_types::name.eq(job_type.name),
                job_types::description.eq(job_type.description),
                job_types::processing_logic_id.eq(job_type.processing_logic_id),
                job_types::processor_type.eq(job_type.processor_type.as_str()),
                job_types::standard_cost_cents.eq(job_type.standard_cost_cents),
                job_types::enabled.eq(job_type.enabled),
                job_types::updated_at.eq(diesel::dsl::now),
            ))
            .returning(JobType::as_select())
            .get_result(&mut conn)
            .map_err(|e| Error::Database(e))
    }

    async fn list_all(&self) -> Result<Vec<JobType>> {
        let mut conn = get_connection(&self.pool)?;
        
        job_types::table
            .select(JobType::as_select())
            .load(&mut conn)
            .map_err(|e| Error::Database(e))
    }

    async fn list_enabled(&self) -> Result<Vec<JobType>> {
        let mut conn = get_connection(&self.pool)?;
        
        job_types::table
            .filter(job_types::enabled.eq(true))
            .select(JobType::as_select())
            .load(&mut conn)
            .map_err(|e| Error::Database(e))
    }
}
