use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use innosystem_common::{migrations, seed::{Seeder}, database};
use innosystem_common::repositories::diesel::{DieselJobTypeRepository, DieselJobRepository, DieselCustomerRepository, DieselWalletRepository};
use innosystem_common::repositories::{job_type::JobTypeRepository, customer::CustomerRepository, job::JobRepository, wallet::WalletRepository};
use std::env;
use std::error::Error;
use std::sync::Arc;

/// Innosystem Database Migration Tool
#[derive(Parser)]
#[clap(name = "innosystem-migrations", version = "0.1.0", author = "Innosystem Team")]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run all pending migrations
    #[clap(name = "run")]
    Run,
    
    /// Check the current migration state
    #[clap(name = "status")]
    Status,
    
    /// Rerun the last migration (useful for development)
    #[clap(name = "rerun-latest")]
    RerunLatest,

    /// Seed the database with development data
    #[clap(name = "seed")]
    Seed,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load environment variables from .env file
    dotenv().ok();
    
    // Parse command line arguments
    let cli = Cli::parse();
    
    // Get database URL from environment
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL environment variable must be set");
    
    // Process commands
    match cli.command {
        Commands::Run => {
            println!("Running migrations...");
            migrations::run_migrations(&database_url)?;
            println!("Migrations completed successfully.");
        },
        Commands::Status => {
            println!("Migration status feature not yet implemented.");
            println!("This will be added in a future update.");
        },
        Commands::RerunLatest => {
            println!("Rerun latest migration feature not yet implemented.");
            println!("This will be added in a future update.");
        },
        Commands::Seed => {
            println!("Seeding database with development data...");
            
            // First, ensure migrations are run
            println!("Running migrations to ensure schema is up to date...");
            migrations::run_migrations(&database_url)?;
            
            // Initialize database connection pool
            let pool = database::init_pool()?;
            
            // Create repository implementations
            let job_type_repo: Arc<dyn JobTypeRepository + Send + Sync> = Arc::new(DieselJobTypeRepository::new(pool.clone()));
            
            // For repositories that don't have Diesel implementations yet, we'll need to implement those
            // or use in-memory implementations for now
            println!("Using Diesel repositories for all entity types");
            // Using in-memory implementations for repositories that don't have Diesel implementations yet
            let customer_repo: Arc<dyn CustomerRepository + Send + Sync> = Arc::new(DieselCustomerRepository::new(pool.clone()));
            let wallet_repo: Arc<dyn WalletRepository + Send + Sync> = Arc::new(DieselWalletRepository::new(pool.clone()));
            
            let job_repo: Arc<dyn JobRepository + Send + Sync> = Arc::new(DieselJobRepository::new(pool.clone()));
            
            // Create and run seeder
            let seeder = Seeder::new(
                job_type_repo,
                customer_repo,
                job_repo,
                wallet_repo
            );
            
            // Seed all entity types now that we have proper Diesel repositories for all
            println!("Seeding all entity types: job types, customers, wallets, and jobs...");
            seeder.seed_all().await?;
            
            println!("Seed data successfully inserted into database.");
        },
    }
    
    Ok(())
}
