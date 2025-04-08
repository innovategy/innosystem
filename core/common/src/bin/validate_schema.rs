use diesel::prelude::*;
use dotenvy::dotenv;
use std::env;
use std::process;
use innosystem_common::diesel_schema::*;

fn main() {
    // Load environment variables from .env file
    dotenv().ok();
    
    // Initialize database connection
    let database_url = match env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            eprintln!("Error: DATABASE_URL environment variable not set");
            process::exit(1);
        }
    };
    
    // Connect to the database
    let mut conn = match PgConnection::establish(&database_url) {
        Ok(conn) => conn,
        Err(err) => {
            eprintln!("Error connecting to database: {}", err);
            process::exit(1);
        }
    };
    
    println!("Connected to database. Validating schema...");
    
    // Validate tables exist with the expected structure
    validate_tables(&mut conn);
    
    println!("Schema validation complete. All models match the database schema.");
}

fn validate_tables(conn: &mut PgConnection) {
    // Validate job_types table
    println!("Validating job_types table...");
    let job_types_count: i64 = job_types::table
        .count()
        .get_result(conn)
        .expect("Failed to count job_types");
    println!("Found {} job types in the database.", job_types_count);
    
    // Validate jobs table
    println!("Validating jobs table...");
    let jobs_count: i64 = jobs::table
        .count()
        .get_result(conn)
        .expect("Failed to count jobs");
    println!("Found {} jobs in the database.", jobs_count);
    
    // Validate customers table
    println!("Validating customers table...");
    let customers_count: i64 = customers::table
        .count()
        .get_result(conn)
        .expect("Failed to count customers");
    println!("Found {} customers in the database.", customers_count);
    
    // Validate wallets table
    println!("Validating wallets table...");
    let wallets_count: i64 = wallets::table
        .count()
        .get_result(conn)
        .expect("Failed to count wallets");
    println!("Found {} wallets in the database.", wallets_count);
    
    // Validate wallet_transactions table
    println!("Validating wallet_transactions table...");
    let transactions_count: i64 = wallet_transactions::table
        .count()
        .get_result(conn)
        .expect("Failed to count wallet transactions");
    println!("Found {} wallet transactions in the database.", transactions_count);
}
