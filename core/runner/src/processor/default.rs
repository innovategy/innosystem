use std::sync::Arc;

use innosystem_common::{
    models::{
        job::Job,
        job_type::ProcessorType,
        wallet::{NewWalletTransaction, Wallet},
    },
    repositories::{CustomerRepository, JobRepository, JobTypeRepository, WalletRepository},
};
use serde_json::json;
use uuid::Uuid;

use super::JobProcessor;

/// Default implementation of the JobProcessor
pub struct DefaultJobProcessor {
    #[allow(dead_code)]
    job_repo: Arc<dyn JobRepository>,
    job_type_repo: Arc<dyn JobTypeRepository>,
    wallet_repo: Arc<dyn WalletRepository>,
    customer_repo: Arc<dyn CustomerRepository>,
}

impl DefaultJobProcessor {
    /// Create a new DefaultJobProcessor
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

    /// Reserve funds from customer wallet for job processing
    async fn reserve_funds(&self, job: &Job) -> anyhow::Result<Wallet> {
        let wallet = self.wallet_repo.find_by_customer_id(job.customer_id).await?;
        self.wallet_repo.reserve_funds(wallet.id, job.estimated_cost_cents as i64).await
            .map_err(|e| anyhow::anyhow!("Failed to reserve funds: {}", e))
    }

    /// Charge customer wallet for completed job
    async fn charge_wallet(
        &self,
        job: &Job,
        cost_cents: i32,
        success: bool,
    ) -> anyhow::Result<()> {
        let wallet = self.wallet_repo.find_by_customer_id(job.customer_id).await?;
        
        // Release the reserved funds
        self.wallet_repo
            .release_reservation(wallet.id, job.estimated_cost_cents as i64)
            .await?;
        
        // If job was successful, create a transaction for the actual cost
        if success {
            let transaction = NewWalletTransaction {
                id: Uuid::new_v4(),
                wallet_id: wallet.id,
                customer_id: job.customer_id,
                amount_cents: -(cost_cents as i64),
                job_id: Some(job.id),
                description: format!("Job processing: {}", job.id),
            };
            
            self.wallet_repo.add_transaction(transaction).await?;
        }
        
        Ok(())
    }
    
    /// Process a specific job type based on its processor type
    async fn process_job_type(
        &self,
        job: &Job,
        job_type_id: Uuid,
    ) -> anyhow::Result<serde_json::Value> {
        // Get the job type details
        let job_type = self.job_type_repo.find_by_id(job_type_id).await?;
        
        // Process based on processor type
        match job_type.processor_type {
            ProcessorType::Sync => {
                // Sync processor just returns the input data (like the old Echo processor)
                Ok(job.input_data.clone())
            }
            ProcessorType::Async => {
                // Async processor performs a simple transformation (like the old Transform processor)
                let result = if let Some(text) = job.input_data.get("text") {
                    if let Some(text_str) = text.as_str() {
                        json!({
                            "original_text": text_str,
                            "transformed_text": text_str.to_uppercase(),
                            "character_count": text_str.len(),
                            "word_count": text_str.split_whitespace().count()
                        })
                    } else {
                        json!({ "error": "Invalid text format, expected string" })
                    }
                } else {
                    json!({ "error": "Missing text field in input" })
                };
                
                Ok(result)
            }
            ProcessorType::Webhook => {
                // Webhook processor sends data to a specified URL
                let webhook_url = match job.input_data.get("webhook_url") {
                    Some(url_value) => match url_value.as_str() {
                        Some(url) => url,
                        None => return Err(anyhow::anyhow!("webhook_url must be a string"))
                    },
                    None => return Err(anyhow::anyhow!("webhook_url is required for webhook jobs"))
                };
                
                // Create payload with datetime and "hello world" value
                let payload = json!({
                    "datetime": chrono::Utc::now().to_rfc3339(),
                    "value": "hello world"
                });
                
                // Send the webhook request
                tracing::info!("Sending webhook to URL: {}", webhook_url);
                tracing::info!("Webhook payload: {}", payload);
                
                // Use reqwest to make the HTTP POST request
                let client = reqwest::Client::new();
                let response = match tokio::time::timeout(
                    std::time::Duration::from_secs(10),
                    client.post(webhook_url)
                        .json(&payload)
                        .send()
                ).await {
                    Ok(result) => match result {
                        Ok(resp) => resp,
                        Err(e) => return Err(anyhow::anyhow!("Failed to send webhook: {}", e))
                    },
                    Err(_) => return Err(anyhow::anyhow!("Webhook request timed out after 10 seconds"))
                };
                
                // Check if the request was successful
                let status = response.status();
                let status_code = status.as_u16();
                
                if status.is_success() {
                    // Return the result of the webhook call
                    let response_text = response.text().await
                        .unwrap_or_else(|_| "No response body".to_string());
                    
                    Ok(json!({
                        "webhook_url": webhook_url,
                        "payload": payload,
                        "status": "success",
                        "status_code": status_code,
                        "response": response_text
                    }))
                } else {
                    // Return error information
                    Err(anyhow::anyhow!("Webhook request failed with status: {}", status))
                }
            }
            ProcessorType::ExternalApi => {
                // External API processor not implemented in Phase 1
                Err(anyhow::anyhow!("External API processor not implemented in Phase 1"))
            }
            ProcessorType::Batch => {
                // Batch processor not implemented in Phase 1
                Err(anyhow::anyhow!("Batch processor not implemented in Phase 1"))
            }
        }
    }
}

#[async_trait::async_trait]
impl JobProcessor for DefaultJobProcessor {
    async fn process_job(&self, job: Job) -> anyhow::Result<(serde_json::Value, i32)> {
        // Reserve funds for the job
        self.reserve_funds(&job).await?;
        
        // Get the customer details (for future use in Phase 2)
        let _customer = self.customer_repo.find_by_id(job.customer_id).await?;
        
        // Process the job based on its type
        let output = self.process_job_type(&job, job.job_type_id).await?;
        
        // Calculate the actual cost (in Phase 1, use the estimated cost)
        let cost_cents = job.estimated_cost_cents;
        
        // Charge the customer's wallet
        self.charge_wallet(&job, cost_cents, true).await?;
        
        // Return the output and cost
        Ok((output, cost_cents))
    }
}
