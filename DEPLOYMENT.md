# GameCode Web Deployment Plan

## Overview
Deploy GameCode Web as a permanently running service on macOS with secure internet access from behind NAT.

## Current Architecture
- **Backend**: Rust/Axum server on port 8080
- **Frontend**: Leptos WASM served by backend
- **AI Provider**: Local Ollama (already running as service)
- **Authentication**: JWT with password protection

## Internet Access Options

### Option 1: ngrok (Current Approach)
**Pros:**
- Easy setup, already mentioned in README
- Handles SSL/TLS automatically
- Provides stable subdomain with paid plan
- No firewall/router configuration needed

**Cons:**
- Requires ngrok agent running
- Free tier has limitations (random URLs, connection limits)
- Monthly cost for stable subdomain (~$10/month)

**Setup for Production:**
1. Get ngrok paid account for stable subdomain
2. Configure ngrok with authtoken
3. Create ngrok config file with permanent tunnel
4. Run ngrok as a service

### Option 2: Cloudflare Tunnel (Recommended)
**Pros:**
- Free for personal use
- Stable subdomain on your domain
- Built-in DDoS protection
- No exposed ports
- Better performance than ngrok

**Cons:**
- Requires a domain name
- Initial setup more complex

**Setup:**
1. Sign up for Cloudflare (free)
2. Add your domain to Cloudflare
3. Install cloudflared
4. Create tunnel and route to localhost:8080

### Option 3: Tailscale
**Pros:**
- Zero-config VPN
- Very secure (WireGuard based)
- Free for personal use
- No public exposure

**Cons:**
- Only accessible from devices on your Tailscale network
- Not truly "public" internet access

## Recommended Approach

For a permanently running, internet-accessible service, I recommend:

1. **Cloudflare Tunnel** for public access
2. **macOS launchd** for service management
3. **Caddy or nginx** as reverse proxy (optional, for additional features)

## Implementation Plan

### Phase 1: Local Service Setup
1. Create launchd plist for GameCode Web
2. Ensure proper permissions and paths
3. Configure automatic restart on failure
4. Set up log rotation

### Phase 2: Internet Access
1. Choose between ngrok or Cloudflare Tunnel
2. Configure authentication at tunnel level
3. Set up monitoring/alerts

### Phase 3: Hardening
1. Add rate limiting
2. Implement request logging
3. Set up backup/restore procedures
4. Monitor resource usage

## Service Configuration Files Needed

### 1. Build Script (`scripts/build-release.sh`)
```bash
#!/bin/bash
cd "$(dirname "$0")/.."
cargo build --release
cd client && trunk build --release
```

### 2. Launch Agent (`com.gamecode.web.plist`)
```xml
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
    <string>/Users/navicore/gamecode-web</string>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardErrorPath</key>
    <string>/usr/local/var/log/gamecode-web.err</string>
    <key>StandardOutPath</key>
    <string>/usr/local/var/log/gamecode-web.log</string>
</dict>
</plist>
```

### 3. Makefile additions
```makefile
install: build-release
	# Install binary
	sudo cp target/release/gamecode-server /usr/local/bin/gamecode-web
	
	# Create directories
	sudo mkdir -p /usr/local/var/log
	sudo mkdir -p /usr/local/etc/gamecode-web
	
	# Copy config
	sudo cp config/default.toml /usr/local/etc/gamecode-web/
	
	# Install launchd service
	cp com.gamecode.web.plist ~/Library/LaunchAgents/
	launchctl load ~/Library/LaunchAgents/com.gamecode.web.plist

uninstall:
	launchctl unload ~/Library/LaunchAgents/com.gamecode.web.plist
	rm ~/Library/LaunchAgents/com.gamecode.web.plist
	sudo rm /usr/local/bin/gamecode-web
	sudo rm -rf /usr/local/etc/gamecode-web

start:
	launchctl start com.gamecode.web

stop:
	launchctl stop com.gamecode.web

status:
	launchctl list | grep gamecode
```

## Next Steps

1. **Decide on tunneling solution** (ngrok vs Cloudflare)
2. **Test the service locally** with launchd
3. **Configure the tunnel** for production use
4. **Add monitoring** (e.g., UptimeRobot, Healthchecks.io)

## Security Considerations

1. **Use strong password** for JWT auth
2. **Enable HTTPS** at tunnel level
3. **Consider IP allowlisting** if limited users
4. **Regular security updates** for dependencies
5. **Log and monitor access** for anomalies

## Questions to Resolve

1. Do you have a domain name for Cloudflare Tunnel?
2. How many concurrent users expected?
3. Do you need public access or just personal?
4. Backup strategy for chat history?
5. Resource limits needed?