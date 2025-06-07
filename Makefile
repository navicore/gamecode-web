.PHONY: all build run clean install-deps build-client build-server

# Default target
all: build

# Build everything
build: build-client build-server

# Build the client
build-client: check-trunk check-wasm-target
	@echo "Building client..."
	@echo "Cleaning old dist files..."
	@rm -rf dist/*
	@cd client && trunk build --release
	@echo "Client built. Files in dist:"
	@ls -la dist/

# Check if trunk is installed
check-trunk:
	@which trunk > /dev/null || (echo "trunk not found. Installing..." && cargo install trunk)

# Check if wasm target is installed
check-wasm-target:
	@rustup target list --installed | grep -q wasm32-unknown-unknown || \
		(echo "Installing wasm32-unknown-unknown target..." && \
		rustup target add wasm32-unknown-unknown)

# Build the server  
build-server:
	@echo "Building server..."
	cargo build --release -p gamecode-server

# Run the server (builds if needed)
run: build
	@echo "Starting server..."
	cargo run --release -p gamecode-server --bin gamecode-server

# Development mode - run without release flag
dev:
	cd client && trunk build
	cargo run -p gamecode-server --bin gamecode-server

# Clean build artifacts
clean:
	cargo clean
	rm -rf dist

# Install required tools
install-deps:
	rustup target add wasm32-unknown-unknown
	cargo install trunk
	cargo install cargo-watch

# Help
help:
	@echo "Available commands:"
	@echo "  make build    - Build client and server"
	@echo "  make run      - Build and run everything"
	@echo "  make dev      - Run in development mode"
	@echo "  make clean    - Clean build artifacts"
	@echo "  make install-deps - Install required tools"