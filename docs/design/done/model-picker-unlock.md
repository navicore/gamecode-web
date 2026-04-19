# Unlock the model picker; hydrate it per-conversation

## Intent

The mid-chat model lock is arbitrary. The `/api/chat` contract already accepts `model` per-request (`server/src/providers/mod.rs:19 ChatRequest`), Ollama is stateless across calls, and the client re-POSTs full history + chosen model on every turn. Nothing is tied to a specific model after the first reply. Drop the lock so a user who isn't satisfied with one model can consult a better one on the same conversation.

Same root cause drives the second bug: switching to an existing chat doesn't restore that chat's model. The picker keeps showing whatever was last globally selected, because `on_select` (`client/src/components/chat.rs:440`) loads `notebook` + `context_state` + `created_at` from storage but ignores `stored.metadata.{provider,model}`.

## Constraints

- `/api/chat` contract unchanged. `ChatRequest.model` stays `Option<String>`; server passes through.
- `StoredConversation.metadata.{model, provider}` already exists and is already written on every save (`chat.rs:321`). No schema migration.
- Context token estimation stays heuristic. Different model families have different tokenizers; `MAX_CONTEXT_TOKENS = 4096` is already model-agnostic per `ROADMAP.md`. Mid-chat switches make that fuzzier, not newly broken.
- Cross-provider switches within one chat are in — `selected_provider` and `selected_model` already move together via `ModelPicker`; no separate work needed.
- Old stored conversations may have empty `metadata.model`/`provider`. Fall through to current picker state silently.
- `just ci` green, no new `#[allow]`.
- Out of scope: per-model tokenizers, rendering a per-cell model badge in the message header (the `chat-ui-refresh.md` design anticipates "author · model · timestamp" — call that a follow-up once this lands), offering to "ask a different model to retry the last turn" as a first-class action.

## Approach

Three small edits in `client/src/components/chat.rs`:

1. **Remove the lock.** Delete `disabled=has_messages.into()` from the `ModelPicker` at `chat.rs:525`. Drop the `has_messages` memo if no other consumer remains, or leave it for the streaming disable path if there is one.
2. **Hydrate on chat switch.** In `on_select` (`chat.rs:440`), after `set_notebook.update(|nb| *nb = stored.notebook)`, also:
   - `if !stored.metadata.provider.is_empty() { selected_provider.set(stored.metadata.provider.clone()); }`
   - `if !stored.metadata.model.is_empty() { selected_model.set(stored.metadata.model.clone()); }`
   - Do the same in the initial `create_effect` at `chat.rs:169` that loads the conversation on mount (the two code paths should behave identically — consider extracting a small `apply_stored` helper).
3. **Leave "new chat" behavior alone.** On `on_new_chat`, the picker keeps its current value (sourced from the `selected_model` / `selected_provider` localStorage keys written on change). That's already the "last model the user picked" default the user asked for.

The picker's `disabled` prop can stay — flip it for the duration of an in-flight stream (`is_streaming`) if we want to avoid switching models mid-token. That's a small quality-of-life fix, not a requirement.

## Domain Events

- **Model changed mid-chat** — `selected_model` signal updates → next submit sends the new model in `ChatRequest.model` → on save, `StoredConversation.metadata.model` reflects the newest choice → on reload, picker hydrates to that.
- **Conversation selected** — load stored notebook + context + created_at → *also* hydrate `selected_provider` / `selected_model` from `stored.metadata` → picker snaps to that chat's last-used model.
- **New chat created** — no metadata to hydrate from; picker keeps its current (last-used) selection.
- **Provider disappears from `/api/providers`** — existing effect at `chat.rs:262` already reconciles model-not-in-list on provider change. Same path covers hydration pointing at a model the server no longer offers; acceptable fallback.

## Checkpoints

1. Open an existing chat that has assistant cells → picker displays that chat's last-used model, not whatever was selected globally before the click.
2. Start a two-turn chat with model A. Switch picker to model B. Send a third turn. Network tab shows `model: "B"` on that request. Reopen the chat → picker shows model B.
3. Two separate chats, each with a different model. Switching between them in the sidebar moves the picker to the chat's own model each time.
4. Brand-new chat still inherits the globally-last-used model (no regression on the new-chat path).
5. Conversation with no stored model (legacy entry) loads without throwing; picker stays on its current selection.
6. Server logs show the right model name per turn when the user toggles mid-chat.
7. `just ci` green.
