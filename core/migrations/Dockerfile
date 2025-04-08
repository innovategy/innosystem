FROM postgres:15

# Install dependencies
RUN apt-get update && \
    apt-get install -y bash && \
    rm -rf /var/lib/apt/lists/*

# Create app directory for the application
WORKDIR /app

# Copy the migration script
COPY core/migrations/run_migrations.sh /app/

# Make the script executable
RUN chmod +x /app/run_migrations.sh

# Set the entrypoint
ENTRYPOINT ["/app/run_migrations.sh"]
