use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use axum::{Router, routing::{get, post, put}};
use axum::middleware::from_fn_with_state;

mod config;
mod handlers;
mod middleware;
mod services;
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
            // Reseller management endpoints (admin only)
            .route("/resellers", get(handlers::resellers::get_all_resellers)
                                .post(handlers::resellers::create_reseller))
            .route("/resellers/active", get(handlers::resellers::get_active_resellers))
            .route("/resellers/{id}", get(handlers::resellers::get_reseller)
                                    .put(handlers::resellers::update_reseller))
            .route("/resellers/{id}/regenerate-key", post(handlers::resellers::regenerate_api_key))
            .layer(from_fn_with_state(app_state.clone(), crate::middleware::auth::admin_auth))
        )
        
        // Reseller routes (reseller authentication required)
        .nest("/reseller", Router::new()
            // Endpoints accessible to resellers
            .route("/profile", get(handlers::resellers::get_current_reseller_profile))
            .route("/active-resellers", get(handlers::resellers::get_active_resellers))
            .layer(from_fn_with_state(app_state.clone(), crate::middleware::auth::reseller_auth))
        )
        
        // Runner heartbeat endpoint (public - no auth required)
        .route("/runners/{id}/heartbeat", post(handlers::runners::update_heartbeat))
        
        // Regular API routes with appropriate authentication
        // Jobs endpoints - require customer auth
        .route("/jobs", get(handlers::jobs::get_all_jobs)
                        .post(handlers::jobs::create_job))
        .route("/jobs/{id}", get(handlers::jobs::get_job))
        .route("/jobs/cost/calculate", post(handlers::jobs::calculate_job_cost))
        .route("/jobs/complete", post(handlers::jobs::complete_job))
        
        // Project endpoints - require customer auth
        .route("/projects", get(handlers::projects::list_customer_projects)
                           .post(handlers::projects::create_project))
        .route("/projects/{id}", get(handlers::projects::get_project)
                               .put(handlers::projects::update_project)
                               .delete(handlers::projects::delete_project))
                               
        // Wallet endpoints - require customer auth
        .route("/wallets/{customer_id}", get(handlers::wallet::get_wallet))
        .route("/wallets/{customer_id}/deposit", post(handlers::wallet::deposit_funds))
        .route("/wallets/{customer_id}/transactions/{limit}/{offset}", get(handlers::wallet::get_transactions))
        .route("/wallets/job/{job_id}/transactions", get(handlers::wallet::get_job_transactions))
        .layer(from_fn_with_state(app_state.clone(), crate::middleware::auth::customer_auth))
        
        // Job types endpoints - require admin auth
        .route("/job-types", get(handlers::job_types::get_all_job_types)
                             .post(handlers::job_types::create_job_type))
        .route("/job-types/{id}", get(handlers::job_types::get_job_type))
        
        // Admin project endpoints - require admin auth
        .route("/all-projects", get(handlers::projects::list_all_projects))
        
        // Runner management endpoints - require admin auth
        .route("/runners", get(handlers::runners::list_all_runners)
                          .post(handlers::runners::register_runner))
        .route("/runners/active", get(handlers::runners::list_active_runners))
        .route("/runners/{id}", get(handlers::runners::get_runner))
        .route("/runners/{id}/capabilities", put(handlers::runners::update_capabilities))
        .route("/runners/{id}/status", put(handlers::runners::set_runner_status))
        
        // Runner health and compatibility endpoints - require admin auth
        .route("/runners/{id}/health", get(handlers::runner_health::check_runner_health))
        .route("/runners/{runner_id}/compatible/{job_type_id}", get(handlers::runner_health::check_compatibility))
        .route("/job-types/{job_type_id}/compatible-runners", get(handlers::runner_health::find_compatible_runners))
        .route("/runners/maintenance/reassign-jobs", post(handlers::runner_health::check_and_reassign_jobs))
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
