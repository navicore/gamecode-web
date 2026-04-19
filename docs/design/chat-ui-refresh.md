# Chat UI Refresh

Source handoff: `docs/design/inbound/gamecode-web/` (Claude Design bundle, React/HTML/CSS prototype).

## Intent

Replace the current chat shell with the visual language in the handoff: two-column layout, grouped sidebar conversation list, model + persona pill popovers in the header, avatar-rail message list, auto-grow composer with attachments/temperature, empty-state starters, and a light/dark theme driven by CSS custom-property tokens. We are **recreating the look** (per the bundle README) — not porting React. Leptos components map onto the prototype's component tree; the CSS tokens port wholesale.

## Constraints

- Stack: Leptos 0.6 CSR + Trunk. No JS framework, no Tailwind JIT, no CSS-in-JS. Static CSS under `client/index.html` / `client/assets/`.
- SSE contract (`/api/chat` → `ChatChunk { text, done }`) is unchanged. Do not adopt the prototype's structured-block streaming — keep the existing text-append + `markdown.rs` re-render path.
- Backend is Ollama-only today. UI must source provider/model lists from `/api/providers`; do not hardcode DeepSeek/OpenRouter.
- Notebook aggregate (`Notebook` / `Cell` / `CellContent`) stays authoritative. The redesign is a presentation-layer swap.
- Conversations already fully load from IndexedDB — no pagination / infinite-scroll work.
- `just ci` must stay green; zero new `#[allow]`.

## Callouts on the design (what not to copy wholesale)

- **Attachments + voice (`Composer.jsx`)** — backend has no upload channel and no STT. Ship the visual-only affordances later; leave them out of this pass or hide behind a feature cfg.
- **Multi-provider chrome (`data.js`)** — DeepSeek / OpenRouter / cloud badges are aspirational per ROADMAP. Render whatever `/api/providers` returns; group by real provider only.
- **Infinite scroll + day dividers (`App.jsx` `loadOlder`)** — unnecessary; we always have the full notebook in memory. Skip.
- **Tweaks panel + `__activate_edit_mode` postMessage protocol** — Claude Design tooling artifact. Drop.
- **Footer "streamed through `/v1/chat`"** — wrong path; ours is `/api/chat`. Fix copy.
- **"Persona" ≈ our system prompt** — the coloured "persona" concept is a nicer surface for `SystemPrompt` from `prompts.toml` + custom prompt. Adopt the naming and the coloured rail/avatar, keep the existing prompt source.
- **Google Fonts CDN (Inter + JetBrains Mono)** — local-first app making outbound requests to `fonts.gstatic.com` is a privacy regression. Self-host woff2 under `client/assets/fonts/` or keep system stack; don't add the `<link>` tags.
- **Header Share/More buttons** — no share target for a single-user local app. Drop or repurpose (e.g. Export).
- **`dangerouslySetInnerHTML` for code highlighting** — React-ism. Use Leptos `inner_html=` into a `<code>`; syntect already produces the HTML.

## Approach

1. **Tokens first.** Port `:root` + `[data-theme="dark"]` custom properties from `styles.css` into a new `client/assets/styles/tokens.css`. Leave the OKLCH values intact. Set `data-theme` on `<html>` from a `theme` signal persisted in localStorage.
2. **Layout skeleton.** Replace the current chat shell with `app` grid (sidebar default 280px, user-resizable via existing `ResizeHandle`, width persisted to localStorage; main 1fr). Sensible clamp: min ~220px, max ~480px. New Leptos components: `Sidebar`, `ChatHeader`, `Thread`, `Composer`, `EmptyState`.
3. **Sidebar.** Brand, `New chat` button, search input filtering `ConversationMetadata`, grouped list derived from `updated_at` (Today / Yesterday / Earlier), user chip with Ollama connection dot driven by `/api/providers` availability.
4. **Header pills.** `ModelPicker` and `PersonaPicker` as popovers (click-outside to close; focus trap optional). ModelPicker groups by provider using the `/api/providers` response. PersonaPicker replaces the current prompt dropdown; colour comes from a small persona→colour map keyed by prompt name.
5. **Message list.** `Message` component with avatar rail, persona-coloured line, header (author · model tag · timestamp), markdown body via existing `markdown.rs`, streaming-cursor span appended to the last block while `done=false`, action row (Copy / Regenerate / …) on finalized assistant cells only.
6. **Code blocks.** Extend the markdown renderer to emit a `<div class="code-block">` with a copy button around syntect output. New: clipboard write + 1.4s "Copied" state.
7. **Composer.** Auto-grow `<textarea>` (cap 220px), temperature slider inline, send/stop toggle. Drag-and-drop + paperclip + mic are visual-only stubs behind `#[cfg(feature = "attachments")]` or simply omitted.
8. **Empty state.** Four starter chips that populate the composer on click. Greeting uses JWT `sub` or a placeholder — not "Dani".
9. **Dark mode.** Flip `data-theme`; tokens do the rest. Persist choice.

## Domain Events

- **Conversation selected** → load `StoredConversation` from IndexedDB → replace `Notebook` → scroll to bottom.
- **User sends text** → append `UserInput` cell → open SSE → append streaming `TextResponse` cell, setting `streaming=true` on the last cell → on `done=true`, finalize cell, run `ContextManager` check, persist conversation.
- **Theme toggled** → update `theme` signal → `document.documentElement.dataset.theme = ...` → write to localStorage.
- **Persona selected** → update `selected_prompt_name` → persist to localStorage → surface colour on subsequent assistant cells.
- **Temperature changed** → update signal → persist to localStorage.
- **New chat** → mint new conversation id → clear notebook → persist empty conversation.

## Checkpoints

1. Side-by-side with `Gamecode Chat.html` in light theme: spacing, radii, shadow depth, font rendering match within ~2px.
2. `data-theme="dark"` toggles cleanly; no FOUC on reload (apply from localStorage before Leptos hydrates — set on `<html>` in `index.html` via inline script, or accept a flash).
3. Sidebar search filters the real conversation list; empty result shows nothing (not a placeholder).
4. Streaming cursor visible and blinking only on the last message while `done=false`, gone the instant `done=true`.
5. Code fences render highlighted *and* Copy button copies + briefly shows "Copied".
6. Temperature + persona + theme survive reload.
7. `/api/providers` controls the ModelPicker — Ollama only, no phantom providers.
8. `just ci` green.

## Out of scope (this pass)

File attachments, voice input, export/share, infinite scroll, multi-provider UI beyond what the backend actually exposes, Tweaks edit-mode protocol, Google Fonts CDN, prompt editor ("Edit personas" link).
