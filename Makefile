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

# Service management
install-service: build
	./scripts/install-service.sh

uninstall-service:
	./scripts/uninstall-service.sh

service-start:
	launchctl start com.gamecode.web

service-stop:
	launchctl stop com.gamecode.web

service-status:
	@echo "Service Status:"
	@launchctl list | grep gamecode || echo "Service not found"
	@echo "\nRecent Logs:"
	@tail -n 20 /usr/local/var/log/gamecode-web/output.log 2>/dev/null || echo "No logs found"

service-logs:
	tail -f /usr/local/var/log/gamecode-web/*.log

# Cloudflare Tunnel
setup-tunnel:
	./scripts/setup-cloudflare.sh

tunnel-status:
	@if pgrep -f "cloudflared tunnel run" > /dev/null; then \
		echo "✅ Cloudflare tunnel is running"; \
		echo "🌐 Access at: https://gamecode.navicore.tech"; \
	else \
		echo "❌ Cloudflare tunnel is not running"; \
		echo "Run 'make setup-tunnel' to configure"; \
	fi

# Help
help:
	@echo "Available commands:"
	@echo "  make build         - Build client and server"
	@echo "  make run           - Build and run everything"
	@echo "  make dev           - Run in development mode"
	@echo "  make clean         - Clean build artifacts"
	@echo "  make install-deps  - Install required tools"
	@echo ""
	@echo "Service commands:"
	@echo "  make install-service   - Install as macOS service"
	@echo "  make uninstall-service - Remove macOS service"
	@echo "  make service-start     - Start the service"
	@echo "  make service-stop      - Stop the service"
	@echo "  make service-status    - Check service status"
	@echo "  make service-logs      - Follow service logs"
	@echo ""
	@echo "Cloudflare Tunnel:"
	@echo "  make setup-tunnel      - Setup Cloudflare tunnel for gamecode.navicore.tech"
	@echo "  make tunnel-status     - Check tunnel status"