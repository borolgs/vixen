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

## Examples and state

The `hello` tab keeps its form, two source snippets, and result below them.
The `todos` tab puts the UI beside its Rust source, or above it on narrow
screens. Source text scales to fit the code panel. Switching tabs keeps the
existing DOM, including unfinished edits.

Each mounted preview owns a WASM `PreviewApp` with one `axum::Router`.
Requests clone that router, sharing its `axum::State` rather than rebuilding
the application. Todos live in an `Arc<Mutex<TodoState>>`; handlers hold the
lock only while updating state and rendering, never across an `await`.
New previews start with the standalone example's sample todos. Reloading the
page restores those samples. Nothing uses localStorage.

The todo actions follow [the standalone example](../todos/src/pages/todos/mod.rs).
Here, search and filter live in the preview state instead of `HxCurrentQuery`,
so they don't change the host page's URL. Todos has no request-step animations;
hello keeps its original animation.

## Test

`bun run --cwd examples/vixen-preview test` (after building WASM).
`cargo test -p vixen-preview` checks the source-panel helpers.
