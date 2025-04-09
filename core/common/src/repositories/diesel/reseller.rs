use async_trait::async_trait;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use uuid::Uuid;
use anyhow::{Result, anyhow};
use chrono::Utc;

use crate::models::reseller::{Reseller, NewReseller};
use crate::repositories::ResellerRepository;
use crate::diesel_schema::resellers;

/// Diesel implementation of the ResellerRepository
pub struct DieselResellerRepository {
    pool: Pool<ConnectionManager<PgConnection>>,
}

impl DieselResellerRepository {
    /// Create a new DieselResellerRepository with the given connection pool
    pub fn new(pool: Pool<ConnectionManager<PgConnection>>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ResellerRepository for DieselResellerRepository {
    async fn create(&self, reseller: NewReseller) -> Result<Reseller> {
        let mut conn = self.pool.get()?;
        
        // Insert the new reseller
        let reseller: Reseller = tokio::task::spawn_blocking(move || {
            diesel::insert_into(resellers::table)
                .values(&reseller)
                .get_result::<Reseller>(&mut conn)
        }).await??;
        
        Ok(reseller)
    }
    
    async fn find_by_id(&self, id: Uuid) -> Result<Reseller> {
        let mut conn = self.pool.get()?;
        
        let reseller: Reseller = tokio::task::spawn_blocking(move || {
            resellers::table
                .find(id)
                .first(&mut conn)
                .optional()
        }).await??
            .ok_or_else(|| anyhow!("Reseller not found with ID: {}", id))?;
        
        Ok(reseller)
    }
    
    async fn find_by_api_key(&self, api_key: &str) -> Result<Reseller> {
        let api_key = api_key.to_string();
        let mut conn = self.pool.get()?;
        
        let reseller: Reseller = tokio::task::spawn_blocking(move || {
            resellers::table
                .filter(resellers::api_key.eq(api_key))
                .first(&mut conn)
                .optional()
        }).await??
            .ok_or_else(|| anyhow!("Reseller not found with API key"))?;
        
        Ok(reseller)
    }
    
    async fn update(&self, reseller: &Reseller) -> Result<Reseller> {
        let reseller_clone = reseller.clone();
        let mut conn = self.pool.get()?;
        
        // Create an updated reseller with the current timestamp
        let mut updated_reseller = reseller_clone.clone();
        updated_reseller.updated_at = Some(Utc::now().naive_utc());
        
        let updated_reseller = tokio::task::spawn_blocking(move || {
            diesel::update(resellers::table.find(reseller_clone.id))
                .set((
                    resellers::name.eq(&updated_reseller.name),
                    resellers::email.eq(&updated_reseller.email),
                    resellers::api_key.eq(&updated_reseller.api_key),
                    resellers::active.eq(updated_reseller.active),
                    resellers::commission_rate.eq(updated_reseller.commission_rate),
                    resellers::updated_at.eq(updated_reseller.updated_at),
                ))
                .get_result::<Reseller>(&mut conn)
        }).await??;
        
        Ok(updated_reseller)
    }
    
    async fn list_all(&self) -> Result<Vec<Reseller>> {
        let mut conn = self.pool.get()?;
        
        let resellers: Vec<Reseller> = tokio::task::spawn_blocking(move || {
            resellers::table
                .load::<Reseller>(&mut conn)
        }).await??;
        
        Ok(resellers)
    }
    
    async fn list_active(&self) -> Result<Vec<Reseller>> {
        let mut conn = self.pool.get()?;
        
        let resellers: Vec<Reseller> = tokio::task::spawn_blocking(move || {
            resellers::table
                .filter(resellers::active.eq(true))
                .load::<Reseller>(&mut conn)
        }).await??;
        
        Ok(resellers)
    }
}
