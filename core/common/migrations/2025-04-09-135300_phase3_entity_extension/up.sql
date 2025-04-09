-- Create the resellers table
CREATE TABLE IF NOT EXISTS resellers (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT NOT NULL UNIQUE,
    api_key TEXT NOT NULL UNIQUE,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    commission_rate INTEGER NOT NULL DEFAULT 0, -- Stored as basis points (1/100 of a percent), e.g., 1000 = 10.00%
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Modify customers table to add reseller_id (nullable)
ALTER TABLE customers ADD COLUMN IF NOT EXISTS reseller_id UUID REFERENCES resellers(id) ON DELETE SET NULL;
ALTER TABLE customers ADD COLUMN IF NOT EXISTS api_key TEXT UNIQUE;

-- Create the projects table
CREATE TABLE IF NOT EXISTS projects (
    id UUID PRIMARY KEY,
    customer_id UUID NOT NULL REFERENCES customers(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Modify jobs table to add project_id (nullable)
ALTER TABLE jobs ADD COLUMN IF NOT EXISTS project_id UUID REFERENCES projects(id) ON DELETE SET NULL;

-- Create the runners table
CREATE TABLE IF NOT EXISTS runners (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL DEFAULT 'inactive',
    compatible_job_types TEXT[] NOT NULL DEFAULT '{}',
    last_heartbeat TIMESTAMP,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Add description field to wallet_transactions if it doesn't exist
ALTER TABLE wallet_transactions ADD COLUMN IF NOT EXISTS description TEXT;
-- Add job_id field to wallet_transactions if it doesn't exist
ALTER TABLE wallet_transactions ADD COLUMN IF NOT EXISTS job_id UUID REFERENCES jobs(id) ON DELETE SET NULL;

-- Create indexes for better query performance
CREATE INDEX IF NOT EXISTS idx_customers_reseller_id ON customers(reseller_id);
CREATE INDEX IF NOT EXISTS idx_customers_api_key ON customers(api_key);
CREATE INDEX IF NOT EXISTS idx_resellers_api_key ON resellers(api_key);
CREATE INDEX IF NOT EXISTS idx_projects_customer_id ON projects(customer_id);
CREATE INDEX IF NOT EXISTS idx_jobs_project_id ON jobs(project_id);
CREATE INDEX IF NOT EXISTS idx_runners_status ON runners(status);
CREATE INDEX IF NOT EXISTS idx_wallet_transactions_job_id ON wallet_transactions(job_id);
