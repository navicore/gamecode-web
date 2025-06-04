# GameCode Web

A secure web chat interface for AI models, starting with local Ollama integration.

## Architecture

- **Frontend**: Rust/Leptos WebAssembly app
- **Backend**: Rust/Axum server with provider abstraction
- **Security**: Password protection with JWT sessions
- **Deployment**: ngrok tunnel for secure external access

## Quick Start

### 1. Start Ollama with your model
```bash
ollama run fortean
```

### 2. Build and run the server
```bash
cd server
cargo run --release
```

### 3. Access via ngrok
```bash
ngrok http 8080
```

## Configuration

Edit `config/default.toml`:

- Change the password hash (default password is "gamecode")
- Update JWT secret
- Configure Ollama models

## Development

### Server
```bash
cd server
cargo watch -x run
```

### Client (coming next)
```bash
cd client
trunk serve
```

## Security Notes

1. **Change the default password** in production
2. Use a strong JWT secret
3. Enable ngrok authentication for additional security
4. Monitor ngrok dashboard for abuse

## Future Providers

The architecture supports adding:
- AWS Bedrock
- OpenAI API
- Local models via candle
- MCP tool integration