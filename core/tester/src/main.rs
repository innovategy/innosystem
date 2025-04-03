use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use anyhow::{Context, Result};
use chrono::Utc;
use clap::Parser;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};
use uuid::Uuid;

/// Command line arguments for the tester application
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// Base URL of the API to test
    #[clap(short, long, default_value = "http://localhost:8080")]
    api_url: String,

    /// Output directory for log files
    #[clap(short, long, default_value = "./logs")]
    output_dir: String,
}

/// Response structure for health endpoint
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct HealthResponse {
    status: String,
}

/// Request structure for creating a customer
#[derive(Debug, Serialize)]
struct CreateCustomerRequest {
    name: String,
    email: String,
    phone: Option<String>,
}

/// Response structure for customer endpoints
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct CustomerResponse {
    id: Uuid,
    name: String,
    email: String,
    phone: Option<String>,
    created_at: Option<String>,
    updated_at: Option<String>,
}

/// Request structure for creating a job type
#[derive(Debug, Serialize)]
struct CreateJobTypeRequest {
    name: String,
    description: String,
    processor_type: String,
    processing_logic_id: Option<Uuid>,
    standard_cost_cents: i32,
    enabled: bool,
}

/// Response structure for job type endpoints
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct JobTypeResponse {
    id: Uuid,
    name: String,
    description: String,
    created_at: Option<String>,
    updated_at: Option<String>,
}

/// Request structure for creating a job
#[derive(Debug, Serialize)]
struct CreateJobRequest {
    customer_id: Uuid,
    job_type_id: Uuid,
    priority: i32,
    input_data: serde_json::Value,
}

/// Response structure for job endpoints
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct JobResponse {
    id: Uuid,
    customer_id: Uuid,
    job_type_id: Uuid,
    status: String,
    priority: i32,
    input_data: serde_json::Value,
    output_data: Option<serde_json::Value>,
    created_at: Option<String>,
    updated_at: Option<String>,
    started_at: Option<String>,
    completed_at: Option<String>,
}

/// API Client for testing
struct ApiClient {
    client: reqwest::Client,
    base_url: String,
    log_file: File,
}

impl ApiClient {
    /// Create a new ApiClient instance
    fn new(base_url: String, log_path: &Path) -> Result<Self> {
        let client = reqwest::Client::builder()
            .build()
            .context("Failed to create HTTP client")?;

        let log_file = File::create(log_path)
            .context("Failed to create log file")?;

        Ok(Self {
            client,
            base_url,
            log_file,
        })
    }

    /// Log a message to the log file
    fn log(&mut self, message: &str) -> Result<()> {
        let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S");
        writeln!(self.log_file, "[{}] {}", timestamp, message)
            .context("Failed to write to log file")
    }

    /// Log a separator line to visually break sections
    fn log_separator(&mut self) -> Result<()> {
        self.log("\n----------------------------------------\n")
    }

    /// Test the health endpoint
    async fn test_health(&mut self) -> Result<()> {
        self.log("Testing health endpoint...")?;
        
        let url = format!("{}/health", self.base_url);
        let response = self.client.get(&url).send().await?;
        
        let status = response.status();
        self.log(&format!("Health status code: {}", status))?;
        
        if status.is_success() {
            let health: HealthResponse = response.json().await?;
            self.log(&format!("Health response: {:?}", health))?;
            info!("Health check succeeded");
        } else {
            let error_text = response.text().await?;
            self.log(&format!("Health check failed: {}", error_text))?;
            error!("Health check failed: {}", error_text);
        }
        
        Ok(())
    }

    /// Test customer endpoints
    async fn test_customers(&mut self) -> Result<Uuid> {
        self.log("Testing customer endpoints...")?;
        
        // Create a customer
        let customer_request = CreateCustomerRequest {
            name: "Test Customer".to_string(),
            email: format!("test_{}@example.com", Utc::now().timestamp()),
            phone: Some("+1234567890".to_string()),
        };
        
        self.log(&format!("Creating customer: {:?}", customer_request))?;
        
        let url = format!("{}/customers", self.base_url);
        let response = self.client
            .post(&url)
            .json(&customer_request)
            .send()
            .await?;
        
        let status = response.status();
        self.log(&format!("Create customer status code: {}", status))?;
        
        let customer: CustomerResponse = if status.is_success() {
            let customer = response.json().await?;
            self.log(&format!("Created customer: {:?}", customer))?;
            info!("Customer creation succeeded");
            customer
        } else {
            let error_text = response.text().await?;
            self.log(&format!("Customer creation failed: {}", error_text))?;
            error!("Customer creation failed: {}", error_text);
            return Err(anyhow::anyhow!("Failed to create customer"));
        };
        
        // Get the customer
        self.log(&format!("Fetching customer with ID: {}", customer.id))?;
        
        let url = format!("{}/customers/{}", self.base_url, customer.id);
        let response = self.client.get(&url).send().await?;
        
        let status = response.status();
        self.log(&format!("Get customer status code: {}", status))?;
        
        if status.is_success() {
            let fetched_customer: CustomerResponse = response.json().await?;
            self.log(&format!("Fetched customer: {:?}", fetched_customer))?;
            info!("Customer fetch succeeded");
        } else {
            let error_text = response.text().await?;
            self.log(&format!("Customer fetch failed: {}", error_text))?;
            warn!("Customer fetch failed: {}", error_text);
        }
        
        // Get all customers
        self.log("Fetching all customers")?;
        
        let url = format!("{}/customers", self.base_url);
        let response = self.client.get(&url).send().await?;
        
        let status = response.status();
        self.log(&format!("Get all customers status code: {}", status))?;
        
        if status.is_success() {
            let customers: Vec<CustomerResponse> = response.json().await?;
            self.log(&format!("Fetched {} customers", customers.len()))?;
            info!("Fetched {} customers", customers.len());
        } else {
            let error_text = response.text().await?;
            self.log(&format!("Get all customers failed: {}", error_text))?;
            warn!("Get all customers failed: {}", error_text);
        }
        
        Ok(customer.id)
    }

    /// Test job type endpoints
    async fn test_job_types(&mut self) -> Result<Uuid> {
        self.log("Testing job type endpoints...")?;
        
        // Create a webhook job type
        let job_type_request = CreateJobTypeRequest {
            name: format!("Webhook Job Type {}", Utc::now().timestamp()),
            description: "Sends data to a webhook".to_string(),
            processor_type: "webhook".to_string(),
            processing_logic_id: None,
            standard_cost_cents: 1000, // $10.00
            enabled: true,
        };
        
        self.log(&format!("Creating job type: {:?}", job_type_request))?;
        
        let url = format!("{}/job-types", self.base_url);
        let response = self.client
            .post(&url)
            .json(&job_type_request)
            .send()
            .await?;
        
        let status = response.status();
        self.log(&format!("Create job type status code: {}", status))?;
        
        let job_type: JobTypeResponse = if status.is_success() {
            let job_type = response.json().await?;
            self.log(&format!("Created job type: {:?}", job_type))?;
            info!("Job type creation succeeded");
            job_type
        } else {
            let error_text = response.text().await?;
            self.log(&format!("Job type creation failed: {}", error_text))?;
            error!("Job type creation failed: {}", error_text);
            return Err(anyhow::anyhow!("Failed to create job type"));
        };
        
        // Get the job type
        self.log(&format!("Fetching job type with ID: {}", job_type.id))?;
        
        let url = format!("{}/job-types/{}", self.base_url, job_type.id);
        let response = self.client.get(&url).send().await?;
        
        let status = response.status();
        self.log(&format!("Get job type status code: {}", status))?;
        
        if status.is_success() {
            let fetched_job_type: JobTypeResponse = response.json().await?;
            self.log(&format!("Fetched job type: {:?}", fetched_job_type))?;
            info!("Job type fetch succeeded");
        } else {
            let error_text = response.text().await?;
            self.log(&format!("Job type fetch failed: {}", error_text))?;
            warn!("Job type fetch failed: {}", error_text);
        }
        
        // Get all job types
        self.log("Fetching all job types")?;
        
        let url = format!("{}/job-types", self.base_url);
        let response = self.client.get(&url).send().await?;
        
        let status = response.status();
        self.log(&format!("Get all job types status code: {}", status))?;
        
        if status.is_success() {
            let job_types: Vec<JobTypeResponse> = response.json().await?;
            self.log(&format!("Fetched {} job types", job_types.len()))?;
            info!("Fetched {} job types", job_types.len());
        } else {
            let error_text = response.text().await?;
            self.log(&format!("Get all job types failed: {}", error_text))?;
            warn!("Get all job types failed: {}", error_text);
        }
        
        Ok(job_type.id)
    }

    /// Test job endpoints
    async fn test_jobs(&mut self, customer_id: Uuid, job_type_id: Uuid) -> Result<()> {
        self.log("Testing job endpoints...")?;
        
        // Create a webhook job
        let job_request = CreateJobRequest {
            customer_id,
            job_type_id,
            priority: 1, // Normal priority
            input_data: serde_json::json!({
                "webhook_url": "https://webhook.site/1e6c6b1a-552b-48a5-9c8b-679298b249a7", // This is a test webhook URL
                "datetime": Utc::now().to_rfc3339(),
                "value": "hello world"
            }),
        };
        
        self.log(&format!("Creating job: {:?}", job_request))?;
        
        let url = format!("{}/jobs", self.base_url);
        let response = self.client
            .post(&url)
            .json(&job_request)
            .send()
            .await?;
        
        let status = response.status();
        self.log(&format!("Create job status code: {}", status))?;
        
        let job: JobResponse = if status.is_success() {
            let job = response.json().await?;
            self.log(&format!("Created job: {:?}", job))?;
            info!("Job creation succeeded");
            job
        } else {
            let error_text = response.text().await?;
            self.log(&format!("Job creation failed: {}", error_text))?;
            error!("Job creation failed: {}", error_text);
            return Err(anyhow::anyhow!("Failed to create job"));
        };
        
        // Get the job
        self.log(&format!("Fetching job with ID: {}", job.id))?;
        
        let url = format!("{}/jobs/{}", self.base_url, job.id);
        let response = self.client.get(&url).send().await?;
        
        let status = response.status();
        self.log(&format!("Get job status code: {}", status))?;
        
        if status.is_success() {
            let fetched_job: JobResponse = response.json().await?;
            self.log(&format!("Fetched job: {:?}", fetched_job))?;
            info!("Job fetch succeeded");
        } else {
            let error_text = response.text().await?;
            self.log(&format!("Job fetch failed: {}", error_text))?;
            warn!("Job fetch failed: {}", error_text);
        }
        
        // Get all jobs
        self.log("Fetching all jobs")?;
        
        let url = format!("{}/jobs", self.base_url);
        let response = self.client.get(&url).send().await?;
        
        let status = response.status();
        self.log(&format!("Get all jobs status code: {}", status))?;
        
        if status.is_success() {
            let jobs: Vec<JobResponse> = response.json().await?;
            self.log(&format!("Fetched {} jobs", jobs.len()))?;
            info!("Fetched {} jobs", jobs.len());
            
            // Check if our job is in the queue
            let found = jobs.iter().any(|j| j.id == job.id);
            self.log(&format!("Job {} found in job list: {}", job.id, found))?;
            
            // Wait for a bit to let the runner process the job
            self.log("Waiting for 5 seconds to allow the job runner to process the job...")?;
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            
            // Check job status after waiting
            let url = format!("{}/jobs/{}", self.base_url, job.id);
            let response = self.client.get(&url).send().await?;
            
            if response.status().is_success() {
                let updated_job: JobResponse = response.json().await?;
                self.log(&format!("Job status after waiting: {}", updated_job.status))?;
                info!("Job status after waiting: {}", updated_job.status);
            }
        } else {
            let error_text = response.text().await?;
            self.log(&format!("Get all jobs failed: {}", error_text))?;
            warn!("Get all jobs failed: {}", error_text);
        }
        
        Ok(())
    }

    /// Run all tests
    async fn run_all_tests(&mut self) -> Result<()> {
        self.log("Starting API tests")?;
        self.log(&format!("Testing API at: {}", self.base_url))?;
        
        // Test health endpoint
        self.test_health().await?;
        self.log_separator()?;
        
        // Test customer endpoints
        let customer_id = match self.test_customers().await {
            Ok(id) => id,
            Err(e) => {
                error!("Customer tests failed: {}", e);
                self.log(&format!("Customer tests failed: {}", e))?;
                return Err(e);
            }
        };
        self.log_separator()?;
        
        // Test job type endpoints
        let job_type_id = match self.test_job_types().await {
            Ok(id) => id,
            Err(e) => {
                error!("Job type tests failed: {}", e);
                self.log(&format!("Job type tests failed: {}", e))?;
                return Err(e);
            }
        };
        self.log_separator()?;
        
        // Test job endpoints
        self.test_jobs(customer_id, job_type_id).await?;
        self.log_separator()?;
        
        self.log("All tests completed")?;
        info!("All tests completed");
        
        Ok(())
    }
}

/// Generate a random 5-character string
fn generate_random_suffix() -> String {
    let mut rng = rand::rng();
    (0..5)
        .map(|_| rng.sample(rand::distr::Alphanumeric) as char)
        .collect::<String>()
        .to_uppercase()
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let args = Args::parse();
    
    // Initialize tracing for console logging
    tracing_subscriber::fmt::init();
    
    // Create the output directory if it doesn't exist
    if !Path::new(&args.output_dir).exists() {
        fs::create_dir_all(&args.output_dir)?;
    }
    
    // Generate log file name with date and random suffix
    let date = Utc::now().format("%Y-%m-%d").to_string();
    let random_suffix = generate_random_suffix();
    let log_file_name = format!("{}-{}", date, random_suffix);
    let log_path = Path::new(&args.output_dir).join(format!("{}.log", log_file_name));
    
    info!("Starting API test run");
    info!("Log file: {}", log_path.display());
    
    // Create and run the API client
    let mut api_client = ApiClient::new(args.api_url, &log_path)?;
    
    match api_client.run_all_tests().await {
        Ok(_) => {
            info!("API test run completed successfully");
            println!("API test run completed successfully. Log file: {}", log_path.display());
            Ok(())
        }
        Err(e) => {
            error!("API test run failed: {}", e);
            println!("API test run failed: {}. Log file: {}", e, log_path.display());
            Err(e)
        }
    }
}
