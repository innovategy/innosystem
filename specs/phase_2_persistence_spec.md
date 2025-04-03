# Innovation Ecosystem Platform - Phase 2 Specification: Persistence & Robust Queueing

## 1. Phase Goal

Implement database persistence for core models using PostgreSQL and Diesel. Enhance the Redis queue with priority support. Introduce basic wallet balance tracking and cost deduction.

## 2. Phase Deliverables

*   Integrate PostgreSQL and Diesel ORM.
*   Create database migrations (`diesel_migrations`) for Customer, Wallet, JobType, Job models.
*   Implement `JobRepository`, `CustomerRepository`, and basic `WalletRepository` traits using Diesel.
*   Refactor API endpoints to persist/retrieve data from the database.
*   Enhance Redis queue logic to support job priorities.
*   Update Runner logic to pull from priority queues and update job status in the database.
*   Implement basic wallet balance deduction upon successful job completion.
*   **Functional Outcome:** Jobs, customers, and wallets are persisted in PostgreSQL. Jobs can be submitted with priorities, retrieved by runners based on priority, and their status is updated in the DB. Basic cost deduction occurs from the customer's wallet upon job success.

## 3. Technology Stack (Additions/Focus)

*   **Database**: PostgreSQL
*   **ORM**: Diesel.rs (`diesel`, `diesel_migrations`, `dotenvy`)
*   **DB Pooling**: `r2d2` or `deadpool-diesel`

## 4. Data Models & Database Schema

*   Define Diesel schema using `diesel print-schema > src/schema.rs` after migrations.
*   Annotate Phase 1 structs (`Customer`, `Wallet`, `JobType`, `Job`) with Diesel attributes (`#[derive(Queryable, Insertable, Identifiable)]`, etc.). Add necessary fields like `created_at`, `updated_at`.
*   Define `NewCustomer`, `NewWallet`, `NewJobType`, `NewJob` structs for insertion.
*   **Migrations (`diesel_migrations`):
    *   Initial migration for `customers` table (id, name, email, api_key [nullable for now], active, created_at, updated_at).
    *   Migration for `wallets` table (id, customer_id, current_balance_cents, pending_charges_cents [default 0], currency [default 'EUR'], created_at, updated_at).
    *   Migration for `job_types` table (id, name, processing_logic_id, standard_cost_cents, enabled, created_at, updated_at).
    *   Migration for `jobs` table (id, customer_id, job_type_id, status [enum/text], priority [integer], input_data [jsonb], output_data [jsonb, nullable], estimated_cost_cents, cost_cents [nullable], created_at, started_at [nullable], completed_at [nullable]).
    *   Add foreign key constraints (e.g., `jobs.customer_id` -> `customers.id`).
    *   Define `JobStatus` enum mapping if using native PG enums, or use TEXT representation.

```rust
// Example refinement (Job struct)
use diesel::prelude::*;
use chrono::NaiveDateTime;
use crate::schema::jobs;

#[derive(Queryable, Identifiable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = jobs)]
pub struct Job {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub job_type_id: Uuid,
    pub status: String, // Or custom Diesel enum type
    pub priority: i32,
    pub input_data: serde_json::Value,
    pub output_data: Option<serde_json::Value>,
    pub estimated_cost_cents: i32,
    pub cost_cents: Option<i32>,
    pub created_at: NaiveDateTime,
    pub started_at: Option<NaiveDateTime>,
    pub completed_at: Option<NaiveDateTime>,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = jobs)]
pub struct NewJob {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub job_type_id: Uuid,
    pub status: String,
    pub priority: i32,
    pub input_data: serde_json::Value,
    pub estimated_cost_cents: i32,
    // etc. (fields without defaults)
}
```

## 5. Repository Layer (Diesel Implementation)

*   Define traits `CustomerRepository`, `JobRepository`, `WalletRepository` (can be in a shared `core` crate).
*   Implement these traits using Diesel against a connection pool.
    *   `CustomerRepository`: `create`, `find_by_id`, `find_by_api_key` (basic versions).
    *   `JobRepository`: `create`, `find_by_id`, `update_status`, `set_started`, `set_completed`.
    *   `WalletRepository`: `find_by_customer_id`, `deduct_balance` (atomic update).
*   Handle Diesel errors and map them to application-specific errors.

## 6. API Layer (Axum Refinements)

*   Inject database connection pool into Axum state.
*   Refactor `POST /jobs` endpoint:
    1.  Accept `priority` in the request body (default if not provided).
    2.  Validate `customer_id` and `job_type_id` exist in the DB.
    3.  Fetch `standard_cost_cents` from `JobType` to set `estimated_cost_cents`.
    4.  Create `NewJob` struct.
    5.  Insert the new job into the `jobs` table using `JobRepository`.
    6.  If successful, push the `job_id` (UUID) onto the appropriate Redis priority queue.
    7.  Return the Job ID.
*   Add basic retrieval endpoints (optional but useful):
    *   `GET /jobs/:job_id`
    *   `GET /customers/:customer_id`

## 7. Redis Queue (Priority Implementation)

*   Define priority levels (e.g., 0=High, 1=Medium, 2=Low).
*   API pushes `job_id` to the corresponding list: `LPUSH jobs:p<priority>:pending <job_id_string>`.
*   Runner retrieves jobs using blocking pop across multiple lists in priority order: `BRPOP jobs:p0:pending jobs:p1:pending jobs:p2:pending 0`.
*   Store Job *ID* in Redis, not the full Job JSON. The runner will fetch the full job details from the DB using the ID.

## 8. Runner Process (DB Integration)

*   Inject database connection pool.
*   Modify loop:
    1.  Call `BRPOP` on priority queues to get a `job_id`.
    2.  Fetch the full `Job` details from the DB using `JobRepository::find_by_id`.
    3.  Update job status to `Running` and set `started_at` in the DB (`JobRepository::update_status`, `JobRepository::set_started`).
    4.  Log processing.
    5.  Simulate processing.
    6.  On success:
        *   Fetch `JobType.standard_cost_cents` (this might be refined in Phase 3).
        *   Update job status to `Succeeded`, set `completed_at`, set `cost_cents` in the DB.
        *   Call `WalletRepository::deduct_balance` for the `customer_id` and `cost_cents`. Handle potential insufficient funds error.
    7.  On failure:
        *   Update job status to `Failed`, set `completed_at` in the DB.
    8.  Handle DB errors throughout.

## 9. Configuration

*   Use `.env` file and `dotenvy` crate to manage `DATABASE_URL` and `REDIS_URL`.
*   Implement basic configuration loading for API and Runner.

## 10. Database Setup

*   Provide instructions (e.g., in README) for setting up PostgreSQL database, user, and running `diesel setup` and `diesel migration run`.
