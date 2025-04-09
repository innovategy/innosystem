use std::sync::Arc;
use std::time::Duration;

use diesel;
use innosystem_common::{
    queue::{JobQueue, JobQueueConfig, RedisJobQueue},
    repositories::{
        CustomerRepository, JobRepository, JobTypeRepository, WalletRepository,
        diesel::{DieselCustomerRepository, DieselJobRepository, DieselJobTypeRepository, DieselWalletRepository},
    },
    database,
};
use tokio::time::sleep;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod processor;

use config::RunnerConfig;
use processor::{DefaultJobProcessor, JobProcessor};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = RunnerConfig::load()?;
    tracing::info!("Starting job runner with configuration: {:?}", config);

    // Get database URL from config or use default
    let database_url = config.database_url.clone().unwrap_or_else(|| 
        "postgres://postgres:postgres@postgres:5432/innosystem".to_string());
    
    // Create a database connection manager
    let manager = diesel::r2d2::ConnectionManager::<diesel::pg::PgConnection>::new(database_url);
    
    // Build the connection pool
    let pool = diesel::r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to establish database connection");
    
    // Initialize repositories with Diesel implementations
    let job_repo = Arc::new(DieselJobRepository::new(pool.clone()));
    let job_type_repo = Arc::new(DieselJobTypeRepository::new(pool.clone()));
    let wallet_repo = Arc::new(DieselWalletRepository::new(pool.clone()));
    let customer_repo = Arc::new(DieselCustomerRepository::new(pool.clone()));

    // Initialize Redis connection for job queue
    let job_queue = RedisJobQueue::new(
        JobQueueConfig::new(config.redis_url.clone())
            .with_timeout(config.queue_timeout_seconds),
    )
    .await?;

    // Create job processor
    let processor = DefaultJobProcessor::new(
        job_repo.clone(),
        job_type_repo.clone(),
        wallet_repo.clone(),
        customer_repo.clone(),
    );

    // Main processing loop
    tracing::info!("Job runner started and waiting for jobs");
    loop {
        // Process any jobs that may be scheduled for now
        // Use concrete types directly to avoid object safety issues
        let due_jobs = job_queue.get_due_scheduled_jobs().await?;
        for job_id in due_jobs {
            tracing::info!("Processing scheduled job: {}", job_id);
            // Mark job as started
            let job = job_repo.set_started(job_id).await?;
            
            // Process the job
            let result = processor.process_job(job.clone()).await;
            
            // Update job status based on processing result
            match result {
                Ok((output, cost_cents)) => {
                    // Job completed successfully
                    job_repo
                        .set_completed(job_id, true, Some(output), None, cost_cents)
                        .await?;
                    tracing::info!("Job {} completed successfully", job_id);
                }
                Err(err) => {
                    // Job failed
                    job_repo
                        .set_completed(job_id, false, None, Some(err.to_string()), 0) // Use 0 cost for failed jobs
                        .await?;
                    tracing::error!("Job {} failed: {}", job_id, err);
                }
            }
        }

        // Try to get a job from the queue
        match job_queue.pop_job().await {
            Ok(Some(job_id)) => {
                // Process the job directly in the main loop
                tracing::info!("Processing job: {}", job_id);
                
                // Mark job as started
                let job = job_repo.set_started(job_id).await?;
                
                // Process the job
                let result = processor.process_job(job.clone()).await;
                
                // Update job status based on processing result
                match result {
                    Ok((output, cost_cents)) => {
                        // Job completed successfully
                        job_repo
                            .set_completed(job_id, true, Some(output), None, cost_cents)
                            .await?;
                        tracing::info!("Job {} completed successfully", job_id);
                    }
                    Err(err) => {
                        // Job failed
                        job_repo
                            .set_completed(job_id, false, None, Some(err.to_string()), 0) // Use 0 cost for failed jobs
                            .await?;
                        tracing::error!("Job {} failed: {}", job_id, err);
                    }
                }
            }
            Ok(None) => {
                // No jobs available, wait a bit before trying again
                tracing::debug!("No jobs in queue, waiting...");
                sleep(Duration::from_millis(config.poll_interval_ms)).await;
            }
            Err(err) => {
                // Log error and continue
                tracing::error!("Error polling job queue: {}", err);
                sleep(Duration::from_secs(1)).await;
            }
        }
    }
}


