use async_trait::async_trait;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use uuid::Uuid;
use anyhow::{Result, anyhow};
use rand::Rng;
use chrono::Utc;

use crate::diesel_schema::customers;
use crate::models::customer::{Customer, NewCustomer};
use crate::repositories::CustomerRepository;

/// Diesel-backed implementation of CustomerRepository
pub struct DieselCustomerRepository {
    pool: Pool<ConnectionManager<PgConnection>>,
}

impl DieselCustomerRepository {
    pub fn new(pool: Pool<ConnectionManager<PgConnection>>) -> Self {
        Self { pool }
    }
    
    /// Generate a random API key
    fn generate_random_api_key() -> String {
        let mut rng = rand::rng();
        let key: String = (0..32)
            .map(|_| {
                // Generate a random character (A-Z, a-z, 0-9)
                const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                                          abcdefghijklmnopqrstuvwxyz\
                                          0123456789";
                let idx = rng.random_range(0..CHARSET.len());
                CHARSET[idx] as char
            })
            .collect();
        format!("cust_{}", key)
    }
}

#[async_trait]
impl CustomerRepository for DieselCustomerRepository {
    async fn create(&self, new_customer: NewCustomer) -> Result<Customer> {
        let mut conn = self.pool.get()?;
        
        // Execute the insert and return the new record
        let customer: Customer = tokio::task::spawn_blocking(move || {
            let result = diesel::insert_into(customers::table)
                .values(&new_customer)
                .get_result::<Customer>(&mut conn);
                
            match result {
                Ok(customer) => Ok(customer),
                Err(e) => Err(anyhow!("Failed to create customer: {}", e))
            }
        }).await??;
        
        Ok(customer)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Customer> {
        let mut conn = self.pool.get()?;
        
        let customer: Customer = tokio::task::spawn_blocking(move || {
            customers::table
                .find(id)
                .first(&mut conn)
                .optional()
        }).await??
            .ok_or_else(|| anyhow!("Customer not found with ID: {}", id))?;
        
        Ok(customer)
    }

    async fn find_by_api_key(&self, api_key: &str) -> Result<Customer> {
        let api_key = api_key.to_string();
        let mut conn = self.pool.get()?;
        
        let customer: Customer = tokio::task::spawn_blocking(move || {
            customers::table
                .filter(customers::api_key.eq(api_key))
                .first(&mut conn)
                .optional()
        }).await??
            .ok_or_else(|| anyhow!("Customer not found with API key"))?;
        
        Ok(customer)
    }
    
    async fn find_by_reseller_id(&self, reseller_id: Uuid) -> Result<Vec<Customer>> {
        let mut conn = self.pool.get()?;
        
        let customers: Vec<Customer> = tokio::task::spawn_blocking(move || {
            let result = customers::table
                .filter(customers::reseller_id.eq(reseller_id))
                .load::<Customer>(&mut conn);
                
            match result {
                Ok(customers) => Ok(customers),
                Err(e) => Err(anyhow!("Failed to find customers by reseller ID: {}", e))
            }
        }).await??;
        
        Ok(customers)
    }

    async fn update(&self, customer: &Customer) -> Result<Customer> {
        let customer_clone = customer.clone();
        let mut conn = self.pool.get()?;
        
        // Create an updated customer with the current timestamp
        let mut updated_customer = customer_clone.clone();
        updated_customer.updated_at = Some(Utc::now().naive_utc());
        
        let updated_customer = tokio::task::spawn_blocking(move || {
            let result = diesel::update(customers::table.find(customer_clone.id))
                .set((
                    customers::name.eq(&updated_customer.name),
                    customers::email.eq(&updated_customer.email),
                    customers::api_key.eq(&updated_customer.api_key),
                    customers::reseller_id.eq(updated_customer.reseller_id),
                    customers::updated_at.eq(updated_customer.updated_at),
                ))
                .get_result::<Customer>(&mut conn);
                
            match result {
                Ok(customer) => Ok(customer),
                Err(e) => Err(anyhow!("Failed to update customer: {}", e))
            }
        }).await??;
        
        Ok(updated_customer)
    }
    
    async fn set_reseller(&self, customer_id: Uuid, reseller_id: Option<Uuid>) -> Result<Customer> {
        let mut conn = self.pool.get()?;
        
        // First validate the customer exists
        let _customer = self.find_by_id(customer_id).await?;
        
        // Update only the reseller_id field
        let updated_customer = tokio::task::spawn_blocking(move || {
            let result = diesel::update(customers::table.find(customer_id))
                .set(customers::reseller_id.eq(reseller_id))
                .get_result::<Customer>(&mut conn);
                
            match result {
                Ok(customer) => Ok(customer),
                Err(e) => Err(anyhow!("Failed to set reseller for customer: {}", e))
            }
        }).await??;
        
        Ok(updated_customer)
    }
    
    async fn generate_api_key(&self, customer_id: Uuid) -> Result<String> {
        let mut conn = self.pool.get()?;
        
        // Generate a unique API key
        let api_key = Self::generate_random_api_key();
        
        // Update the customer's API key
        let customer = tokio::task::spawn_blocking(move || {
            let result = diesel::update(customers::table.find(customer_id))
                .set(customers::api_key.eq(&api_key))
                .get_result::<Customer>(&mut conn);
                
            match result {
                Ok(customer) => Ok(customer),
                Err(e) => Err(anyhow!("Failed to update API key for customer: {}", e))
            }
        }).await??;
        
        Ok(customer.api_key.unwrap_or_default())
    }

    async fn list_all(&self) -> Result<Vec<Customer>> {
        let mut conn = self.pool.get()?;
        
        let customers: Vec<Customer> = tokio::task::spawn_blocking(move || {
            let result = customers::table
                .load::<Customer>(&mut conn);
                
            match result {
                Ok(customers) => Ok(customers),
                Err(e) => Err(anyhow!("Failed to list all customers: {}", e))
            }
        }).await??;
        
        Ok(customers)
    }
}
