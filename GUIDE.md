# InnoSystem User Guide

This guide provides detailed information about the InnoSystem platform, including current capabilities, job types, and how to extend the system.

## Current Features and System Capabilities

### Core Functionality

1. **Job Queue System**
   - Multi-tenant job processing system
   - Support for different job types with varying processing requirements
   - Robust job lifecycle management (creation, execution, status tracking)
   - Job status tracking (Pending, Running, Succeeded, Failed)

2. **Customer Management**
   - Customer registration and profile management
   - Customer-to-job association for tracking ownership
   - Email-based customer identification

3. **Wallet System**
   - Balance tracking for each customer
   - Job cost deduction from customer wallets
   - Transaction history

4. **Job Type Management**
   - Multiple processing types: Sync, Async, and Batch
   - Cost definition for different job types
   - Enable/disable functionality for job types

5. **Runner Infrastructure**
   - Job execution engine
   - Processing logic for various job types
   - Status reporting back to the API

6. **Database Persistence**
   - PostgreSQL database with Diesel ORM
   - Migrations framework for schema management
   - Data seeding for initial setup

7. **API Endpoints**
   - RESTful API for all system operations
   - Health check endpoints
   - Job CRUD operations
   - Customer management endpoints
   - Job type management endpoints

### Technical Components

1. **API Service**
   - Handles incoming requests
   - Validates input data
   - Routes requests to appropriate handlers
   - Returns formatted responses

2. **Runner Service**
   - Processes jobs from the queue
   - Updates job status
   - Implements processor-specific logic

3. **Migrations Service**
   - Sets up the database schema
   - Seeds initial data
   - Used for database updates

4. **Tester Service**
   - Validates system functionality
   - Tests API endpoints
   - Confirms end-to-end workflows

## Currently Available Job Types

The system comes pre-configured with the following job types:

| Name | Description | Processing Logic ID | Processor Type | Cost (cents) | Enabled |
|------|-------------|---------------------|----------------|--------------|---------|
| Text Analysis | Analyze text documents for sentiment and key concepts | text-analysis-v1 | Async | 100 | Yes |
| Image Recognition | Process images to identify objects and scenes | image-recog-v2 | Async | 200 | Yes |
| Data Processing | Process structured data files | data-proc-v1 | Batch | 50 | Yes |
| Report Generation | Generate PDF reports from templates | report-gen-v1 | Sync | 75 | Yes |
| Email Processing | Process and categorize emails | email-proc-v1 | Batch | 25 | No |

## How to Add a New Job Type

Adding a new job type to the system involves two main steps:

### 1. Add the Job Type to the Database

You can add a new job type programmatically using the API:

```bash
curl -X POST http://localhost:8080/job-types \
  -H "Content-Type: application/json" \
  -d '{
    "name": "New Job Type Name",
    "description": "Description of what this job type does",
    "processing_logic_id": "unique-processor-id",
    "processor_type": "async",
    "standard_cost_cents": 150,
    "enabled": true
  }'
```

Or you can add it to the seed data in `core/common/src/seed.rs`:

```rust
NewJobType {
    id: Uuid::new_v4(),
    name: "New Job Type Name".to_string(),
    description: Some("Description of the job type".to_string()),
    processing_logic_id: "unique-processor-id".to_string(),
    processor_type: ProcessorType::Async.as_str().to_string(),
    standard_cost_cents: 150,
    enabled: true,
},
```

### 2. Implement the Processing Logic

For each job type, you need to implement the corresponding processing logic in the runner:

1. Create a new processor module in `core/runner/src/processor/`:

```rust
// core/runner/src/processor/new_processor.rs

use async_trait::async_trait;
use common::models::job::Job;
use common::errors::Error;
use crate::processor::Processor;

pub struct NewProcessor;

#[async_trait]
impl Processor for NewProcessor {
    async fn process(&self, job: &Job) -> Result<(), Error> {
        // Implement job processing logic here
        tracing::info!("Processing job {} of type 'New Job Type'", job.id);
        
        // Your implementation goes here
        
        Ok(())
    }
}
```

2. Register the processor in the processor factory (`core/runner/src/processor/mod.rs`):

```rust
pub fn get_processor(processing_logic_id: &str) -> Option<Box<dyn Processor + Send + Sync>> {
    match processing_logic_id {
        // Existing processors
        "text-analysis-v1" => Some(Box::new(default::DefaultProcessor)),
        "image-recog-v2" => Some(Box::new(default::DefaultProcessor)),
        "data-proc-v1" => Some(Box::new(default::DefaultProcessor)),
        "report-gen-v1" => Some(Box::new(default::DefaultProcessor)),
        "email-proc-v1" => Some(Box::new(default::DefaultProcessor)),
        
        // Add your new processor
        "unique-processor-id" => Some(Box::new(new_processor::NewProcessor)),
        
        _ => None,
    }
}
```

### Important Notes

- The `processing_logic_id` must be unique and match between the database entry and the runner implementation
- The `processor_type` can be one of:
  - `sync`: Synchronous processing, completes immediately
  - `async`: Asynchronous processing, runs in the background
  - `batch`: Batch processing, handles multiple items at once
- Make sure to implement appropriate error handling in your processor
- Always test new job types with the tester service before deploying to production

## How to Create Additional Runners

The InnoSystem platform supports horizontal scaling through multiple runner instances. Here's how to set up additional runners:

### 1. Update the Docker Compose Configuration

Add a new runner service entry to the `docker-compose.yml` file:

```yaml
runner-2:
  build:
    context: .
    dockerfile: core/runner/Dockerfile
  depends_on:
    - postgres
    - redis
    - api
  environment:
    - RUST_LOG=debug
    - REDIS_URL=redis://redis:6379
    - DATABASE_URL=postgres://postgres:postgres@postgres:5432/innosystem
    - RUNNER_ID=runner-2
    - POLL_INTERVAL_MS=5000
  volumes:
    - ./logs:/logs
```

### 2. Configure Runner Specialization (Optional)

You can configure runners to specialize in certain job types by adding environment variables:

```yaml
runner-specialized:
  build:
    context: .
    dockerfile: core/runner/Dockerfile
  depends_on:
    - postgres
    - redis
    - api
  environment:
    - RUST_LOG=debug
    - REDIS_URL=redis://redis:6379
    - DATABASE_URL=postgres://postgres:postgres@postgres:5432/innosystem
    - RUNNER_ID=runner-specialized
    - POLL_INTERVAL_MS=5000
    - SUPPORTED_JOB_TYPES=image-recog-v2,text-analysis-v1
  volumes:
    - ./logs:/logs
```

### 3. Running Multiple Runners

Start the updated system with multiple runners:

```bash
docker compose up -d
```

### 4. Monitoring Runner Performance

Check the logs to see how jobs are distributed across runners:

```bash
docker compose logs -f runner runner-2 runner-specialized
```

### Best Practices for Runner Scaling

1. **Resource Allocation**: Ensure each runner has appropriate CPU and memory resources
2. **Job Type Specialization**: Consider dedicating runners to specific job types for optimization
3. **Monitoring**: Implement monitoring to track runner load and job distribution
4. **Redundancy**: Run multiple instances of each specialized runner for fault tolerance
5. **Scaling Policy**: Determine when to scale runners up or down based on job queue depth

## Advanced Configuration

### Job Polling Intervals

Adjust how frequently runners poll for new jobs:

```yaml
environment:
  - POLL_INTERVAL_MS=3000  # Poll every 3 seconds
```

### Concurrent Job Processing

Control how many jobs a runner can process simultaneously:

```yaml
environment:
  - MAX_CONCURRENT_JOBS=5  # Process up to 5 jobs simultaneously
```

### Runner Load Balancing

The system automatically distributes jobs to available runners based on:
- Runner availability
- Runner specialization (if configured)
- Job priority
- Queue depth

## Troubleshooting

If you encounter issues with job processing, check:

1. Runner logs for errors: `docker compose logs -f runner`
2. Database connectivity from the runner
3. Redis connectivity for job queue access
4. Job type configuration in the database
5. Processor implementation for the specific job type

For detailed diagnostics, increase the logging level:

```yaml
environment:
  - RUST_LOG=trace
```

## Future Enhancements

The InnoSystem is designed for extensibility. Planned enhancements include:

1. Worker pool optimization for higher throughput
2. Advanced job scheduling with time-based execution
3. Webhook notifications for job status changes
4. Enhanced monitoring and metrics collection
5. Runner auto-scaling based on queue depth
