# Multi-stage build for Rust web application
FROM rust:1.88 AS builder

# Install trunk for frontend build
RUN cargo install trunk --locked
RUN rustup target add wasm32-unknown-unknown

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY server/Cargo.toml server/
COPY client/Cargo.toml client/

# Copy source code
COPY src ./src
COPY server/src ./server/src
COPY client/src ./client/src
COPY client/index.html ./client/
COPY client/Trunk.toml ./client/
COPY build.rs ./

# Copy config files
COPY config ./config
COPY client/config ./client/config

# Build frontend
WORKDIR /app/client
RUN trunk build --release

# Build backend
WORKDIR /app
RUN cargo build --release --manifest-path server/Cargo.toml

# Final stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary
COPY --from=builder /app/target/release/gamecode-server /app/

# Copy frontend dist
COPY --from=builder /app/dist /app/dist

# Copy config
COPY config /app/config

# Create a non-root user
RUN useradd -m -u 1001 gamecode && chown -R gamecode:gamecode /app
USER gamecode

# Expose port
EXPOSE 8080

# Set environment variables
ENV RUST_LOG=info
ENV GAMECODE_CONFIG=/app/config/default.toml

# Run the server
CMD ["./gamecode-server"]