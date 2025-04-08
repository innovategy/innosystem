FROM rust:1.85.1-slim-bookworm as builder

# Install PostgreSQL client libraries required by Diesel ORM
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    libpq-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/api

# Copy the entire project structure
COPY . .

# Build the application with verbose output
RUN cargo build --release --bin innosystem-api --verbose && \
    # Debug: List what was built
    ls -la /usr/src/api/target/release/

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/local/bin

# Copy the binary from the builder stage
COPY --from=builder /usr/src/api/target/release/innosystem-api .

# Expose the port the app runs on
EXPOSE 8080

# Command to run the application
CMD ["innosystem-api"]
