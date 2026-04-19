# GameCode Web — build recipes.
#
# This justfile is the single source of truth for build/test/lint.
# Both local dev and GitHub Actions call `just ci` — no drift.
#
# The workspace has two crates:
#   - server/: native binary (axum)
#   - client/: wasm32-unknown-unknown bundle (leptos, built via trunk)
# Native cargo commands target server only; client goes through trunk.

# Default recipe: show available recipes
default:
    @just --list

# Format all crates
fmt:
    cargo fmt --all

# Check formatting (fails if anything is unformatted)
fmt-check:
    cargo fmt --all -- --check

# Clippy on the server crate. Client is WASM and is linted via `cargo check`
# against wasm32-unknown-unknown inside `build-client`.
lint:
    cargo clippy --locked -p gamecode-server --all-targets -- -D warnings
    cargo clippy --locked -p gamecode-client --target wasm32-unknown-unknown --all-targets -- -D warnings

# Tests (server only — client code runs in the browser)
test:
    cargo test --locked -p gamecode-server --all-targets

# Release builds: server via cargo, client via trunk
build: build-server build-client

build-server:
    cargo build --locked --release -p gamecode-server

# `cargo check --locked` first enforces Cargo.lock integrity for the client's
# wasm deps (trunk doesn't forward --locked). Then trunk does the full wasm
# build + bindgen + static bundle into dist/.
build-client:
    cargo check --locked --release --target wasm32-unknown-unknown -p gamecode-client
    cd client && trunk build --release

# Run all CI checks (same as GitHub Actions).
# This is what developers should run before pushing.
ci: fmt-check lint test build
    @echo "Safe to push to GitHub - CI will pass."

# Remove build artifacts
clean:
    cargo clean
    rm -rf dist
