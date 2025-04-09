use diesel::Connection;
use diesel::pg::{PgConnection, Pg};
use std::path::Path;
use crate::errors::Error;
use diesel_migrations::MigrationHarness;
use diesel::migration::{MigrationSource, Migration};
use anyhow::anyhow;

/// Run all pending migrations
pub fn run_migrations(database_url: &str) -> Result<(), Error> {
    // Connect to the database
    let mut conn = PgConnection::establish(database_url)
        .map_err(|e| Error::Other(anyhow!("Database connection error: {}", e)))?;
    
    // Get absolute path to migrations directory
    let migrations_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("migrations");
    
    // Run the migrations using the file-based migration source
    let migrations = diesel_migrations::FileBasedMigrations::from_path(&migrations_dir)
        .map_err(|e| Error::Other(anyhow!("Failed to load migrations: {}", e)))?;
    
    conn.run_pending_migrations(migrations)
        .map_err(|e| Error::Other(anyhow!("Migration failed: {}", e)))?;
    
    Ok(())
}

/// Get the current migration status
pub fn migration_status(database_url: &str) -> Result<String, Error> {
    // Connect to the database
    let mut conn = PgConnection::establish(database_url)
        .map_err(|e| Error::Other(anyhow!("Database connection error: {}", e)))?;
    
    // Get absolute path to migrations directory
    let migrations_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("migrations");
    
    // Load the migrations from the file system
    let migrations = diesel_migrations::FileBasedMigrations::from_path(&migrations_dir)
        .map_err(|e| Error::Other(anyhow!("Failed to load migrations: {}", e)))?;
    
    // Get applied migrations
    let applied_versions = conn.applied_migrations()
        .map_err(|e| Error::Other(anyhow!("Failed to query applied migrations: {}", e)))?;
    
    // Build a basic status string first
    let mut status = String::new();
    status.push_str(&format!("Applied migrations: {}\n", applied_versions.len()));
    
    // Get available migrations using the trait implementation
    let available_names: Vec<String> = MigrationSource::<Pg>::migrations(&migrations)
        .map_err(|e| Error::Other(anyhow!("Failed to get available migrations: {}", e)))?        
        .iter()
        .map(|m: &Box<dyn Migration<Pg>>| m.name().to_string())
        .collect();
    
    status.push_str(&format!("Total migrations: {}\n", available_names.len()));
    
    // List applied migrations
    if !applied_versions.is_empty() {
        status.push_str("\nApplied migrations:\n");
        for version in &applied_versions {
            status.push_str(&format!("  - {}\n", version));
        }
    }
    
    // Check for pending migrations by comparing the counts
    if applied_versions.len() < available_names.len() {
        status.push_str("\nPending migrations:\n");
        
        // Extract the applied version strings for comparison
        let applied_version_strings: Vec<String> = applied_versions
            .iter()
            .map(|v| v.to_string())
            .collect();
        
        // List migrations that haven't been applied yet
        for name in &available_names {
            if !applied_version_strings.contains(name) {
                status.push_str(&format!("  - {}\n", name));
            }
        }
    } else {
        status.push_str("\nAll migrations have been applied.\n");
    }
    
    Ok(status)
}
