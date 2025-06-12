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

# Complete installation (build, install files, restart service)
install: build
	@echo "🚀 Installing GameCode Web..."
	@echo "This will require your sudo password to install system files."
	@echo ""
	
	# Install binary
	@echo "📦 Installing server binary..."
	sudo cp target/release/gamecode-server /usr/local/bin/gamecode-web
	
	# Install static files
	@echo "📄 Installing web files..."
	sudo rm -rf /usr/local/share/gamecode-web
	sudo mkdir -p /usr/local/share/gamecode-web
	sudo cp -r dist/* /usr/local/share/gamecode-web/
	
	# Install config files
	@echo "⚙️  Installing configuration..."
	sudo mkdir -p /usr/local/etc/gamecode-web
	@if [ ! -f /usr/local/etc/gamecode-web/config.toml ]; then \
		sudo cp config/default.toml /usr/local/etc/gamecode-web/config.toml; \
		echo "   Created new config.toml"; \
	else \
		echo "   Keeping existing config.toml"; \
	fi
	
	# Install prompts config
	@echo "💬 Installing prompts configuration..."
	@if [ ! -f /usr/local/etc/gamecode-web/prompts.toml ]; then \
		sudo cp config/prompts.toml /usr/local/etc/gamecode-web/prompts.toml; \
		echo "   Created new prompts.toml"; \
	else \
		echo "   Keeping existing prompts.toml (edit it to add custom prompts)"; \
	fi
	
	# Update config to use installed paths
	@echo "🔧 Updating configuration paths..."
	sudo sed -i '' 's|static_dir = "dist"|static_dir = "/usr/local/share/gamecode-web"|' /usr/local/etc/gamecode-web/config.toml
	
	# Log directory will use /tmp instead for simplicity
	
	# Install or update service
	@echo "🎯 Installing service..."
	@if [ -f ~/Library/LaunchAgents/com.gamecode.web.plist ]; then \
		echo "   Stopping existing service..."; \
		launchctl stop com.gamecode.web 2>/dev/null || true; \
		launchctl unload ~/Library/LaunchAgents/com.gamecode.web.plist 2>/dev/null || true; \
	fi
	
	# Create service plist
	@./scripts/create-service-plist.sh
	
	# Load and start service
	@echo "🚀 Starting service..."
	launchctl load ~/Library/LaunchAgents/com.gamecode.web.plist
	launchctl start com.gamecode.web
	
	# Wait a moment
	@sleep 2
	
	# Check status
	@echo ""
	@echo "✅ Installation complete!"
	@echo ""
	@if launchctl list | grep -q "com.gamecode.web"; then \
		echo "🟢 Service is running"; \
		echo "🌐 Local access: http://localhost:8080"; \
		echo ""; \
		echo "📝 To edit AI personas: sudo vim /usr/local/etc/gamecode-web/prompts.toml"; \
		echo "🔄 To restart after changes: make restart"; \
		echo "📊 To view logs: make logs"; \
	else \
		echo "❌ Service failed to start!"; \
		echo "Check logs: tail -f /tmp/gamecode-*.log"; \
		exit 1; \
	fi

# Quick restart service
restart:
	@echo "🔄 Restarting GameCode Web service..."
	launchctl stop com.gamecode.web || true
	@sleep 1
	launchctl start com.gamecode.web
	@sleep 1
	@if launchctl list | grep -q "com.gamecode.web"; then \
		echo "✅ Service restarted successfully"; \
	else \
		echo "❌ Service failed to start!"; \
	fi

# Service management
install-service: install
	@echo "Note: Use 'make install' instead"

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
	@tail -n 20 /tmp/gamecode-output.log 2>/dev/null || echo "No logs found"

service-logs:
	tail -f /tmp/gamecode-*.log

logs: service-logs

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