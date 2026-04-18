# Architecture

## Context & Scope

GameCode Web is a single-user, password-gated chat UI for local LLMs. It fronts an HTTP inference backend (today: Ollama) and renders streamed responses in a notebook-style interface.

Inside the boundary: the Axum server (`server/`), the Leptos WASM client (`client/`), and the `dist/` static bundle the server serves. Outside the boundary: Ollama (reached over HTTP at `GAMECODE_OLLAMA_BASE_URL`, typically `http://localhost:11434`), and the deployment environment (Docker image → k8s via Flux GitOps, typically exposed through ngrok).

The server is the only system with outbound calls: the browser talks only to the server at `/api/*`, and the server translates requests into the provider's native protocol.

## Solution Strategy

- **Rust end-to-end.** Workspace with two crates: `gamecode-server` (Axum + tokio) and `gamecode-client` (Leptos CSR compiled to WASM via Trunk).
- **Streaming over SSE.** Provider tokens are relayed to the browser as Server-Sent Events; the client renders progressively.
- **Stateless server, stateful client.** The server holds no conversation state. Conversations, context summaries, and UI prefs live in the browser (IndexedDB + localStorage).
- **Password + JWT.** Single-user auth: argon2 hash verified on login, HS256 JWT bearer on subsequent calls. Password is supplied plaintext via `GAMECODE_AUTH_PASSWORD` and hashed on startup.
- **Config via env.** All runtime config comes from `GAMECODE_*` environment variables (see `server/src/config.rs`). Prompts load from `config/prompts.toml` (or `/usr/local/etc/gamecode-web/prompts.toml`).

## Building Blocks

**`server/` — `gamecode-server` binary**
- `main.rs` — wires `Config`, `ProviderManager`, static `ServeDir` for `dist/`, and `api::routes()` under `/api`.
- `api.rs` — endpoints: `GET /health`, `POST /auth`, `GET /providers`, `GET /prompts`, `POST /chat`. `/chat` returns an SSE stream of `ChatChunk` JSON events.
- `auth.rs` — `AuthUser` extractor validates the bearer JWT; `verify_password` (argon2) and `generate_token` (jsonwebtoken). Claims: `sub`, `exp`, `iat`.
- `providers/` — `InferenceProvider` trait (`name`, `available`, `list_models`, `chat` → `ChatStream`). `ProviderManager` owns a `HashMap<String, Box<dyn InferenceProvider>>`. Only `OllamaProvider` is implemented; it posts to `{base_url}/api/chat` with `stream: true` and parses newline-delimited JSON. A stop-pattern filter cuts the stream on `\nUser:` / `\nHuman:` / `\n---\n`.
- `bin/hash_password.rs` — utility for generating argon2 hashes out-of-band.

**`client/` — `gamecode-client` (WASM)**
- `main.rs` — Leptos `App` with auth gate: `AuthForm` → `Chat` once a valid token exists in localStorage.
- `api.rs` — `ApiClient` wraps `/api/*` calls. Owns request/response types and constructs the chat URL consumed by the SSE reader.
- `components/` — `auth.rs` (login + token storage), `chat.rs` (top-level chat shell, provider/model/prompt selectors, streaming loop), `context_manager.rs` (token-count driven auto-compression at 85 % of `MAX_CONTEXT_TOKENS = 4096`), `resize_handle.rs`.
- `notebook/` — domain model for the scrolling UI: `Notebook { cells, cursor_position, active_input }`, `Cell { id, content, timestamp, metadata }`, and `CellContent` variants `UserInput | TextResponse | Code | Diagram | Image | Table | Chart | Error | Loading`. `DiagramFormat` enumerates Graphviz/PlantUML/Mermaid/D2/Excalidraw. The `Notebook` is the aggregate — mutation goes through `add_cell`, `update_streaming_response`, and `finalize_streaming_response`. `parser.rs` extracts fenced code blocks; `renderer.rs` holds renderer stubs (currently return placeholder SVG).
- `storage.rs` — `ConversationStorage` over IndexedDB (`gamecode_conversations` DB, `conversations` store). `StoredConversation` = `{ id, notebook, context_state, metadata }`. `simple_storage.rs` is a lighter localStorage fallback used alongside.
- `markdown.rs` — pulldown-cmark + syntect for server-free markdown & syntax highlighting inside the WASM bundle.

**Root**
- `build.rs` — invokes `trunk build --release` in `client/` when the root crate is built; the root `src/main.rs` is a vestigial stub.
- `Makefile` — canonical build/run entry points (`build`, `run`, `dev`, `watch`, `docker-build`).
- `Dockerfile` — multi-stage: `rust:1.88` builder installs trunk + wasm target, builds client then server; runtime is `debian:bookworm-slim` running as non-root `gamecode` user on port 8080.
- `.github/workflows/docker-build.yml` — CI image build; deployment is Flux GitOps from another repo.

## Crosscutting Concepts

- **Errors.** Server uses `anyhow` internally and a thin `AppError` enum with an `IntoResponse` impl for HTTP mapping (`Unauthorized`, `BadRequest`, `NotFound`, `Internal`). Client uses `thiserror` (`ApiError`) and propagates auth failures up to the root component, which clears the token and returns to the login form.
- **Streaming contract.** The SSE payload is the provider-agnostic `ChatChunk { text, done }`. Non-text server errors are emitted as a JSON event with an `error` field. The client's SSE reader checks `done` to close the cell's streaming state and trigger post-processing (diagram detection hook).
- **Context budgeting.** Token counts are estimated client-side; the `ContextManager` compresses older turns into summary strings when the running estimate exceeds 85 % of the configured window. Compression state is persisted with the conversation.
- **Configuration.** All server config reads through `Config::load()` at startup; there is no runtime reload. Required vars fail fast (`context` on `env::var`). Optional vars have defaults via `parse_env`. An unset `GAMECODE_AUTH_JWT_SECRET` logs a warning and generates an ephemeral secret — sessions do not survive restarts in that mode.
- **Logging.** `tracing` + `tracing-subscriber` on the server (INFO by default); `tracing-wasm` plus `web_sys::console` on the client.
- **Serialization.** `serde` / `serde_json` everywhere on the wire. TOML only for `prompts.toml`.
