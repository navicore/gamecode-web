# GameCode Web

A secure web chat interface for AI models, starting with local Ollama integration.

## Architecture

- **Frontend**: Rust/Leptos WebAssembly app
- **Backend**: Rust/Axum server with provider abstraction
- **Security**: Password protection with JWT sessions

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

### CI parity

The `justfile` is the single source of truth for build/test/lint; GitHub
Actions calls the same recipes. Before pushing, run:

```bash
just ci
```

`just ci` runs, in order: `fmt-check` (rustfmt), `lint` (clippy with
`-D warnings` on server + wasm client), `test` (server tests), and `build`
(server via cargo, client via trunk). The Rust toolchain is pinned to
`1.93.0` in `rust-toolchain.toml` and in every `.github/workflows/ci-*.yml` —
both must agree.

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
