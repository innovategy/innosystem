use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use axum::{Router, routing::get};
use axum::middleware::from_fn_with_state;

mod config;
mod handlers;
mod middleware;
mod state;

use config::AppConfig;
use state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Print startup message
    println!("API STARTING - InnoSystem API Service");
    
    // Initialize tracing for logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
    tracing::info!("Tracing initialized");
    
    // Load application configuration
    let config = AppConfig::load()?;
    tracing::info!("Configuration loaded: {:?}", config);
    
    // Initialize application state with Diesel repositories (persistent database)
    let app_state = match AppState::new_with_diesel(config.clone()).await {
        Ok(state) => state,
        Err(e) => {
            tracing::error!("Failed to initialize application state: {}", e);
            return Err(e.into());
        }
    };
    
    // Create the router with routes
    let app = Router::new()
        // Health check endpoint (no auth required)
        .route("/health", get(handlers::health::health_check))
        
        // Public routes (no authentication needed)
        .nest("/public", Router::new()
            // Add public endpoints here
        )
        
        // Admin routes (admin authentication required)
        .nest("/admin", Router::new()
            // Add admin-only endpoints here
            .layer(from_fn_with_state(app_state.clone(), crate::middleware::auth::admin_auth))
        )
        
        // Reseller routes (reseller authentication required)
        .nest("/reseller", Router::new()
            // Add reseller-only endpoints here
            .layer(from_fn_with_state(app_state.clone(), crate::middleware::auth::reseller_auth))
        )
        
        // Regular API routes with appropriate authentication
        // Jobs endpoints - require customer auth
        .route("/jobs", get(handlers::jobs::get_all_jobs)
                        .post(handlers::jobs::create_job))
        .route("/jobs/{id}", get(handlers::jobs::get_job))
        .layer(from_fn_with_state(app_state.clone(), crate::middleware::auth::customer_auth))
        
        // Job types endpoints - require admin auth
        .route("/job-types", get(handlers::job_types::get_all_job_types)
                             .post(handlers::job_types::create_job_type))
        .route("/job-types/{id}", get(handlers::job_types::get_job_type))
        .layer(from_fn_with_state(app_state.clone(), crate::middleware::auth::admin_auth))
        
        // Customers endpoints - require reseller auth
        .route("/customers", get(handlers::customers::get_all_customers)
                             .post(handlers::customers::create_customer))
        .route("/customers/{id}", get(handlers::customers::get_customer))
        .layer(from_fn_with_state(app_state.clone(), crate::middleware::auth::reseller_auth))
        
        // Add application state
        .with_state(app_state);
    
    // Determine the address to bind to
    let port = config.port.unwrap_or(8080);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Starting server on {}", addr);
    
    // Start the server
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
    
    Ok(())
}
