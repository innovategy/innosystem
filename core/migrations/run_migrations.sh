#!/bin/bash
set -e

echo "Running migrations and seeding the database..."

# Run the SQL migration directly
echo "Creating database tables..."
PGPASSWORD=postgres psql -h postgres -U postgres -d innosystem -c "
-- Create job_types table
CREATE TABLE IF NOT EXISTS job_types (
    id UUID PRIMARY KEY,
    name VARCHAR NOT NULL,
    description TEXT,
    processor_type VARCHAR NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Create customers table
CREATE TABLE IF NOT EXISTS customers (
    id UUID PRIMARY KEY,
    name VARCHAR NOT NULL,
    email VARCHAR NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Create wallets table
CREATE TABLE IF NOT EXISTS wallets (
    id UUID PRIMARY KEY,
    customer_id UUID NOT NULL REFERENCES customers(id),
    balance_cents INTEGER NOT NULL DEFAULT 0,
    reserved_cents INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Create wallet_transactions table
CREATE TABLE IF NOT EXISTS wallet_transactions (
    id UUID PRIMARY KEY,
    wallet_id UUID NOT NULL REFERENCES wallets(id),
    amount_cents INTEGER NOT NULL,
    transaction_type VARCHAR NOT NULL,
    description TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT NOW()
);

-- Create jobs table
CREATE TABLE IF NOT EXISTS jobs (
    id UUID PRIMARY KEY,
    job_type_id UUID NOT NULL REFERENCES job_types(id),
    customer_id UUID NOT NULL REFERENCES customers(id),
    status VARCHAR NOT NULL,
    input JSONB,
    output JSONB,
    error TEXT,
    started_at TIMESTAMP,
    completed_at TIMESTAMP,
    cost_cents INTEGER,
    created_at TIMESTAMP NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP NOT NULL DEFAULT NOW()
);
"

# Seed the database with initial data
echo "Seeding database with initial data..."

# Insert job types
echo "Inserting job types..."
PGPASSWORD=postgres psql -h postgres -U postgres -d innosystem -c "
INSERT INTO job_types (id, name, description, processor_type, enabled) VALUES 
('11111111-1111-1111-1111-111111111111', 'Basic Processing', 'Simple processing job', 'standard', true),
('22222222-2222-2222-2222-222222222222', 'Premium Processing', 'Enhanced processing job', 'premium', true),
('33333333-3333-3333-3333-333333333333', 'Advanced Analysis', 'In-depth analysis job', 'advanced', true),
('44444444-4444-4444-4444-444444444444', 'Experimental Processing', 'Experimental features', 'experimental', false)
ON CONFLICT (id) DO UPDATE SET
name = EXCLUDED.name,
description = EXCLUDED.description,
processor_type = EXCLUDED.processor_type,
enabled = EXCLUDED.enabled,
updated_at = NOW();"

# Insert customers
echo "Inserting customers..."
PGPASSWORD=postgres psql -h postgres -U postgres -d innosystem -c "
INSERT INTO customers (id, name, email) VALUES 
('aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'Test User', 'test@example.com'),
('bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'Demo User', 'demo@example.com'),
('cccccccc-cccc-cccc-cccc-cccccccccccc', 'Sample Corp', 'info@samplecorp.com')
ON CONFLICT (id) DO UPDATE SET
name = EXCLUDED.name,
email = EXCLUDED.email,
updated_at = NOW();"

# Insert wallets
echo "Inserting wallets..."
PGPASSWORD=postgres psql -h postgres -U postgres -d innosystem -c "
INSERT INTO wallets (id, customer_id, balance_cents) VALUES 
('eeeeeeee-eeee-eeee-eeee-eeeeeeeeeeee', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 10000),
('ffffffff-ffff-ffff-ffff-ffffffffffff', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 5000),
('dddddddd-dddd-dddd-dddd-dddddddddddd', 'cccccccc-cccc-cccc-cccc-cccccccccccc', 25000)
ON CONFLICT (id) DO UPDATE SET
customer_id = EXCLUDED.customer_id,
balance_cents = EXCLUDED.balance_cents,
updated_at = NOW();"

# Insert some wallet transactions
echo "Inserting wallet transactions..."
PGPASSWORD=postgres psql -h postgres -U postgres -d innosystem -c "
INSERT INTO wallet_transactions (id, wallet_id, amount_cents, transaction_type, description) VALUES 
('a1b2c3d4-e5f6-47a8-b9c0-d1e2f3a4b5c6', 'eeeeeeee-eeee-eeee-eeee-eeeeeeeeeeee', 10000, 'deposit', 'Initial deposit'),
('b2c3d4e5-f6a7-48b9-c0d1-e2f3a4b5c6d7', 'ffffffff-ffff-ffff-ffff-ffffffffffff', 5000, 'deposit', 'Initial deposit'),
('c3d4e5f6-a7b8-49c0-d1e2-f3a4b5c6d7e8', 'dddddddd-dddd-dddd-dddd-dddddddddddd', 25000, 'deposit', 'Initial deposit')
ON CONFLICT (id) DO NOTHING;"

# Insert some jobs
echo "Inserting jobs..."
PGPASSWORD=postgres psql -h postgres -U postgres -d innosystem -c "
INSERT INTO jobs (id, job_type_id, customer_id, status, input, cost_cents) VALUES 
('d4e5f6a7-b8c9-40d1-e2f3-a4b5c6d7e8f9', '11111111-1111-1111-1111-111111111111', 'aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa', 'pending', '{\"data\": \"test\"}', 100),
('e5f6a7b8-c9d0-41e2-f3a4-b5c6d7e8f9a0', '22222222-2222-2222-2222-222222222222', 'bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb', 'pending', '{\"data\": \"premium test\"}', 500),
('f6a7b8c9-d0e1-42f3-a4b5-c6d7e8f9a0b1', '33333333-3333-3333-3333-333333333333', 'cccccccc-cccc-cccc-cccc-cccccccccccc', 'pending', '{\"data\": \"advanced test\"}', 1000)
ON CONFLICT (id) DO UPDATE SET
job_type_id = EXCLUDED.job_type_id,
customer_id = EXCLUDED.customer_id,
status = EXCLUDED.status,
input = EXCLUDED.input,
cost_cents = EXCLUDED.cost_cents,
updated_at = NOW();"

echo "Migration and seeding completed successfully."
