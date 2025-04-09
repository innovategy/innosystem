#!/bin/bash
set -e

echo "Running migrations and seeding the database..."

# Run the SQL migration directly
echo "Creating database tables..."
PGPASSWORD=postgres psql -h postgres -U postgres -d innosystem -f /app/migrations/2025-04-08-201700_initial_schema/up.sql

# Seed database with initial data (configurable)
if [ "${SEED_DATABASE:-true}" = "true" ]; then
  echo "Seeding database with initial data..."
  
  echo "Inserting job types..."
  PGPASSWORD=postgres psql -h postgres -U postgres -d innosystem -c "
  INSERT INTO job_types (id, name, description, processing_logic_id, processor_type, standard_cost_cents, enabled)
  VALUES 
    ('550e8400-e29b-41d4-a716-446655440000', 'Text Analysis', 'Analyze text for sentiment and keywords', 'text-analysis-v1', 'async', 500, true),
    ('660e8400-e29b-41d4-a716-446655440000', 'Image Tagging', 'Tag images with AI-powered recognition', 'image-tagging-v1', 'async', 1000, true),
    ('770e8400-e29b-41d4-a716-446655440000', 'Data Processing', 'Process data files with transformation rules', 'data-processing-v1', 'batch', 2000, true),
    ('880e8400-e29b-41d4-a716-446655440000', 'Quick Check', 'Perform fast validation checks on inputs', 'quick-check-v1', 'sync', 100, true)
  ON CONFLICT (id) DO NOTHING;
  "
  
else
  echo "Skipping database seeding as per configuration."
fi

echo "Migration completed successfully."
