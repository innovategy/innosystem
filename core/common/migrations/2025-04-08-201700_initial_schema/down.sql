-- Drop tables in reverse order of creation to respect foreign key constraints
DROP TABLE IF EXISTS wallet_transactions;
DROP TABLE IF EXISTS jobs;
DROP TABLE IF EXISTS wallets;
DROP TABLE IF EXISTS customers;
DROP TABLE IF EXISTS job_types;

-- Drop indexes (they will be dropped when tables are dropped, but for clarity)
DROP INDEX IF EXISTS idx_jobs_customer_id;
DROP INDEX IF EXISTS idx_jobs_job_type_id;
DROP INDEX IF EXISTS idx_jobs_status;
DROP INDEX IF EXISTS idx_wallets_customer_id;
DROP INDEX IF EXISTS idx_wallet_transactions_wallet_id;
DROP INDEX IF EXISTS idx_wallet_transactions_reference_id;
