use diesel::prelude::*;
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use std::env;
use crate::errors::Error;

pub type PgPool = Pool<ConnectionManager<PgConnection>>;
pub type PgPooledConnection = PooledConnection<ConnectionManager<PgConnection>>;

// Define a trait for transaction operations
pub trait Transaction {
    fn run_in_transaction<F, T>(&self, f: F) -> Result<T, Error>
    where
        F: FnOnce(&mut PgConnection) -> Result<T, diesel::result::Error>;
}

/// Initialize database connection pool
pub fn init_pool() -> Result<PgPool, Error> {
    let database_url = env::var("DATABASE_URL")
        .map_err(|_| Error::Configuration("DATABASE_URL environment variable not set".to_string()))?;
    
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder()
        .build(manager)
        .map_err(|e| Error::Configuration(format!("Failed to create database pool: {}", e)))
}

/// Get a connection from the pool
pub fn get_connection(pool: &PgPool) -> Result<PgPooledConnection, Error> {
    pool.get()
        .map_err(|e| Error::Configuration(format!("Failed to get DB connection from pool: {}", e)))
}

/// Execute a function within a transaction
pub fn transaction<F, T>(conn: &mut PgConnection, f: F) -> Result<T, Error>
where
    F: FnOnce(&mut PgConnection) -> Result<T, diesel::result::Error>,
{
    conn.transaction(|conn| f(conn))
        .map_err(|e| {
            if let diesel::result::Error::RollbackTransaction = e {
                Error::Transaction("Transaction was rolled back".to_string())
            } else {
                Error::Database(e)
            }
        })
}

// Implement Transaction trait for PgPool
impl Transaction for PgPool {
    fn run_in_transaction<F, T>(&self, f: F) -> Result<T, Error>
    where
        F: FnOnce(&mut PgConnection) -> Result<T, diesel::result::Error>,
    {
        let mut conn = get_connection(self)?;
        transaction(&mut conn, f)
    }
}

/// Validate that all models match the database schema
pub fn validate_schema(conn: &mut PgConnection) -> Result<(), Error> {
    // This function will be expanded in future phases to perform actual validation
    // For now, it just checks if we can connect to the database
    
    // Try to execute a simple query
    diesel::select(diesel::dsl::sql::<diesel::sql_types::Bool>("SELECT TRUE"))
        .execute(conn)
        .map_err(|e| Error::Database(e))?;
    
    Ok(())
}

// This helper will be used for integration tests to verify schema compatibility
#[cfg(test)]
pub mod test_utils {
    use super::*;
    
    pub fn setup_test_db() -> PgPool {
        let test_url = env::var("TEST_DATABASE_URL")
            .expect("TEST_DATABASE_URL environment variable must be set for tests");
            
        let manager = ConnectionManager::<PgConnection>::new(test_url);
        Pool::builder()
            .build(manager)
            .expect("Failed to create test database pool")
    }
    
    pub fn clean_test_db(conn: &mut PgConnection) {
        // This will be implemented later to truncate test tables between tests
    }
}
