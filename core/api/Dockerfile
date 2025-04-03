FROM rust:1.85.1-slim-bookworm as builder

# Install PostgreSQL client libraries required by Diesel ORM
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    libpq-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/api

# Copy source files
COPY core/common/src ./core/common/src/
COPY core/api/src ./core/api/src/

# Create directory structure
RUN mkdir -p core/common core/api

# Create standalone common package with dummy source file to help with dependency caching
RUN echo '[package]\nname = "innosystem-common"\nversion = "0.1.0"\nedition = "2024"\n\n[dependencies]\ntokio = { version = "1.44.1", features = ["full"] }\nserde = { version = "1.0.219", features = ["derive"] }\nserde_json = "1.0.140"\nuuid = { version = "1.16.0", features = ["v4", "serde"] }\nchrono = { version = "0.4.40", features = ["serde"] }\ntracing = "0.1.41"\nanyhow = "1.0.97"\nthiserror = "2.0.12"\ndiesel = { version = "2.2.8", features = ["postgres", "uuid", "chrono", "serde_json", "r2d2"] }\nredis = { version = "0.29.2", features = ["tokio-comp", "connection-manager"] }\nbb8-redis = "0.21.0"\nasync-trait = "0.1.88"' > core/common/Cargo.toml

# Create standalone api package with dummy source file to help with dependency caching
RUN echo '[package]\nname = "innosystem-api"\nversion = "0.1.0"\nedition = "2024"\n\n[dependencies]\ninnosystem-common = { path = "../common" }\ntokio = { version = "1.44.1", features = ["full"] }\naxum = { version = "0.8.3", features = ["macros"] }\naxum-extra = { version = "0.10.1", features = ["typed-header"] }\ntower = "0.5.2"\ntower-http = { version = "0.6.2", features = ["trace", "cors"] }\nserde = { version = "1.0.219", features = ["derive"] }\nserde_json = "1.0.140"\nuuid = { version = "1.16.0", features = ["v4", "serde"] }\nchrono = { version = "0.4.40", features = ["serde"] }\ntracing = "0.1.41"\ntracing-subscriber = { version = "0.3.19", features = ["env-filter", "json"] }\nanyhow = "1.0.97"\nthiserror = "2.0.12"\ndiesel = { version = "2.2.8", features = ["postgres", "uuid", "chrono", "serde_json", "r2d2"] }\ndiesel_migrations = "2.2.0"\ndotenvy = "0.15.7"\nr2d2 = "0.8.10"\nbb8-redis = "0.21.0"' > core/api/Cargo.toml

# Build the application with verbose output
RUN cargo build --release --manifest-path=core/api/Cargo.toml --verbose && \
    # Debug: List what was built
    ls -la /usr/src/api/core/api/target/release/

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/local/bin

# Copy the binary from the builder stage
COPY --from=builder /usr/src/api/core/api/target/release/innosystem-api .

# Expose the port the app runs on
EXPOSE 8080

# Command to run the application
CMD ["innosystem-api"]
