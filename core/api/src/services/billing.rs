use std::sync::Arc;
use uuid::Uuid;
use anyhow::{Result, Context, anyhow};
use tracing::{info, error, warn};

// Import wallet models when needed
use innosystem_common::repositories::{JobRepository, JobTypeRepository, WalletRepository, CustomerRepository};

/// Service for handling billing and cost calculation operations
pub struct BillingService {
    job_repo: Arc<dyn JobRepository>,
    job_type_repo: Arc<dyn JobTypeRepository>,
    wallet_repo: Arc<dyn WalletRepository>,
    customer_repo: Arc<dyn CustomerRepository>,
}

impl BillingService {
    /// Create a new BillingService
    pub fn new(
        job_repo: Arc<dyn JobRepository>,
        job_type_repo: Arc<dyn JobTypeRepository>,
        wallet_repo: Arc<dyn WalletRepository>,
        customer_repo: Arc<dyn CustomerRepository>,
    ) -> Self {
        Self {
            job_repo,
            job_type_repo,
            wallet_repo,
            customer_repo,
        }
    }
    
    /// Calculate the actual cost of a completed job
    pub async fn calculate_job_cost(&self, job_id: Uuid) -> Result<i32> {
        // Fetch the job
        let job = self.job_repo.find_by_id(job_id)
            .await
            .context("Failed to fetch job for cost calculation")?;
        
        // Fetch the job type to get the standard cost
        let job_type = self.job_type_repo.find_by_id(job.job_type_id)
            .await
            .context("Failed to fetch job type for cost calculation")?;
        
        // Start with the base cost from the job type
        let mut final_cost = job_type.standard_cost_cents;
        
        // Apply dynamic cost factors based on job details
        // For now, we'll use a simple multiplier based on priority
        let priority_multiplier = match job.priority.as_i32() {
            0 => 1.0,   // Low priority - standard cost
            1 => 1.0,   // Medium priority - standard cost
            2 => 1.5,   // High priority - 50% premium
            3 => 2.0,   // Critical priority - 100% premium
            _ => 1.0,   // Default
        };
        
        // Apply priority multiplier
        final_cost = (final_cost as f64 * priority_multiplier).round() as i32;
        
        // Apply any other business rules for cost adjustment
        // (In the future, this could include duration-based costs, resource usage, etc.)
        
        info!("Calculated final cost for job {}: {} cents", job_id, final_cost);
        
        Ok(final_cost)
    }
    
    /// Process billing for a completed job
    /// This method handles the wallet transaction and updates the job record
    pub async fn process_job_billing(&self, job_id: Uuid, success: bool) -> Result<()> {
        // Fetch the job
        let job = self.job_repo.find_by_id(job_id)
            .await
            .context("Failed to fetch job for billing")?;
        
        // Calculate the actual cost of the job
        let actual_cost = if success {
            self.calculate_job_cost(job_id).await?
        } else {
            // For failed jobs, we might charge a reduced fee or nothing
            // For now, let's charge 25% of the estimated cost for failed jobs
            (job.estimated_cost_cents as f64 * 0.25).round() as i32
        };
        
        // Try to find the customer's wallet
        let wallet = match self.wallet_repo.find_by_customer_id(job.customer_id).await {
            Ok(wallet) => wallet,
            Err(e) => {
                error!("Failed to find wallet for customer {}: {}", job.customer_id, e);
                return Err(anyhow!("Customer wallet not found"));
            }
        };
        
        // Perform the wallet transaction
        // Use the correct transaction type from the model
        // JobDebit for all jobs (successful and failed) with different descriptions
        
        let description = format!(
            "{} job {} - {}",
            if success { "Completed" } else { "Failed" },
            job_id,
            if let Ok(job_type) = self.job_type_repo.find_by_id(job.job_type_id).await {
                job_type.name
            } else {
                "Unknown job type".to_string()
            }
        );
        
        // Check if there's a reservation to release or create a new charge
        // In a real system, you'd have a record of the reservation
        // Here we'll just create a new withdrawal
        match self.wallet_repo.withdraw(
            wallet.id,
            actual_cost,
            Some(description),
            Some(job_id)
        ).await {
            Ok(_) => {
                info!("Successfully charged {} cents for job {}", actual_cost, job_id);
                
                // Update the job with the final cost
                if let Err(e) = self.job_repo.set_completed(
                    job_id,
                    success,
                    job.output_data.clone(),
                    job.error.clone(),
                    actual_cost
                ).await {
                    error!("Failed to update job with final cost: {}", e);
                    // We don't want to fail the whole operation if just the cost update fails
                    // The customer has been charged, but the job record might not reflect the final cost
                    warn!("Job {} completed and customer charged, but job record not updated with final cost", job_id);
                }
                
                Ok(())
            },
            Err(e) => {
                error!("Failed to process payment for job {}: {}", job_id, e);
                Err(anyhow!("Payment processing failed: {}", e))
            }
        }
    }
    
    /// Pre-authorize funds for a job
    /// This creates a reservation in the customer's wallet
    pub async fn reserve_funds_for_job(&self, job_id: Uuid) -> Result<()> {
        // Fetch the job
        let job = self.job_repo.find_by_id(job_id)
            .await
            .context("Failed to fetch job for fund reservation")?;
        
        // Find the customer's wallet
        let wallet = self.wallet_repo.find_by_customer_id(job.customer_id)
            .await
            .context("Failed to find customer wallet")?;
        
        // Reserve the estimated cost
        let description = format!("Reservation for job {}", job_id);
        
        self.wallet_repo.reserve_funds(
            wallet.id,
            job.estimated_cost_cents,
            Some(description),
            Some(job_id)
        ).await
        .context("Failed to reserve funds for job")?;
        
        info!("Reserved {} cents for job {}", job.estimated_cost_cents, job_id);
        
        Ok(())
    }
    
    /// Release funds reservation for a job (e.g., if cancelled)
    pub async fn release_reserved_funds(&self, job_id: Uuid) -> Result<()> {
        // Fetch the job
        let job = self.job_repo.find_by_id(job_id)
            .await
            .context("Failed to fetch job for releasing funds")?;
        
        // Find the customer's wallet
        let wallet = self.wallet_repo.find_by_customer_id(job.customer_id)
            .await
            .context("Failed to find customer wallet")?;
        
        // Release the reserved funds
        let description = format!("Release reservation for job {}", job_id);
        
        self.wallet_repo.release_reservation(
            wallet.id,
            job.estimated_cost_cents, // Release the originally estimated amount
            Some(description),
            Some(job_id)
        ).await
        .context("Failed to release reserved funds")?;
        
        info!("Released reservation of {} cents for job {}", job.estimated_cost_cents, job_id);
        
        Ok(())
    }
}
