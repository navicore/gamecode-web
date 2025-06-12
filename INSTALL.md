# Installation Instructions

## Quick Install

Just run:
```bash
make install
```

This will:
1. Build the project
2. Install all files (requires sudo password)
3. Set up the service
4. Start it automatically

## What Gets Installed

- **Binary**: `/usr/local/bin/gamecode-web`
- **Web files**: `/usr/local/share/gamecode-web/`
- **Config**: `/usr/local/etc/gamecode-web/config.toml`
- **Prompts**: `/usr/local/etc/gamecode-web/prompts.toml`
- **Logs**: `/usr/local/var/log/gamecode-web/`
- **Service**: `~/Library/LaunchAgents/com.gamecode.web.plist`

## Managing the Service

```bash
# Install/update everything
make install

# Quick restart after config changes
make restart

# View logs
make logs

# Check status
make service-status
```

## Editing AI Personas

The prompts are now in a config file:
```bash
sudo vim /usr/local/etc/gamecode-web/prompts.toml
make restart  # Restart to apply changes
```

## Troubleshooting

If the prompts dropdown is empty:
1. Check that prompts.toml was installed: `ls -la /usr/local/etc/gamecode-web/`
2. Check logs for errors: `make logs`
3. Make sure the service restarted: `make restart`

## Manual Installation

If `make install` fails, you can install manually:

```bash
# 1. Build
make build

# 2. Install files
sudo cp target/release/gamecode-server /usr/local/bin/gamecode-web
sudo mkdir -p /usr/local/share/gamecode-web
sudo cp -r dist/* /usr/local/share/gamecode-web/
sudo mkdir -p /usr/local/etc/gamecode-web
sudo cp config/default.toml /usr/local/etc/gamecode-web/config.toml
sudo cp config/prompts.toml /usr/local/etc/gamecode-web/prompts.toml
sudo sed -i '' 's|static_dir = "dist"|static_dir = "/usr/local/share/gamecode-web"|' /usr/local/etc/gamecode-web/config.toml

# 3. Create service
./scripts/create-service-plist.sh
launchctl load ~/Library/LaunchAgents/com.gamecode.web.plist
launchctl start com.gamecode.web
```