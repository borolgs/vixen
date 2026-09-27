# counter

Minimal vixen app: one counter, one Rust file.

```bash
bun install            # once, for htmx
cargo run -p counter   # http://127.0.0.1:4002/
```

## What it shows

- `#[route("/")]` and `.view(home)` define the page route.
- `#[action("/add")]` makes `Add` the POST route and form extractor;
  `.action(add)` registers it. `Add::action().by(...)` sends `by` through
  `hx-vals`.
- `#[fragment]` makes `heading` and `counter` independently replaceable.
  `partial!` updates both after each click.
- `assets!()` loads `src/index.ts` and its CSS; `assets_router!()` serves the
  bundle. `build.rs` points the bundler at this non-default entry path.
