use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use innosystem_common::migrations;
use std::env;
use std::error::Error;

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
    }
    
    Ok(())
}
