use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use async_trait::async_trait;
use uuid::Uuid;

use crate::errors::Error;
use crate::models::customer::{Customer, NewCustomer};
use crate::repositories::CustomerRepository;
use crate::Result;

/// In-memory implementation of CustomerRepository for Phase 1
pub struct InMemoryCustomerRepository {
    customers: Arc<Mutex<HashMap<Uuid, Customer>>>,
    api_keys: Arc<Mutex<HashMap<String, Uuid>>>,
}

impl InMemoryCustomerRepository {
    pub fn new() -> Self {
        Self {
            customers: Arc::new(Mutex::new(HashMap::new())),
            api_keys: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl CustomerRepository for InMemoryCustomerRepository {
    async fn create(&self, new_customer: NewCustomer) -> Result<Customer> {
        let customer = Customer {
            id: new_customer.id,
            name: new_customer.name,
            email: new_customer.email,
            api_key: new_customer.api_key.clone(),
            active: new_customer.active,
            created_at: Some(chrono::Utc::now().naive_utc()),
            updated_at: Some(chrono::Utc::now().naive_utc()),
        };
        
        let mut customers = self.customers.lock().map_err(|_| Error::Other(anyhow::anyhow!("Lock error")))?;
        customers.insert(customer.id, customer.clone());
        
        // Register API key if provided
        if let Some(api_key) = &new_customer.api_key {
            let mut api_keys = self.api_keys.lock().map_err(|_| Error::Other(anyhow::anyhow!("Lock error")))?;
            api_keys.insert(api_key.clone(), customer.id);
        }
        
        Ok(customer)
    }
    
    async fn find_by_id(&self, id: Uuid) -> Result<Customer> {
        let customers = self.customers.lock().map_err(|_| Error::Other(anyhow::anyhow!("Lock error")))?;
        customers
            .get(&id)
            .cloned()
            .ok_or_else(|| Error::NotFound(format!("Customer not found: {}", id)))
    }
    
    async fn find_by_api_key(&self, api_key: &str) -> Result<Customer> {
        // Get customer ID from API key map, but drop the lock before the await
        let customer_id = {
            let api_keys = self.api_keys.lock().map_err(|_| Error::Other(anyhow::anyhow!("Lock error")))?;
            api_keys
                .get(api_key)
                .cloned()
                .ok_or_else(|| Error::NotFound(format!("API key not found: {}", api_key)))?
        };
            
        // Now find the customer with the ID (no lock held across await)
        self.find_by_id(customer_id).await
    }
    
    async fn update(&self, customer: Customer) -> Result<Customer> {
        let mut customers = self.customers.lock().map_err(|_| Error::Other(anyhow::anyhow!("Lock error")))?;
        
        // Check if customer exists
        if !customers.contains_key(&customer.id) {
            return Err(Error::NotFound(format!("Customer not found: {}", customer.id)));
        }
        
        // Update the customer
        let updated_customer = Customer {
            updated_at: Some(chrono::Utc::now().naive_utc()),
            ..customer
        };
        
        customers.insert(updated_customer.id, updated_customer.clone());
        
        // Update API key mapping if present
        if let Some(api_key) = &updated_customer.api_key {
            let mut api_keys = self.api_keys.lock().map_err(|_| Error::Other(anyhow::anyhow!("Lock error")))?;
            api_keys.insert(api_key.clone(), updated_customer.id);
        }
        
        Ok(updated_customer)
    }
    
    async fn list_all(&self) -> Result<Vec<Customer>> {
        let customers = self.customers.lock().map_err(|_| Error::Other(anyhow::anyhow!("Lock error")))?;
        Ok(customers.values().cloned().collect())
    }
}
