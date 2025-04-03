# Innovation Ecosystem Platform - Phase 1 Specification: Core Foundation & Basic Job Flow

## 1. Phase Goal

Establish the basic project structure, core data model structs (in-memory initially), and a minimal end-to-end job submission/processing flow using a simple Redis queue and a basic runner.

## 2. Phase Deliverables

*   Rust project setup (`Cargo.toml` with initial dependencies).
*   Define core data model structs (minimal versions for Job, JobType, Customer, Wallet).
*   Basic API structure (using Axum) with a single endpoint to submit a simple job.
*   Simple Redis queue implementation (LPUSH/BRPOP).
*   A basic, hardcoded runner process that polls Redis, logs job processing, and marks it complete (in-memory status).
*   Basic logging setup (`tracing`).
*   **Functional Outcome:** Ability to submit a job via API and see it logged as processed by a local runner.

## 3. Technology Stack (Phase 1 Focus)

*   **Backend**: Rust
*   **Web Framework**: Axum
*   **Async Runtime**: Tokio
*   **Serialization**: Serde
*   **Messaging/Queue**: Redis (via `redis-rs`)
*   **Logging**: `tracing` / `tracing-subscriber`
*   **IDs**: `uuid`

## 4. Data Models (Minimal In-Memory Structs)

Define these structs in Rust. Persistence is not required in this phase.

```rust
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap; // Or similar for in-memory 'status' tracking

// Minimal representation, no DB integration yet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Customer {
    id: Uuid,
    name: String,
    // Other fields deferred
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    id: Uuid,
    customer_id: Uuid,
    balance_cents: i64, // Simple balance for now
    // Other fields deferred
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobType {
    id: Uuid,
    name: String,
    processing_logic_id: String, // Identifier for hardcoded runner logic
    // Other fields deferred
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobStatus {
    Pending,
    Running,
    Succeeded, // Simple outcome for Phase 1
    Failed,    // Simple outcome for Phase 1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    id: Uuid,
    customer_id: Uuid,
    job_type_id: Uuid,
    status: JobStatus,
    input_data: serde_json::Value, // Flexible input
    // Output, cost, timestamps, etc., deferred
}
```

## 5. API Layer (Axum)

*   Setup basic Axum application structure.
*   Implement health check endpoint (`GET /health`).
*   Implement job submission endpoint:
    *   **Endpoint:** `POST /jobs`
    *   **Request Body:**
        ```json
        {
          "customer_id": "uuid-string",
          "job_type_id": "uuid-string",
          "input_data": { ... } // Arbitrary JSON
        }
        ```
    *   **Logic:**
        1.  Generate a new UUID for the Job ID.
        2.  Create a `Job` struct with `status: JobStatus::Pending`.
        3.  Serialize the `Job` struct to JSON.
        4.  Push the JSON string onto a Redis list (e.g., `jobs:pending`).
        5.  Return the new Job ID and `201 Created` status.
    *   **Error Handling:** Basic validation (e.g., presence of fields), Redis connection errors.

## 6. Redis Queue Implementation

*   Use the `redis-rs` crate.
*   Establish connection pooling (e.g., `bb8-redis` or `deadpool-redis`).
*   **Job Submission (API):** Use `LPUSH jobs:pending <job_json_string>`.
*   **Job Retrieval (Runner):** Use blocking `BRPOP jobs:pending 0` to wait for and retrieve jobs.

## 7. Basic Runner Process

*   Create a separate Rust binary application (`runner`).
*   Connect to Redis using the same configuration as the API.
*   Loop indefinitely:
    1.  Call `BRPOP jobs:pending 0` to wait for a job.
    2.  If a job JSON string is received:
        *   Deserialize the JSON string into a `Job` struct.
        *   Log the job details (e.g., "Processing job {job_id} for customer {customer_id} of type {job_type_id}").
        *   Simulate processing (e.g., `tokio::time::sleep`).
        *   Log completion (e.g., "Job {job_id} Succeeded").
        *   **(Optional In-Memory Status):** Could update a shared `HashMap<Uuid, JobStatus>` if running API/Runner in one process for simplicity, otherwise skip status update logic for Phase 1. Persistence/status updates deferred.
    3.  Handle potential deserialization errors or Redis errors.

## 8. Logging

*   Use the `tracing` crate for structured logging.
*   Configure a basic `tracing_subscriber` to output logs to the console.
*   Add informative logs in the API (request handling, Redis interactions) and Runner (job retrieval, processing start/end).

## 9. Project Setup

*   Create a Cargo workspace with two members: `api` and `runner`.
*   Define `Cargo.toml` for the workspace and individual crates.
*   **Dependencies (Workspace/Shared):** `tokio`, `serde`, `serde_json`, `uuid`, `redis`, `tracing`, `tracing-subscriber`.
*   **Dependencies (API):** `axum`.
*   **Dependencies (Runner):** None specific beyond shared.
*   Include a basic `.gitignore`.
*   (Optional) Basic `Dockerfile` stubs for `api` and `runner`.

