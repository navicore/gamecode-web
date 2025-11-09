# Multi-stage build for Rust web application
FROM rust:1.88 AS builder

# Install build tools
RUN cargo install trunk --locked && \
    rustup target add wasm32-unknown-unknown

WORKDIR /app

# Copy everything
COPY . .

# Build frontend
WORKDIR /app/client
RUN trunk build --release

# Build backend
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
COPY --from=builder /app/config /app/config

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
