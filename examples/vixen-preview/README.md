# vixen-preview

`fetch` → WASM-compiled `axum::Router` → `Response`.

## Run

From the repository root (Rust, Bun, and Python 3 required):

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-pack --version 0.15.1 --locked
bun run --cwd examples/vixen-preview build
bun run --cwd examples/vixen-preview serve
```

Open <http://127.0.0.1:4007/>. Deploy `examples/vixen-preview/dist/` to a static host.

## Test

`bun run --cwd examples/vixen-preview test` (after building WASM).
