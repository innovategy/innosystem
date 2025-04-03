use async_trait::async_trait;
use uuid::Uuid;

use crate::models::customer::{Customer, NewCustomer};
use crate::Result;

#[async_trait]
pub trait CustomerRepository: Send + Sync {
    async fn create(&self, new_customer: NewCustomer) -> Result<Customer>;
    async fn find_by_id(&self, id: Uuid) -> Result<Customer>;
    async fn find_by_api_key(&self, api_key: &str) -> Result<Customer>;
    async fn update(&self, customer: Customer) -> Result<Customer>;
    async fn list_all(&self) -> Result<Vec<Customer>>;
}
