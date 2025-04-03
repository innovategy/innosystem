FROM rust:1.85.1-slim-bookworm as builder

# Install build dependencies
RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev libpq-dev && \
    rm -rf /var/lib/apt/lists/*

# Create the working directory for our application
WORKDIR /usr/src/runner

# Copy source code
COPY core/common/src ./core/common/src/
COPY core/runner/src ./core/runner/src/

# Create standalone Cargo.toml files without workspace inheritance
RUN mkdir -p core/common core/runner

# Create a standalone common package
RUN echo '[package]\nname = "innosystem-common"\nversion = "0.1.0"\nedition = "2024"\n\n[dependencies]\ntokio = { version = "1.44.1", features = ["full"] }\nserde = { version = "1.0.219", features = ["derive"] }\nserde_json = "1.0.140"\nuuid = { version = "1.16.0", features = ["v4", "serde"] }\nchrono = { version = "0.4.40", features = ["serde"] }\ntracing = "0.1.41"\nanyhow = "1.0.97"\nthiserror = "2.0.12"\ndiesel = { version = "2.2.8", features = ["postgres", "uuid", "chrono", "serde_json", "r2d2"] }\nredis = { version = "0.29.2", features = ["tokio-comp", "connection-manager"] }\nbb8-redis = "0.21.0"\nasync-trait = "0.1.88"\nreqwest = { version = "0.11.27", features = ["json"] }' > core/common/Cargo.toml

# Create a standalone runner package
RUN echo '[package]\nname = "innosystem-runner"\nversion = "0.1.0"\nedition = "2024"\n\n[dependencies]\ninnosystem-common = { path = "../common" }\ntokio = { version = "1.44.1", features = ["full"] }\nserde = { version = "1.0.219", features = ["derive"] }\nserde_json = "1.0.140"\nuuid = { version = "1.16.0", features = ["v4", "serde"] }\nchrono = { version = "0.4.40", features = ["serde"] }\ntracing = "0.1.41"\ntracing-subscriber = { version = "0.3.19", features = ["env-filter", "json"] }\nanyhow = "1.0.97"\nthiserror = "2.0.12"\ndiesel = { version = "2.2.8", features = ["postgres", "uuid", "chrono", "serde_json", "r2d2"] }\ndotenvy = "0.15.7"\nredis = { version = "0.29.2", features = ["tokio-comp", "connection-manager"] }\nbb8-redis = "0.21.0"\nasync-trait = "0.1.88"\nreqwest = { version = "0.11.27", features = ["json"] }' > core/runner/Cargo.toml

# Build the application with verbose output
RUN cargo build --release --manifest-path=core/runner/Cargo.toml --verbose && \
    # Debug: List what was built
    ls -la /usr/src/runner/core/runner/target/release/

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y ca-certificates libpq-dev && rm -rf /var/lib/apt/lists/*

# Copy the binary using the correct path
WORKDIR /usr/local/bin
COPY --from=builder /usr/src/runner/core/runner/target/release/innosystem-runner .

# Set the entrypoint
CMD ["innosystem-runner"]
