#!/bin/bash
# Cloudflare Tunnel Setup for gamecode.navicore.tech

set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

DOMAIN="gamecode.navicore.tech"
SERVICE_NAME="gamecode-tunnel"

echo -e "${BLUE}☁️  Cloudflare Tunnel Setup for GameCode Web${NC}"
echo "==========================================="
echo -e "Domain: ${GREEN}$DOMAIN${NC}"
echo ""

# Check if cloudflared is installed
if ! command -v cloudflared &> /dev/null; then
    echo -e "${YELLOW}Installing cloudflared...${NC}"
    if [[ "$OSTYPE" == "darwin"* ]]; then
        brew install cloudflare/cloudflare/cloudflared
    else
        # Linux
        wget -q https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-amd64.deb
        sudo dpkg -i cloudflared-linux-amd64.deb
        rm cloudflared-linux-amd64.deb
    fi
fi

echo -e "${GREEN}✅ cloudflared is installed${NC}"

# Check if already logged in
if ! cloudflared tunnel list &> /dev/null; then
    echo ""
    echo -e "${YELLOW}First-time setup required${NC}"
    echo "This will open a browser to authenticate with Cloudflare."
    echo "Make sure navicore.tech is already added to your Cloudflare account."
    echo ""
    read -p "Press Enter to continue..."
    
    cloudflared tunnel login
fi

# Check if tunnel already exists
if cloudflared tunnel list | grep -q "$SERVICE_NAME"; then
    echo -e "${YELLOW}Tunnel '$SERVICE_NAME' already exists${NC}"
    read -p "Delete and recreate? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        cloudflared tunnel delete "$SERVICE_NAME"
    else
        echo "Using existing tunnel"
    fi
fi

# Create tunnel if it doesn't exist
if ! cloudflared tunnel list | grep -q "$SERVICE_NAME"; then
    echo -e "${BLUE}Creating tunnel...${NC}"
    cloudflared tunnel create "$SERVICE_NAME"
fi

# Get tunnel ID
TUNNEL_ID=$(cloudflared tunnel list | grep "$SERVICE_NAME" | awk '{print $1}')
echo -e "${GREEN}Tunnel ID: $TUNNEL_ID${NC}"

# Create config directory
mkdir -p ~/.cloudflared

# Create config file
echo -e "${BLUE}Creating configuration...${NC}"
cat > ~/.cloudflared/config.yml << EOF
tunnel: $TUNNEL_ID
credentials-file: $HOME/.cloudflared/$TUNNEL_ID.json

ingress:
  - hostname: $DOMAIN
    service: http://localhost:8080
    originRequest:
      noTLSVerify: true
  - service: http_status:404
EOF

# Create DNS record
echo -e "${BLUE}Creating DNS record for $DOMAIN...${NC}"
if cloudflared tunnel route dns "$SERVICE_NAME" "$DOMAIN"; then
    echo -e "${GREEN}✅ DNS record created${NC}"
else
    echo -e "${YELLOW}⚠️  DNS record may already exist${NC}"
fi

# Test the tunnel
echo ""
echo -e "${BLUE}Testing tunnel configuration...${NC}"
echo "Starting tunnel in test mode (Ctrl+C to stop)..."
echo ""
cloudflared tunnel run "$SERVICE_NAME" &
TUNNEL_PID=$!

sleep 5

echo ""
echo -e "${GREEN}✅ Tunnel is running!${NC}"
echo ""
echo "Test URLs:"
echo -e "  ${BLUE}https://$DOMAIN${NC} (from internet)"
echo -e "  ${BLUE}http://localhost:8080${NC} (local)"
echo ""
echo -e "${YELLOW}Press Enter to stop test and install as service...${NC}"
read

kill $TUNNEL_PID 2>/dev/null || true
wait $TUNNEL_PID 2>/dev/null || true

# Install as service
echo ""
echo -e "${BLUE}Installing as service...${NC}"

if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS launchd service
    PLIST_FILE="$HOME/Library/LaunchAgents/com.cloudflare.gamecode.plist"
    
    cat > "$PLIST_FILE" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.cloudflare.gamecode</string>
    <key>ProgramArguments</key>
    <array>
        <string>/usr/local/bin/cloudflared</string>
        <string>tunnel</string>
        <string>run</string>
        <string>$SERVICE_NAME</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardErrorPath</key>
    <string>/usr/local/var/log/cloudflared.err</string>
    <key>StandardOutPath</key>
    <string>/usr/local/var/log/cloudflared.log</string>
</dict>
</plist>
EOF

    # Load the service
    launchctl unload "$PLIST_FILE" 2>/dev/null || true
    launchctl load "$PLIST_FILE"
    
    echo -e "${GREEN}✅ Service installed and started${NC}"
    echo ""
    echo "Service commands:"
    echo "  Start:   launchctl start com.cloudflare.gamecode"
    echo "  Stop:    launchctl stop com.cloudflare.gamecode"
    echo "  Logs:    tail -f /usr/local/var/log/cloudflared.log"
else
    # Linux systemd service
    sudo cloudflared service install
    sudo systemctl enable cloudflared
    sudo systemctl start cloudflared
    
    echo -e "${GREEN}✅ Service installed and started${NC}"
    echo ""
    echo "Service commands:"
    echo "  Status:  sudo systemctl status cloudflared"
    echo "  Logs:    sudo journalctl -u cloudflared -f"
fi

# Create uninstall script
cat > uninstall-cloudflare.sh << 'EOF'
#!/bin/bash
echo "🗑️  Uninstalling Cloudflare Tunnel..."

if [[ "$OSTYPE" == "darwin"* ]]; then
    launchctl unload ~/Library/LaunchAgents/com.cloudflare.gamecode.plist 2>/dev/null || true
    rm -f ~/Library/LaunchAgents/com.cloudflare.gamecode.plist
else
    sudo systemctl stop cloudflared
    sudo systemctl disable cloudflared
    sudo cloudflared service uninstall
fi

echo "✅ Service uninstalled"
echo ""
echo "Note: Tunnel and DNS records still exist in Cloudflare."
echo "To completely remove, run:"
echo "  cloudflared tunnel delete gamecode-tunnel"
EOF
chmod +x uninstall-cloudflare.sh

# Summary
echo ""
echo -e "${GREEN}🎉 Setup Complete!${NC}"
echo ""
echo -e "GameCode Web is now accessible at:"
echo -e "  ${BLUE}https://$DOMAIN${NC}"
echo ""
echo -e "${YELLOW}Important:${NC}"
echo "1. Make sure GameCode Web service is running:"
echo "   make service-status"
echo ""
echo "2. The tunnel will auto-start on boot"
echo ""
echo "3. To uninstall Cloudflare tunnel:"
echo "   ./uninstall-cloudflare.sh"
echo ""
echo "4. Access is still protected by your GameCode password"
echo ""
echo -e "${GREEN}Your collaborators can now access GameCode from anywhere!${NC}"