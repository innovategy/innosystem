use async_trait::async_trait;
use uuid::Uuid;
use anyhow::Result;

use crate::models::reseller::Reseller;
use crate::models::reseller::NewReseller;

/// Repository trait for Reseller operations
#[async_trait]
pub trait ResellerRepository: Send + Sync {
    /// Create a new reseller
    async fn create(&self, reseller: NewReseller) -> Result<Reseller>;
    
    /// Find a reseller by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Reseller>;
    
    /// Find a reseller by API key
    async fn find_by_api_key(&self, api_key: &str) -> Result<Reseller>;
    
    /// Update a reseller
    async fn update(&self, reseller: &Reseller) -> Result<Reseller>;
    
    /// List all resellers
    async fn list_all(&self) -> Result<Vec<Reseller>>;
    
    /// List only active resellers
    async fn list_active(&self) -> Result<Vec<Reseller>>;
}
