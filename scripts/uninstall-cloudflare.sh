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
