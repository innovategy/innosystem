FROM rust:1.85.1-slim-bookworm as builder

# Install PostgreSQL client libraries needed for diesel compilation
RUN apt-get update && \
    apt-get install -y libpq-dev pkg-config && \
    rm -rf /var/lib/apt/lists/*

# Set up the workspace structure
WORKDIR /app

# Copy the workspace root Cargo.toml and entire project structure
COPY Cargo.toml ./Cargo.toml
COPY ./core ./core

# Build for release
WORKDIR /app/core/migrations
RUN cargo build --release

# Create the runtime image
FROM debian:bookworm-slim

# Install necessary dependencies
RUN apt-get update && \
    apt-get install -y libpq5 ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Create the expected directory structure - this is critical
WORKDIR /app
RUN mkdir -p /app/core/common

# Copy the migration binary
COPY --from=builder /app/target/release/innosystem-migrations /usr/local/bin/migrations

# Copy the migration files to where the code expects them
COPY ./core/common/migrations /app/core/common/migrations

# Set environment variable to help with finding the right directory
ENV CARGO_MANIFEST_DIR=/app/core/common

# Set the entrypoint to run migrations
ENTRYPOINT ["migrations", "run"]