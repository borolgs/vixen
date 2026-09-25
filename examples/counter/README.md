# counter

Minimal vixen app: one counter, one Rust file.

```bash
bun install            # once, for htmx
cargo run -p counter   # http://127.0.0.1:4002/
```

## What it shows

- `#[view_path("/")]` and `typed_get` define the page route.
- `#[action("/add")]` defines the POST route and extracts `by` from the form.
  `Add::action().by(...)` sends it through `hx-vals`.
- `#[fragment]` makes `heading` and `counter` independently replaceable.
  `partial!` updates both after each click.
- `assets!()` loads `src/index.ts` and its CSS; `assets_router!()` serves the
  bundle. `build.rs` points the bundler at this non-default entry path.
