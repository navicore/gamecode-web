#!/bin/bash
# GameCode Web Service Installation Script for macOS

set -e

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo "🚀 GameCode Web Service Installer"
echo "================================="

# Check if running on macOS
if [[ "$OSTYPE" != "darwin"* ]]; then
    echo -e "${RED}This script is for macOS only${NC}"
    exit 1
fi

# Get the script directory
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

cd "$PROJECT_DIR"

# Check prerequisites
echo "Checking prerequisites..."

if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Rust/Cargo not found. Please install Rust first.${NC}"
    exit 1
fi

if ! command -v trunk &> /dev/null; then
    echo -e "${RED}Trunk not found. Installing...${NC}"
    cargo install trunk
fi

if ! command -v ollama &> /dev/null; then
    echo -e "${RED}Warning: Ollama not found. The service will not work without Ollama.${NC}"
    read -p "Continue anyway? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Build the project
echo -e "${GREEN}Building release version...${NC}"
cargo build --release
cd client && trunk build --release && cd ..

# Create required directories
echo -e "${GREEN}Creating directories...${NC}"
sudo mkdir -p /usr/local/bin
sudo mkdir -p /usr/local/var/log/gamecode-web
sudo mkdir -p /usr/local/etc/gamecode-web

# Install binary
echo -e "${GREEN}Installing binary...${NC}"
sudo cp target/release/gamecode-server /usr/local/bin/gamecode-web

# Copy config
echo -e "${GREEN}Installing configuration...${NC}"
sudo cp config/default.toml /usr/local/etc/gamecode-web/config.toml

# Copy static files
echo -e "${GREEN}Copying static files...${NC}"
sudo rm -rf /usr/local/share/gamecode-web
sudo mkdir -p /usr/local/share/gamecode-web
sudo cp -r dist/* /usr/local/share/gamecode-web/

# Update config to use installed paths
sudo sed -i '' 's|static_dir = "dist"|static_dir = "/usr/local/share/gamecode-web"|' /usr/local/etc/gamecode-web/config.toml

# Create launchd plist
echo -e "${GREEN}Creating launch agent...${NC}"
cat > ~/Library/LaunchAgents/com.gamecode.web.plist << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.gamecode.web</string>
    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/bin/gamecode-web</string>
    </array>
    <key>WorkingDirectory</key>
    <string>/usr/local/share/gamecode-web</string>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <dict>
        <key>SuccessfulExit</key>
        <false/>
    </dict>
    <key>StandardErrorPath</key>
    <string>/usr/local/var/log/gamecode-web/error.log</string>
    <key>StandardOutPath</key>
    <string>/usr/local/var/log/gamecode-web/output.log</string>
    <key>EnvironmentVariables</key>
    <dict>
        <key>RUST_LOG</key>
        <string>info</string>
        <key>GAMECODE_CONFIG</key>
        <string>/usr/local/etc/gamecode-web/config.toml</string>
    </dict>
</dict>
</plist>
EOF

# Load the service
echo -e "${GREEN}Loading service...${NC}"
launchctl load ~/Library/LaunchAgents/com.gamecode.web.plist

# Wait a moment for service to start
sleep 2

# Check if service is running
if launchctl list | grep -q "com.gamecode.web"; then
    echo -e "${GREEN}✅ Service installed and started successfully!${NC}"
    echo ""
    echo "Service Management Commands:"
    echo "  Start:   launchctl start com.gamecode.web"
    echo "  Stop:    launchctl stop com.gamecode.web"
    echo "  Status:  launchctl list | grep gamecode"
    echo "  Logs:    tail -f /usr/local/var/log/gamecode-web/*.log"
    echo ""
    echo "The service is now running on http://localhost:8080"
    echo ""
    echo "For internet access, choose one of these options:"
    echo "  1. ngrok: ngrok http 8080"
    echo "  2. Cloudflare Tunnel: See DEPLOYMENT.md"
else
    echo -e "${RED}❌ Service failed to start. Check logs at /usr/local/var/log/gamecode-web/${NC}"
    exit 1
fi