use crate::errors::Error;
use crate::models::{
    customer::NewCustomer,
    job::{JobStatus, NewJob},
    job_type::{NewJobType, ProcessorType},
    wallet::NewWallet,
};
use crate::repositories::{
    customer::CustomerRepository,
    job::{JobFilter, JobRepository},
    job_type::JobTypeRepository,
    wallet::WalletRepository,
};
use std::sync::Arc;
use uuid::Uuid;

/// Seed struct that handles database seeding
pub struct Seeder {
    job_type_repo: Arc<dyn JobTypeRepository + Send + Sync>,
    customer_repo: Arc<dyn CustomerRepository + Send + Sync>,
    job_repo: Arc<dyn JobRepository + Send + Sync>,
    wallet_repo: Arc<dyn WalletRepository + Send + Sync>,
}

impl Seeder {
    /// Create a new seeder with repository implementations
    pub fn new(
        job_type_repo: Arc<dyn JobTypeRepository + Send + Sync>,
        customer_repo: Arc<dyn CustomerRepository + Send + Sync>,
        job_repo: Arc<dyn JobRepository + Send + Sync>,
        wallet_repo: Arc<dyn WalletRepository + Send + Sync>,
    ) -> Self {
        Self {
            job_type_repo,
            customer_repo,
            job_repo,
            wallet_repo,
        }
    }

    /// Run all seed operations
    pub async fn seed_all(&self) -> Result<(), Error> {
        // Seed in order to respect foreign key constraints
        self.seed_job_types().await?;
        self.seed_customers().await?;
        self.seed_wallets().await?;
        self.seed_jobs().await?;

        Ok(())
    }

    /// Seed job types
    pub async fn seed_job_types(&self) -> Result<(), Error> {
        // Check if any job types exist to make this operation idempotent
        let existing = self.job_type_repo.list_all().await?;
        if !existing.is_empty() {
            return Ok(());
        }

        // Define seed job types
        let job_types = vec![
            NewJobType {
                id: Uuid::new_v4(),
                name: "Text Analysis".to_string(),
                description: Some("Analyze text documents for sentiment and key concepts".to_string()),
                processing_logic_id: "text-analysis-v1".to_string(),
                processor_type: ProcessorType::Async.as_str().to_string(),
                standard_cost_cents: 100,
                enabled: true,
            },
            NewJobType {
                id: Uuid::new_v4(),
                name: "Image Recognition".to_string(),
                description: Some("Process images to identify objects and scenes".to_string()),
                processing_logic_id: "image-recog-v2".to_string(),
                processor_type: ProcessorType::Async.as_str().to_string(),
                standard_cost_cents: 200,
                enabled: true,
            },
            NewJobType {
                id: Uuid::new_v4(),
                name: "Data Processing".to_string(),
                description: Some("Process structured data files".to_string()),
                processing_logic_id: "data-proc-v1".to_string(),
                processor_type: ProcessorType::Batch.as_str().to_string(),
                standard_cost_cents: 50,
                enabled: true,
            },
            NewJobType {
                id: Uuid::new_v4(),
                name: "Report Generation".to_string(),
                description: Some("Generate PDF reports from templates".to_string()),
                processing_logic_id: "report-gen-v1".to_string(),
                processor_type: ProcessorType::Sync.as_str().to_string(),
                standard_cost_cents: 75,
                enabled: true,
            },
            NewJobType {
                id: Uuid::new_v4(),
                name: "Email Processing".to_string(),
                description: Some("Process and categorize emails".to_string()),
                processing_logic_id: "email-proc-v1".to_string(),
                processor_type: ProcessorType::Batch.as_str().to_string(),
                standard_cost_cents: 25,
                enabled: false, // This one is disabled for testing
            },
        ];

        // Insert each job type
        for job_type in job_types {
            self.job_type_repo.create(job_type).await?;
        }

        Ok(())
    }

    /// Seed customers
    pub async fn seed_customers(&self) -> Result<(), Error> {
        // Check if any customers exist to make this operation idempotent
        let existing = self.customer_repo.list_all().await?;
        if !existing.is_empty() {
            return Ok(());
        }

        // Define seed customers
        let customers = vec![
            NewCustomer {
                id: Uuid::new_v4(),
                name: "Acme Corporation".to_string(),
                email: "contact@acme.example.com".to_string(),
            },
            NewCustomer {
                id: Uuid::new_v4(),
                name: "TechStart Inc.".to_string(),
                email: "info@techstart.example.com".to_string(),
            },
            NewCustomer {
                id: Uuid::new_v4(),
                name: "Global Services Ltd.".to_string(),
                email: "support@globalservices.example.com".to_string(),
            },
        ];

        // Insert each customer
        for customer in customers {
            self.customer_repo.create(customer).await?;
        }

        Ok(())
    }

    /// Seed wallets
    pub async fn seed_wallets(&self) -> Result<(), Error> {
        // Since we don't have a list_all method for wallets, we'll check for each customer
        // Get all customers to create wallets for them
        let customers = self.customer_repo.list_all().await?;

        // Create a wallet for each customer if they don't already have one
        for customer in customers {
            // Try to find existing wallet for customer
            let wallet_result = self.wallet_repo.find_by_customer_id(customer.id).await;
            if wallet_result.is_ok() {
                // Wallet already exists for this customer
                continue;
            }
            
            let new_wallet = NewWallet {
                id: Uuid::new_v4(),
                customer_id: customer.id,
                balance_cents: 10000, // Start with $100 balance
            };

            self.wallet_repo.create(new_wallet).await?;
        }

        Ok(())
    }

    /// Seed jobs
    pub async fn seed_jobs(&self) -> Result<(), Error> {
        // Check if any jobs exist to make this operation idempotent
        // Use query_jobs with empty filter to check for existing jobs
        let (existing, _) = self.job_repo.query_jobs(JobFilter::default(), None, None).await?;
        if !existing.is_empty() {
            return Ok(());
        }

        // Get job types and customers
        let job_types = self.job_type_repo.list_all().await?;
        let customers = self.customer_repo.list_all().await?;

        // Ensure we have job types and customers
        if job_types.is_empty() || customers.is_empty() {
            return Ok(());
        }

        // Create some sample jobs with different statuses
        let mut jobs = Vec::new();

        // For each customer, create some jobs
        for customer in &customers {
            // Create jobs with different statuses for testing
            for status in [JobStatus::Pending, JobStatus::Running, JobStatus::Succeeded, JobStatus::Failed].iter() {
                // Use a random job type for each job
                for job_type in &job_types {
                    let job = NewJob {
                        id: Uuid::new_v4(),
                        job_type_id: job_type.id,
                        customer_id: customer.id,
                        status: status.as_str().to_string(),
                        cost_cents: job_type.standard_cost_cents,
                    };

                    jobs.push(job);
                }
            }
        }

        // Insert each job
        for job in jobs {
            self.job_repo.create(job).await?;
        }

        Ok(())
    }
}
