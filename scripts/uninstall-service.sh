#!/bin/bash
# GameCode Web Service Uninstallation Script for macOS

set -e

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo "🗑️  GameCode Web Service Uninstaller"
echo "===================================="

# Unload service if running
if launchctl list | grep -q "com.gamecode.web"; then
    echo -e "${GREEN}Stopping and unloading service...${NC}"
    launchctl unload ~/Library/LaunchAgents/com.gamecode.web.plist
fi

# Remove files
echo -e "${GREEN}Removing files...${NC}"
rm -f ~/Library/LaunchAgents/com.gamecode.web.plist
sudo rm -f /usr/local/bin/gamecode-web
sudo rm -rf /usr/local/etc/gamecode-web
sudo rm -rf /usr/local/share/gamecode-web
sudo rm -rf /usr/local/var/log/gamecode-web

echo -e "${GREEN}✅ Service uninstalled successfully!${NC}"