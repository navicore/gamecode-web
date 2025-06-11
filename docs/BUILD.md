# Building and Running GameCode Web

## Prerequisites

1. **Rust** (latest stable)
2. **Trunk** for WASM builds:
   ```bash
   cargo install trunk
   ```
3. **Ollama** running with your model:
   ```bash
   ollama run fortean
   ```

## Development

### 1. Start the Backend Server

```bash
cd server
cargo run
```

The server will start on `http://localhost:8080`

### 2. Build and Serve the Frontend

In a new terminal:

```bash
cd client
trunk serve --open
```

This will:
- Build the WASM app
- Serve it on `http://localhost:8081`
- Proxy API calls to the backend
- Auto-reload on changes

### 3. Access the App

1. Open `http://localhost:8081`
2. Enter password: `gamecode` (change in production!)
3. Start chatting with your AI

## Production Build

### Build Frontend
```bash
cd client
trunk build --release
```

### Run Production Server
```bash
cd server
cargo run --release
```

### Deploy with ngrok
```bash
ngrok http 8080 --basic-auth="user:password"
```

## Testing the Notebook Features

Try these prompts to test diagram rendering:

1. **Graphviz diagram**:
   ```
   Show me a simple architecture diagram using graphviz
   ```

2. **Mermaid flowchart**:
   ```
   Create a flowchart showing user authentication flow
   ```

3. **PlantUML sequence**:
   ```
   Draw a sequence diagram for a web request
   ```

## Configuration

Edit `config/default.toml` to:
- Change password hash
- Configure Ollama models
- Add new providers

## Architecture

```
Browser (WASM) <-> Rust Server <-> Ollama
                              \-> Bedrock (future)
                              \-> OpenAI (future)
```

The notebook interface will:
- Render text with markdown
- Auto-detect and render diagrams
- Support rich media (images, tables)
- Stream responses in real-time