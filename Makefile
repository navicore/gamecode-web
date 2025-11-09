.PHONY: all build run clean install-deps build-client build-server dev help

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

# Development mode with auto-reload (requires cargo-watch)
watch:
	@echo "Starting development server with auto-reload..."
	@echo "Client will be built once, server will rebuild on changes"
	@cd client && trunk build
	@cargo watch -x 'run -p gamecode-server --bin gamecode-server'

# Clean build artifacts
clean:
	cargo clean
	rm -rf dist

# Install required tools
install-deps:
	rustup target add wasm32-unknown-unknown
	cargo install trunk
	cargo install cargo-watch

# Docker build (for testing locally before CI/CD)
docker-build:
	@echo "Building Docker image..."
	docker build -t gamecode-web:local .

# Docker run (for testing)
docker-run: docker-build
	@echo "Running Docker container..."
	docker run -p 8080:8080 gamecode-web:local

# Help
help:
	@echo "GameCode Web - Makefile"
	@echo ""
	@echo "Development commands:"
	@echo "  make build         - Build client (WASM) and server"
	@echo "  make run           - Build and run locally"
	@echo "  make dev           - Run in development mode (debug builds)"
	@echo "  make watch         - Run with auto-reload on code changes"
	@echo "  make clean         - Clean build artifacts"
	@echo "  make install-deps  - Install required tools (trunk, cargo-watch)"
	@echo ""
	@echo "Docker commands (for testing):"
	@echo "  make docker-build  - Build Docker image locally"
	@echo "  make docker-run    - Run Docker container locally"
	@echo ""
	@echo "Deployment:"
	@echo "  This project is deployed via k8s with Flux GitOps"
	@echo "  Push to main → GitHub Actions builds image → Flux deploys to k8s"
