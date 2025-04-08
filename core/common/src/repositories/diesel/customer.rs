use async_trait::async_trait;
use diesel::prelude::*;
use uuid::Uuid;

use crate::database::{PgPool, get_connection};
use crate::diesel_schema::customers;
use crate::errors::Error;
use crate::models::customer::{Customer, NewCustomer};
use crate::repositories::CustomerRepository;
use crate::Result;

/// Diesel-backed implementation of CustomerRepository
pub struct DieselCustomerRepository {
    pool: PgPool,
}

impl DieselCustomerRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CustomerRepository for DieselCustomerRepository {
    async fn create(&self, new_customer: NewCustomer) -> Result<Customer> {
        let mut conn = get_connection(&self.pool)?;
        
        // Execute the insert and return the new record
        diesel::insert_into(customers::table)
            .values(&new_customer)
            .returning(Customer::as_select())
            .get_result(&mut conn)
            .map_err(|e| Error::Database(e))
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Customer> {
        let mut conn = get_connection(&self.pool)?;
        
        customers::table
            .find(id)
            .select(Customer::as_select())
            .first(&mut conn)
            .map_err(|e| match e {
                diesel::result::Error::NotFound => Error::NotFound(format!("Customer not found: {}", id)),
                e => Error::Database(e),
            })
    }

    async fn find_by_api_key(&self, _api_key: &str) -> Result<Customer> {
        // The database schema doesn't have an api_key field currently
        // This is a placeholder implementation that returns an error
        Err(Error::NotFound("API key lookup is not implemented in the current database schema".to_string()))
    }

    async fn update(&self, customer: Customer) -> Result<Customer> {
        let mut conn = get_connection(&self.pool)?;
        
        // First check if the entity exists
        let _ = customers::table
            .find(customer.id)
            .select(Customer::as_select())
            .first(&mut conn)
            .map_err(|e| match e {
                diesel::result::Error::NotFound => Error::NotFound(format!("Customer not found: {}", customer.id)),
                e => Error::Database(e),
            })?;
            
        // Update the record
        diesel::update(customers::table)
            .filter(customers::id.eq(customer.id))
            .set((
                customers::name.eq(customer.name),
                customers::email.eq(customer.email),
                customers::updated_at.eq(diesel::dsl::now),
            ))
            .returning(Customer::as_select())
            .get_result(&mut conn)
            .map_err(|e| Error::Database(e))
    }

    async fn list_all(&self) -> Result<Vec<Customer>> {
        let mut conn = get_connection(&self.pool)?;
        
        customers::table
            .select(Customer::as_select())
            .load(&mut conn)
            .map_err(|e| Error::Database(e))
    }
}
