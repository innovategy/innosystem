use async_trait::async_trait;
use uuid::Uuid;
use anyhow::Result;

use crate::models::customer::{Customer, NewCustomer};

#[async_trait]
pub trait CustomerRepository: Send + Sync {
    /// Create a new customer
    async fn create(&self, new_customer: NewCustomer) -> Result<Customer>;
    
    /// Find a customer by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Customer>;
    
    /// Find a customer by API key
    async fn find_by_api_key(&self, api_key: &str) -> Result<Customer>;
    
    /// Find customers by reseller ID
    async fn find_by_reseller_id(&self, reseller_id: Uuid) -> Result<Vec<Customer>>;
    
    /// Update a customer
    async fn update(&self, customer: &Customer) -> Result<Customer>;
    
    /// Set or update a customer's reseller
    async fn set_reseller(&self, customer_id: Uuid, reseller_id: Option<Uuid>) -> Result<Customer>;
    
    /// Generate and set API key for a customer
    async fn generate_api_key(&self, customer_id: Uuid) -> Result<String>;
    
    /// List all customers
    async fn list_all(&self) -> Result<Vec<Customer>>;
}
