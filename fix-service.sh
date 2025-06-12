#!/bin/bash
# Fix the gamecode-web service

echo "Fixing gamecode-web service..."

# Stop the service
launchctl stop com.gamecode.web 2>/dev/null
launchctl unload ~/Library/LaunchAgents/com.gamecode.web.plist 2>/dev/null

# Create a working plist that points to the temp logs
cat > ~/Library/LaunchAgents/com.gamecode.web.plist << 'EOF'
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
    <string>/tmp/gamecode-error.log</string>
    <key>StandardOutPath</key>
    <string>/tmp/gamecode-output.log</string>
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

# Load and start
launchctl load ~/Library/LaunchAgents/com.gamecode.web.plist
launchctl start com.gamecode.web

echo "Waiting for service to start..."
sleep 3

# Check status
if launchctl list | grep -q "com.gamecode.web"; then
    echo "✅ Service restarted"
    echo ""
    echo "Check logs with:"
    echo "  tail -f /tmp/gamecode-output.log"
    echo "  tail -f /tmp/gamecode-error.log"
else
    echo "❌ Service failed to start"
fi