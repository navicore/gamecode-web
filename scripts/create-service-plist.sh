#!/bin/bash
# Create the launchd plist for GameCode Web service

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