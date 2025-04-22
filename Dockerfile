# Multi-stage build for RabbitMQ to Kafka bridge

# Build stage
FROM rust:1.77-slim-bookworm as builder

# Install build dependencies including CMake 3.5+
RUN apt-get update && apt-get install -y \
    cmake \
    pkg-config \
    libssl-dev \
    build-essential \
    git \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Check CMake version to ensure it's 3.5+
RUN cmake --version

# Create a new empty project
WORKDIR /app

# Copy over manifests and config files
COPY Cargo.toml ./
COPY config/ ./config/

# Copy source code
COPY src/ ./src/

# Build the application in release mode
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libssl-dev \
    ca-certificates \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user to run the application
RUN useradd -ms /bin/bash appuser

# Create app directory and set permissions
WORKDIR /app
COPY --from=builder /app/target/release/rabbitmq-kafka /app/
COPY --from=builder /app/config/ /app/config/
COPY .env.example /app/.env

# Set ownership to the non-root user
RUN chown -R appuser:appuser /app

# Switch to non-root user
USER appuser

# Set environment variables
ENV RUST_LOG=info

# Run the application
CMD ["/app/rabbitmq-kafka"]