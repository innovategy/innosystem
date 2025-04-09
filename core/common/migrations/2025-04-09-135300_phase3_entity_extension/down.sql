-- Drop indexes
DROP INDEX IF EXISTS idx_wallet_transactions_job_id;
DROP INDEX IF EXISTS idx_runners_status;
DROP INDEX IF EXISTS idx_jobs_project_id;
DROP INDEX IF EXISTS idx_projects_customer_id;
DROP INDEX IF EXISTS idx_resellers_api_key;
DROP INDEX IF EXISTS idx_customers_api_key;
DROP INDEX IF EXISTS idx_customers_reseller_id;

-- Remove column job_id from wallet_transactions
ALTER TABLE wallet_transactions DROP COLUMN IF EXISTS job_id;
-- Remove column description from wallet_transactions
ALTER TABLE wallet_transactions DROP COLUMN IF EXISTS description;

-- Drop runners table
DROP TABLE IF EXISTS runners;

-- Remove column project_id from jobs
ALTER TABLE jobs DROP COLUMN IF EXISTS project_id;

-- Drop projects table
DROP TABLE IF EXISTS projects;

-- Remove column api_key from customers
ALTER TABLE customers DROP COLUMN IF EXISTS api_key;
-- Remove column reseller_id from customers
ALTER TABLE customers DROP COLUMN IF EXISTS reseller_id;

-- Drop resellers table
DROP TABLE IF EXISTS resellers;
