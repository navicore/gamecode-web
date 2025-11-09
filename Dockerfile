# Multi-stage build for Rust web application
FROM rust:1.88 AS builder

# Install build tools once and cache in layer
RUN cargo install trunk --locked && \
    rustup target add wasm32-unknown-unknown

WORKDIR /app

# Copy only manifest files first for better layer caching
COPY Cargo.toml Cargo.lock ./
COPY server/Cargo.toml server/
COPY client/Cargo.toml client/
COPY build.rs ./

# Create dummy source files to build dependencies
RUN mkdir -p src server/src client/src && \
    echo "fn main() {}" > src/main.rs && \
    echo "fn main() {}" > server/src/main.rs && \
    echo "fn main() {}" > client/src/main.rs

# Build dependencies only (will be cached unless Cargo.toml changes)
RUN cargo build --release --manifest-path server/Cargo.toml || true

# Copy actual source code
COPY src ./src
COPY server/src ./server/src
COPY client/src ./client/src
COPY client/index.html ./client/
COPY client/Trunk.toml ./client/
COPY config ./config
COPY client/config ./client/config

# Build frontend
WORKDIR /app/client
RUN trunk build --release

# Build backend (dependencies already compiled)
WORKDIR /app
RUN cargo build --release --manifest-path server/Cargo.toml

# Final stage - minimal runtime image
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy artifacts from builder
COPY --from=builder /app/target/release/gamecode-server /app/
COPY --from=builder /app/dist /app/dist
COPY config /app/config

# Create non-root user
RUN useradd -m -u 1001 gamecode && chown -R gamecode:gamecode /app
USER gamecode

# Expose port
EXPOSE 8080

# Set environment variables
ENV RUST_LOG=info
ENV GAMECODE_CONFIG=/app/config/default.toml

# Run the server
CMD ["./gamecode-server"]
