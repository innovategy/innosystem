# Innovation Ecosystem Platform - Phase 4 Specification: Advanced Features & Security Foundations

## 1. Phase Goal

Integrate foundational security elements (BYOK interface placeholder, basic access logging). Implement job retry policies based on `JobType` configuration.

## 2. Phase Deliverables

*   Define `KeyManagementService` and `AccessLoggingService` traits in the codebase.
*   Implement a *mock* or placeholder implementation for `KeyManagementService` (simulating interaction with an external KMS, no actual encryption yet).
*   Integrate the concept of a `secure_access` function wrapper around sensitive data operations (placeholder logging initially, no real decryption).
*   Implement basic `AccessLoggingService` (e.g., logging to console or a simple DB table) to record start/end of secure access calls.
*   Define `RetryPolicy` struct and add it (as JSONB) to the `job_types` table/model.
*   Implement job retry logic within the runner based on the `RetryPolicy`.
*   Update job status handling to include retry counts and potential delayed requeueing.
*   Add API endpoints related to job status querying (including retry info) and potential cancellation.
*   **Functional Outcome:** Jobs automatically retry on failure according to configured policy. Foundational traits and functions for BYOK and access logging are defined and integrated conceptually. Basic job cancellation is possible.

## 3. Technology Stack (Additions/Focus)

*   **Error Handling:** Refine application-specific error types (`Error`, `Result`).
*   **Async Task Management:** Potentially leverage specific Tokio utilities for managing retries/delays.

## 4. Data Models & Database Schema (Additions/Changes)

*   **Migrations (`diesel_migrations`):**
    *   Add `retry_policy` column (JSONB, nullable) to `job_types` table.
    *   Add `retry_count` column (INTEGER, default 0) to `jobs` table.
    *   Add `last_error` column (TEXT, nullable) to `jobs` table.
    *   (Optional) Create `access_logs` table (id, job_id, runner_id, timestamp, event_type [e.g., 'start_access', 'end_access'], details [jsonb]).
*   Update `JobType` struct to include `retry_policy: Option<RetryPolicy>`.
*   Update `Job` struct to include `retry_count: i32`, `last_error: Option<String>`.
*   Define `RetryPolicy` struct (serializable/deserializable with `serde`):
    ```rust
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct RetryPolicy {
        max_attempts: i32,
        initial_interval_seconds: i32,
        backoff_multiplier: f32, // e.g., 1.0 for linear, 2.0 for exponential
    }
    ```

## 5. Repository Layer (Additions/Changes)

*   `JobRepository`: Add methods like `increment_retry_count`, `update_last_error`, `find_cancellable_jobs`.
*   (Optional) Define and implement `AccessLogRepository` if using a DB table for access logs.

## 6. Core Logic / Traits (Security Foundations)

*   Define `core::security` module.
*   Define `KeyManagementService` trait (as per main spec, focusing on method signatures).
*   Define `AccessLoggingService` trait (as per main spec).
*   Implement mock/stub versions:
    *   `MockKms`: Implements `KeyManagementService`, logs calls, returns dummy data (e.g., `Ok(WrappedKey{...})`). Does not perform real crypto.
    *   `ConsoleAccessLogger` or `DbAccessLogger`: Implements `AccessLoggingService`, logs events to console or DB table.
*   Define `secure_access` function concept:
    ```rust
    // Simplified conceptual example
    async fn secure_access<T, F>(
        job_id: Uuid,
        runner_id: Uuid,
        kms: &impl KeyManagementService,
        logger: &impl AccessLoggingService,
        // ... other context
        access_function: F
    ) -> Result<T, Error>
    where
        F: FnOnce(/* &DecryptedData - notional for Phase 4 */) -> Result<T, Error>
    {
        let log_id = logger.create_access_record(/*...*/).await?;
        // Mock KMS interaction: kms.decrypt_data(...).await?;
        // In Phase 4, DecryptedData is just a placeholder or uses original input
        let result = access_function(/* placeholder data */);
        // Mock KMS interaction: Securely erase data
        logger.complete_access_record(log_id, /* status */).await?;
        result
    }
    ```
*   Conceptually integrate calls to `secure_access` where sensitive data handling *would* occur in the runner (but skip actual data handling for this phase).

## 7. Runner Process (Retry Logic)

*   Modify error handling within the main job processing loop:
    1.  When a processing error occurs:
        *   Fetch the `Job` and its associated `JobType` (including `retry_policy`).
        *   Check if a `retry_policy` exists and if `job.retry_count < policy.max_attempts`.
        *   If retry is possible:
            *   Increment `retry_count` in the DB (`JobRepository::increment_retry_count`).
            *   Store the error message in `last_error` (`JobRepository::update_last_error`).
            *   Calculate delay: `initial_interval * multiplier.powf(retry_count)`. Cap delay if necessary.
            *   Update job status to `PendingRetry` or similar (or keep as `Failed` but use retry count).
            *   **Requeue:** Instead of immediate requeue, consider:
                *   **Delayed Redis Key:** Add `job_id` to a Redis sorted set with score = `now + delay`. A separate task polls this set. (More complex).
                *   **Runner Delay:** Simply `tokio::time::sleep(delay)` within the runner before it attempts to fetch the *next* job. (Simpler, but blocks the runner).
                *   **Simple Requeue:** `LPUSH` back to the pending queue (might cause rapid retries without external delay mechanism). Choose simplest viable option for Phase 4 (e.g., simple requeue or runner sleep).
            *   Log the retry attempt and delay.
        *   If retry is not possible (no policy or max attempts reached):
            *   Update job status to `Failed` (permanently).
            *   Store final error in `last_error`.
            *   Log final failure.
    2.  When a job succeeds after retries, ensure final status is `Succeeded`.

## 8. API Layer (Additions)

*   **Job Status Endpoint (`GET /jobs/:job_id`):** Enhance response to include `retry_count` and `last_error` if present.
*   **Job Cancellation Endpoint (`POST /jobs/:job_id/cancel`):**
    *   Find the job by ID.
    *   Check if the job is in a cancellable state (`Pending`, `PendingRetry`).
    *   Update job status to `Cancelled` in the DB.
    *   **(Challenge):** If the job is already `Running`, cancellation is harder. Simplest approach for Phase 4: only allow cancellation of non-running jobs. More advanced: signal the runner (e.g., via Redis pub/sub or checking a flag in DB before starting work) - defer this complexity.
    *   Return success/failure status.

## 9. Configuration

*   (Optional) Add configuration for default retry behavior if `JobType.retry_policy` is null.
