FROM rust:1.85.1-slim-bookworm as builder

# Install build dependencies
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Create the working directory for our application
WORKDIR /usr/src/tester

# Copy the entire project
COPY . .

# Build the tester binary
RUN cargo build --bin innosystem-tester --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Create directories for logs
RUN mkdir -p /app/logs

# Set the working directory
WORKDIR /app

# Copy the binary from the builder stage
COPY --from=builder /usr/src/tester/target/release/innosystem-tester /usr/local/bin/

# Set the entrypoint
ENTRYPOINT ["/usr/local/bin/innosystem-tester", "--api-url", "http://api:8080", "--output-dir", "/app/logs"]
