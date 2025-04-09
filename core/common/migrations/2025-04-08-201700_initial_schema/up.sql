--------------------------------------------------------------------------------
-- INNOSYSTEM COMPLETE DATABASE SCHEMA
--------------------------------------------------------------------------------
-- This file contains the complete database schema for the InnoSystem platform
-- All tables, relationships, and indexes are defined in this single file
-- Last Updated: 2025-04-09
--------------------------------------------------------------------------------

--------------------------------------------------------------------------------
-- Phase 1: Core Tables
--------------------------------------------------------------------------------

-- Create the resellers table (moved before customers due to FK relationship)
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

-- Create the job_types table
CREATE TABLE IF NOT EXISTS job_types (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    processing_logic_id TEXT NOT NULL,  -- Required field identified in previous troubleshooting
    processor_type TEXT NOT NULL,       -- Must be one of: "sync", "async", or "batch"
    standard_cost_cents INTEGER NOT NULL, -- Required field identified in previous troubleshooting
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create the customers table
CREATE TABLE IF NOT EXISTS customers (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT NOT NULL UNIQUE,
    api_key TEXT UNIQUE,
    reseller_id UUID REFERENCES resellers(id) ON DELETE SET NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create the projects table
CREATE TABLE IF NOT EXISTS projects (
    id UUID PRIMARY KEY,
    customer_id UUID NOT NULL REFERENCES customers(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create the wallets table
CREATE TABLE IF NOT EXISTS wallets (
    id UUID PRIMARY KEY,
    customer_id UUID NOT NULL REFERENCES customers(id) ON DELETE CASCADE,
    balance_cents INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

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

-- Create the jobs table
CREATE TABLE IF NOT EXISTS jobs (
    id UUID PRIMARY KEY,
    job_type_id UUID NOT NULL REFERENCES job_types(id) ON DELETE RESTRICT,
    customer_id UUID NOT NULL REFERENCES customers(id) ON DELETE CASCADE,
    project_id UUID REFERENCES projects(id) ON DELETE SET NULL,
    status TEXT NOT NULL,
    cost_cents INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP
);

-- Create the wallet_transactions table
CREATE TABLE IF NOT EXISTS wallet_transactions (
    id UUID PRIMARY KEY,
    wallet_id UUID NOT NULL REFERENCES wallets(id) ON DELETE CASCADE,
    amount_cents INTEGER NOT NULL,
    transaction_type TEXT NOT NULL,
    description TEXT,
    reference_id UUID,
    job_id UUID REFERENCES jobs(id) ON DELETE SET NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

--------------------------------------------------------------------------------
-- Indexes for Query Optimization
--------------------------------------------------------------------------------

-- Job indexes
CREATE INDEX IF NOT EXISTS idx_jobs_customer_id ON jobs(customer_id);
CREATE INDEX IF NOT EXISTS idx_jobs_job_type_id ON jobs(job_type_id);
CREATE INDEX IF NOT EXISTS idx_jobs_status ON jobs(status);
CREATE INDEX IF NOT EXISTS idx_jobs_project_id ON jobs(project_id);

-- Wallet indexes
CREATE INDEX IF NOT EXISTS idx_wallets_customer_id ON wallets(customer_id);

-- Transaction indexes
CREATE INDEX IF NOT EXISTS idx_wallet_transactions_wallet_id ON wallet_transactions(wallet_id);
CREATE INDEX IF NOT EXISTS idx_wallet_transactions_reference_id ON wallet_transactions(reference_id);
CREATE INDEX IF NOT EXISTS idx_wallet_transactions_job_id ON wallet_transactions(job_id);

-- Customer indexes
CREATE INDEX IF NOT EXISTS idx_customers_reseller_id ON customers(reseller_id);
CREATE INDEX IF NOT EXISTS idx_customers_api_key ON customers(api_key);

-- Reseller indexes
CREATE INDEX IF NOT EXISTS idx_resellers_api_key ON resellers(api_key);

-- Project indexes
CREATE INDEX IF NOT EXISTS idx_projects_customer_id ON projects(customer_id);

-- Runner indexes
CREATE INDEX IF NOT EXISTS idx_runners_status ON runners(status);
