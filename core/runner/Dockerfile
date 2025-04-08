FROM rust:1.85.1-slim-bookworm as builder

# Install build dependencies
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev libpq-dev && \
    rm -rf /var/lib/apt/lists/*

# Create the working directory for our application
WORKDIR /usr/src/runner

# Copy the entire project
COPY . .

# Build the application with verbose output
RUN cargo build --release --bin innosystem-runner --verbose && \
    # Debug: List what was built
    ls -la target/release/

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y ca-certificates libpq-dev && rm -rf /var/lib/apt/lists/*

# Copy the binary using the correct path
WORKDIR /usr/local/bin
COPY --from=builder /usr/src/runner/target/release/innosystem-runner .

# Set the entrypoint
CMD ["innosystem-runner"]
