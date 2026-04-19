# SSO via anz

Replace the argon2 shared-secret login with OIDC against anz (`https://auth.navicore.tech`). Proper BFF shape: server is a confidential OIDC client, browser is driven by an HttpOnly session cookie, no tokens ever reach JavaScript.

## Intent

- One named user per browser session. The lower-right user chip surfaces the real `preferred_username` — the `"user"` literal that `chat.rs:20 user_name_from_token` currently extracts from `sub` is gone.
- Reuse the operator's existing IdP (anz). Match the client-registration pattern already established by Headlamp in `k8s-vcluster-homelab/secrets.manifest.yaml:77` — confidential client, PKCE, client_secret rotated in anz and synced into k8s by hand (the known drift pattern).
- No hacks: no self-minted JWT layer on top of anz, no bearer tokens in `localStorage`, no tokens in URLs.

## Constraints

- **Stateless server.** Session state rides in a signed + encrypted cookie, not server memory. ARCHITECTURE.md's "Stateless server, stateful client" stays true.
- **SSE on `/api/chat` keeps working.** Current client streams via `fetch` (not `EventSource`); same-origin cookies attach automatically, so this is a drop-in swap.
- **No backward compatibility.** Existing password logins and existing `auth_token` localStorage entries are discarded. `POST /api/auth`, `verify_password`, `GAMECODE_AUTH_PASSWORD`, `bin/hash_password.rs`, and the `argon2` dep are deleted outright.
- **Fail-fast config.** All OIDC env vars and `GAMECODE_AUTH_SESSION_KEY` are required; missing any → server refuses to start. No ephemeral-fallback mode.
- **Single-origin.** Browser, gamecode-web, and the OIDC callback are all on `https://gamecode.navicore.tech`. SameSite=Lax + Secure + HttpOnly is sufficient; no CORS, no third-party cookie concerns.
- `just ci` green, no new `#[allow]`.
- Out of scope: RP-initiated logout against anz, group-based authorization inside gamecode, multi-realm support, in-app "switch user" UX.

## Approach

### Flow

1. Unauthenticated request to `/` → static bundle loads → WASM calls `GET /api/me` → 401 → client does `window.location = "/api/auth/login"`.
2. `GET /api/auth/login` — mint PKCE `code_verifier` + CSRF `state`, stash both in a short-lived (5 min) encrypted cookie (`__Host-gc_oidc_tx`), 302 to `{issuer}/authorize?response_type=code&client_id=…&redirect_uri=…&scope=openid+profile&state=…&code_challenge=…&code_challenge_method=S256`.
3. `GET /api/auth/callback?code=&state=` — consume the tx cookie, verify `state` constant-time, POST to `{issuer}/token` (`grant_type=authorization_code`, `code_verifier`, client_secret via Basic auth), verify the ID token against `{issuer}/jwks` (cached JWKS, refresh on unknown `kid`), assert `iss` / `aud` / `exp` / `nonce`. Store `{ access_token, refresh_token, access_exp, username, sub }` in the session cookie (`__Host-gc_session`, HttpOnly, Secure, SameSite=Lax, TTL matching refresh-token lifetime). 302 to `/`.
4. Every `/api/*` call — `AuthUser` extractor decrypts the session cookie. If `access_exp` is in the future, validate the access-token signature + `exp` against JWKS and proceed. If expired, use the refresh token against `{issuer}/token` (`grant_type=refresh_token`), re-encrypt cookie with the rotated tokens, emit a new `Set-Cookie` on the response. If refresh fails → 401.
5. `GET /api/me` — returns `{ "username": "<preferred_username>" }` from the decrypted session. Drives the sidebar chip.
6. `POST /api/auth/logout` — clears the session cookie (`Set-Cookie: …; Max-Age=0`). Local-only; anz-side session untouched.

### Why validate anz's token instead of minting our own

Duplicating a JWT signing layer on top of anz buys nothing — gamecode-web becomes both resource server and auth server, owning two secrets (`JWT_SECRET` + `CLIENT_SECRET`) when it only needs one role. Validating anz's access token against JWKS on every request is the standard resource-server pattern, keeps session lifetime under anz's control, and lets `anz session revoke` actually end a session.

### Cookie crypto

AES-256-GCM with a 32-byte key from `GAMECODE_AUTH_SESSION_KEY` (base64). Binding: authenticated associated data = cookie name. No JWT, no HMAC-only; AEAD so a reader can't even learn the username. Library: `cookie` crate for serialization, `aes-gcm` for the AEAD; no new framework.

### Code shape

**Server (`server/src/`)**
- New `auth/` module (replaces the file): `oidc.rs` (discovery, JWKS cache, token exchange, refresh), `session.rs` (cookie encrypt/decrypt + the tx cookie), `extractor.rs` (`AuthUser { username, sub }` from the session cookie). Keep the module under ~200 lines per file.
- `api.rs` routes: drop `POST /auth`; add `GET /auth/login`, `GET /auth/callback`, `POST /auth/logout`, `GET /me`. Existing `/providers`, `/prompts`, `/chat` extractors become `AuthUser` (same name, new internals).
- `config.rs`: drop `AuthConfig.password_hash`, `AuthConfig.jwt_secret`, `AuthConfig.session_duration_hours`. Add `OidcConfig { issuer, client_id, client_secret, redirect_uri, scopes }` and `session_key: [u8; 32]`. All required; `Config::load` returns an error on any missing var.
- Deps: `+reqwest` (token + JWKS), `+aes-gcm`, `+cookie`; `-argon2`, `-rand::distr::Alphanumeric` (no more random-secret fallback). `jsonwebtoken` stays — now verifying RS256 against JWKS instead of HS256 against a shared key.

**Client (`client/src/`)**
- Delete `components/auth.rs` in its current shape; replace with a tiny `LoginRedirect` that just `window.location`s to `/api/auth/login`. No form, no password, no localStorage.
- Delete the `auth_token` localStorage code in `simple_storage.rs` / elsewhere. Delete `user_name_from_token` in `chat.rs` — source of truth is now `GET /api/me`.
- `main.rs` gate: on mount, fire `GET /api/me`. On success → render `Chat` with `username` signal. On 401 → render `LoginRedirect` (which immediately redirects).
- `api.rs` `ApiClient`: drop the `Authorization` header wiring; cookies ride automatically on same-origin requests.
- Sidebar user chip reads the `username` signal directly; `EmptyState` greeting likewise.

### Deploy-time (`k8s-vcluster-homelab`)

1. **Register the client in anz** (realm: `homelab`):
   ```
   anz client add --realm homelab --client-id gamecode-web \
     --redirect-uri https://gamecode.navicore.tech/api/auth/callback --secret
   ```
   Capture the printed secret.
2. **Rewrite `secrets.manifest.yaml`** — replace the `gamecode-web-secret` entry with an OIDC-shaped secret modeled on `headlamp-oidc` (`secrets.manifest.yaml:77`). Keys: `OIDC_CLIENT_ID` (suggested: `gamecode-web`), `OIDC_CLIENT_SECRET`, `OIDC_ISSUER_URL` (`https://auth.navicore.tech/realms/homelab`), `OIDC_REDIRECT_URI` (`https://gamecode.navicore.tech/api/auth/callback`), `OIDC_SCOPES` (`openid profile`), `SESSION_KEY` (32 bytes base64, `generate: "openssl rand -base64 32"`).
3. **`apps/gamecode-web/deployment.yaml`** — remove `GAMECODE_AUTH_PASSWORD`; add the six env vars above as `secretKeyRef`s against the renamed secret.

## Domain Events

- **Login initiated** — `/api/auth/login` → tx cookie set → 302 to anz `/authorize`. No gamecode-side state.
- **Callback received** — code + state arrive → tx cookie consumed → token exchange → ID token verified → session cookie issued → 302 to `/`.
- **Authenticated API call** — `AuthUser` decrypts cookie, validates access token against cached JWKS, proceeds.
- **Access token expired** — refresh-token grant against anz → rotated tokens re-sealed into the session cookie → `Set-Cookie` on the current response. Transparent to the client.
- **Refresh failed** — session cookie cleared, response 401 → client redirects to `/api/auth/login`.
- **Logout** — `POST /api/auth/logout` → session cookie cleared → client redirects to login.
- **`anz client_secret` rotation** (operator action) — rotate in anz, update k8s secret, rollout gamecode-web. SHA256-compare before blaming the app (per the known drift pattern).
- **`anz session revoke`** — next `/api/*` call's refresh attempt fails → 401 → re-login. Works because we no longer paper over anz with our own JWT.

## Checkpoints

1. Unauthenticated hit on `https://gamecode.navicore.tech/` → arrives at the anz login page with `code_challenge`, `state`, `redirect_uri`, and `scope=openid profile` present on the query string.
2. After anz login, sidebar user chip + empty-state greeting display the real `preferred_username`. No `"user"` or `"friend"` fallback visible.
3. `curl https://gamecode.navicore.tech/api/providers` with no cookie → 401. With a valid session cookie → 200.
4. Leave the tab idle past 1 hour (anz access-token lifetime). Next interaction succeeds without a visible redirect — `Set-Cookie` with the rotated tokens lands in DevTools.
5. `anz session revoke --realm homelab --username <u>` → next request returns 401 → page redirects to login.
6. SSE stream on `/api/chat` still streams end-to-end through the cookie-auth path.
7. Restart gamecode-web pod with the same `SESSION_KEY` → browser session survives. Rotate `SESSION_KEY` → all sessions invalidate, as expected.
8. Server refuses to start if any of `GAMECODE_AUTH_OIDC_*` or `GAMECODE_AUTH_SESSION_KEY` is missing.
9. DevTools → Application → Local Storage for the site has no `auth_token` key; Cookies tab shows only `__Host-gc_session` (HttpOnly, Secure, SameSite=Lax).
10. `docs/ARCHITECTURE.md` updated: drop "single-user, password-gated" / "Password + JWT" language, describe the OIDC + cookie-session shape. `ROADMAP.md` item about the ephemeral JWT secret is removed.
11. `just ci` green; no `#[allow]` added.
