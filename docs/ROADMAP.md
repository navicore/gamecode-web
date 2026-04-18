# Roadmap

## Current State

- Single-provider chat works end-to-end: Ollama → Axum/SSE → Leptos/WASM.
- Password + JWT auth, argon2 hashing on startup from `GAMECODE_AUTH_PASSWORD`.
- Conversation persistence in IndexedDB with client-side context auto-compression at 85 % of a 4096-token budget.
- Markdown and syntax-highlighted code cells render in the WASM bundle (pulldown-cmark + syntect).
- Docker multi-stage image builds in CI; deployment via Flux GitOps to k8s.

## Known Gaps / Next Steps

- **Diagram rendering is stubbed.** `client/src/notebook/renderer.rs` returns placeholder SVGs for Mermaid/Graphviz; real WASM renderers (graphviz-wasm, mermaid, PlantUML, D2) are not wired in. `Cell::detect_and_render_diagrams` in `notebook/mod.rs` has an empty body — finalizing a streamed response does not yet turn fenced diagram blocks into `CellContent::Diagram`.
- **Only the Ollama provider exists.** The `InferenceProvider` trait is shaped for Bedrock / OpenAI / MCP additions (per README), but no other impls are present.
- **Two storage layers coexist.** `storage.rs` (IndexedDB) and `simple_storage.rs` (localStorage) are both in use; worth consolidating once the IndexedDB path is fully trusted.
- **Context-token estimation is heuristic.** `MAX_CONTEXT_TOKENS` is hard-coded to 4096 and does not vary by model.
- **JWT secret is ephemeral by default.** If `GAMECODE_AUTH_JWT_SECRET` is unset, the server logs a warning and generates a random secret; all sessions invalidate on restart.
- **Root `src/main.rs` is vestigial.** It still contains the default `println!("Hello, world!")` stub; the real binaries live in `server/` and `client/`.
- **Stop-pattern filter is provider-specific.** Ollama streaming cuts on `\nUser:` / `\nHuman:` / `\n---\n`. Needs revisiting when additional providers land.
