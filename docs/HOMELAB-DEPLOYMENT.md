# Homelab Deployment Guide for GameCode Web

## Overview
Secure deployment options for collaborating from your homelab without relying on third-party services.

## Recommended Solutions

### Option 1: Tailscale (Recommended for Small Teams)
**Perfect for: 1-20 collaborators**

Tailscale creates a secure mesh VPN between your devices. It's peer-to-peer, so no data goes through their servers after initial coordination.

**Pros:**
- Zero-config WireGuard VPN
- End-to-end encrypted
- Works behind any NAT
- Free for personal use (up to 20 devices)
- No port forwarding needed
- Built-in ACLs for access control

**Setup:**
1. Install Tailscale on your Mac mini:
   ```bash
   brew install tailscale
   tailscale up
   ```

2. Share your Tailscale network with collaborators:
   - They install Tailscale on their devices
   - You approve them in your admin console
   - They access GameCode at `http://your-tailscale-ip:8080`

3. Optional: Set up MagicDNS for friendly names like `http://gamecode:8080`

### Option 2: WireGuard VPN (Maximum Control)
**Perfect for: Tech-savvy teams who want full control**

Run your own WireGuard VPN server on your network.

**Pros:**
- Complete control over infrastructure
- Very fast and secure
- No third-party dependencies
- Can run on router or separate device

**Setup:**
1. Install WireGuard on a dedicated device or router
2. Generate keys for each collaborator
3. They connect via WireGuard and access `http://192.168.x.x:8080`

### Option 3: ZeroTier (Self-Hosted Controller Option)
**Perfect for: Larger teams, more complex networking**

Similar to Tailscale but with option to self-host the controller.

**Pros:**
- Can self-host entire infrastructure
- More advanced networking features
- Free for up to 25 nodes

**Setup:**
1. Use ZeroTier's hosted controller (easier) or self-host
2. Create a network
3. Join devices to network
4. Access via ZeroTier IP

### Option 4: SSH Tunneling (Quick & Simple)
**Perfect for: Temporary access, tech-savvy users**

Each collaborator creates their own secure tunnel.

**Setup for collaborators:**
```bash
# On collaborator's machine
ssh -L 8080:localhost:8080 your-username@your-public-ip
# Then access http://localhost:8080
```

**Pros:**
- No additional software needed
- Very secure
- Easy to revoke access

**Cons:**
- Requires SSH access to your network
- Not as user-friendly

## Security Hardening for Any Solution

### 1. Add IP Allowlisting to GameCode
Create a middleware to restrict access to known IPs:

```rust
// In server/src/auth.rs
pub async fn ip_whitelist_middleware(
    State(allowed_ips): State<HashSet<IpAddr>>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request,
    next: Next,
) -> Response {
    if allowed_ips.contains(&addr.ip()) {
        next.run(request).await
    } else {
        StatusCode::FORBIDDEN.into_response()
    }
}
```

### 2. Enhanced Authentication
- Use unique passwords per collaborator
- Implement API keys instead of single password
- Add 2FA support

### 3. Access Logging
```toml
# In config/default.toml
[security]
log_access = true
log_retention_days = 30
alert_on_suspicious = true
```

### 4. Rate Limiting
Already mentioned in your code - implement it!

## Tailscale Setup Script

Here's a script to automate Tailscale setup:

```bash
#!/bin/bash
# save as scripts/setup-tailscale.sh

echo "🔒 Setting up Tailscale for GameCode Web"

# Install Tailscale if not present
if ! command -v tailscale &> /dev/null; then
    echo "Installing Tailscale..."
    curl -fsSL https://tailscale.com/install.sh | sh
fi

# Start Tailscale
echo "Starting Tailscale..."
sudo tailscale up

# Get Tailscale IP
TAILSCALE_IP=$(tailscale ip -4)
echo "✅ Tailscale IP: $TAILSCALE_IP"

# Update GameCode config to listen on Tailscale interface
echo "Updating GameCode configuration..."
cat > /tmp/tailscale-config.toml << EOF
# Tailscale configuration
[server]
host = "0.0.0.0"  # Listen on all interfaces
port = 8080

[security]
# Only allow connections from Tailscale network
allowed_networks = ["100.64.0.0/10"]
EOF

echo ""
echo "✅ Setup complete!"
echo ""
echo "Next steps:"
echo "1. Share your Tailscale network with collaborators"
echo "2. Have them install Tailscale and join your network"
echo "3. They can access GameCode at: http://$TAILSCALE_IP:8080"
echo ""
echo "For extra security, enable Tailscale ACLs in the admin console"
```

## Monitoring & Maintenance

### Service Health Check
```bash
#!/bin/bash
# save as scripts/health-check.sh

# Check if service is running
if launchctl list | grep -q "com.gamecode.web"; then
    echo "✅ Service is running"
else
    echo "❌ Service is not running"
    exit 1
fi

# Check if port is listening
if lsof -i :8080 | grep -q LISTEN; then
    echo "✅ Port 8080 is listening"
else
    echo "❌ Port 8080 is not listening"
    exit 1
fi

# Check Ollama
if curl -s http://localhost:11434/api/tags > /dev/null; then
    echo "✅ Ollama is responding"
else
    echo "❌ Ollama is not responding"
    exit 1
fi

# Check disk space
DISK_USAGE=$(df -h / | awk 'NR==2 {print $5}' | sed 's/%//')
if [ $DISK_USAGE -lt 90 ]; then
    echo "✅ Disk usage is ${DISK_USAGE}%"
else
    echo "⚠️  Disk usage is ${DISK_USAGE}%"
fi
```

### Backup Script
```bash
#!/bin/bash
# save as scripts/backup.sh

BACKUP_DIR="$HOME/gamecode-backups"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

mkdir -p "$BACKUP_DIR"

# Backup config
cp -r /usr/local/etc/gamecode-web "$BACKUP_DIR/config_$TIMESTAMP"

# Backup logs
tar -czf "$BACKUP_DIR/logs_$TIMESTAMP.tar.gz" /usr/local/var/log/gamecode-web

echo "✅ Backup created: $BACKUP_DIR/*_$TIMESTAMP"

# Clean old backups (keep last 7)
ls -t "$BACKUP_DIR"/config_* | tail -n +8 | xargs rm -rf
ls -t "$BACKUP_DIR"/logs_* | tail -n +8 | xargs rm -f
```

## Recommended: Tailscale + Enhanced Auth

For your homelab with collaborators, I recommend:

1. **Use Tailscale** for network access (it's really that good)
2. **Keep the JWT auth** as second layer
3. **Add user management** if you want per-user access

This gives you:
- No exposed ports to the internet
- End-to-end encryption
- Easy user management
- No ongoing costs
- No dependency on external services (after initial setup)

Would you like me to create the enhanced authentication system with multiple users, or help set up the Tailscale integration?