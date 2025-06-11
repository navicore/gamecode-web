# Cloudflare for Homelab - Honest Assessment

## Cloudflare Tunnel Pricing

**The good news**: Cloudflare Tunnel (formerly Argo Tunnel) is **100% FREE** for your use case!

### What's Free:
- ✅ Cloudflare Tunnel (no limits on tunnels)
- ✅ Unlimited bandwidth
- ✅ DDoS protection
- ✅ SSL/TLS certificates
- ✅ Web Application Firewall (basic rules)
- ✅ Up to 50 users for Zero Trust access

### What Costs Money:
- ❌ Advanced Zero Trust features (you don't need)
- ❌ Advanced WAF rules (you don't need)
- ❌ Load balancing across multiple origins (you don't need)

## Requirements

You need:
1. A domain name (~$10-15/year from any registrar)
2. Free Cloudflare account (email only, no social login required)
3. `cloudflared` daemon on your Mac mini

## Simple Setup for Homelab

### 1. Get a Domain
- Buy from Namecheap, Porkbun, etc. (~$10/year for .com)
- Or use a free subdomain service like DuckDNS (less professional)

### 2. Add Domain to Cloudflare
```bash
# Sign up at cloudflare.com (just email/password)
# Add your domain
# Change nameservers at your registrar
```

### 3. Install cloudflared
```bash
brew install cloudflare/cloudflare/cloudflared
```

### 4. Create Tunnel
```bash
# Login (opens browser for one-time auth)
cloudflared tunnel login

# Create tunnel
cloudflared tunnel create gamecode-tunnel

# Create config file
cat > ~/.cloudflared/config.yml << EOF
url: http://localhost:8080
tunnel: <tunnel-id-from-above>
credentials-file: /Users/$(whoami)/.cloudflared/<tunnel-id>.json
EOF

# Create DNS route
cloudflared tunnel route dns gamecode-tunnel gamecode.yourdomain.com

# Run it
cloudflared tunnel run gamecode-tunnel
```

### 5. Install as Service
```bash
# Create launch agent
sudo cloudflared service install

# It's now running permanently!
```

## Access Control Options

### Option 1: Cloudflare Access (Still Free!)
Add email-based authentication in front of your app:
```bash
# In Cloudflare Dashboard > Zero Trust > Access > Applications
# Add application: gamecode.yourdomain.com
# Add policy: Allow emails in list
# Add your collaborators' emails
```

### Option 2: Just Use GameCode's Auth
Skip Cloudflare Access and rely on your existing JWT auth.

## Privacy Considerations

**What Cloudflare Sees:**
- Your domain name
- Traffic volume (not content - it's encrypted)
- Source IPs of visitors

**What Cloudflare DOESN'T See:**
- Your actual application data (end-to-end encrypted)
- Passwords
- Chat content

## Alternatives If You Still Don't Trust Cloudflare

### 1. Boring Tunnel (Self-Hosted)
```bash
# Your own tunnel server on a VPS
# https://github.com/boringstuff/tunnel
# ~$5/month for a small VPS
```

### 2. Rathole (Similar to Boring)
```bash
# Fast, simple, self-hosted
# https://github.com/rapiz1/rathole
```

### 3. FRP (Fast Reverse Proxy)
```bash
# More complex but very powerful
# https://github.com/fatedier/frp
```

### 4. Just Use Dynamic DNS + Port Forward
- Use DuckDNS or similar for dynamic DNS (free)
- Port forward 8443 -> 8080
- Add nginx with Let's Encrypt for SSL
- Most "honest" approach but exposes your IP

## My Recommendation

For a homelab with collaborators, Cloudflare Tunnel is honestly the best option:

1. **It's actually free** (not "free trial" or "free tier with limits")
2. **No social login BS** - just email/password
3. **Better than exposing your home IP**
4. **Easier than managing VPN users**
5. **Professional URLs** (gamecode.yourdomain.com)

The only real cost is ~$10-15/year for a domain name, which you probably want anyway.

## Setup Script for Cloudflare

Would you like me to create a setup script that:
1. Installs cloudflared
2. Guides you through tunnel creation
3. Sets up the service
4. Configures access control

It's honestly less intrusive than Tailscale's identity requirements, and you get a professional setup for the price of a domain name.