# counter

vixen on one screen: a counter in a `static`, all of the Rust in `src/main.rs`.
`examples/todos` is the next step.

```bash
bun install            # once, for htmx
cargo run -p counter   # http://127.0.0.1:4002/
```

## What to look for

- **`#[action("/add")]`** on `Add`: one struct is the route, the form extractor
  (`async fn add(Add { by }: Add)`) and what the buttons render.
  `hx-action=(Add::action().by(-1))` and `.by(1)` are the same endpoint with a
  different `hx-vals`.
- **`#[id]`** on `CountId`: `id=(CountId)` in the page, `#count` in the
  response.
- **`#[fragment]`** on `heading`: the fn owns the `<h1>` and its id, so the page
  and the response both just call `heading(count)`.
- **`partial!`** in `add`: one click, two swaps. `CountId =>` aims the new
  number at the `<output>`; `heading(count)` swaps the whole `<h1>`. There is
  no main swap, so the button that asked keeps its label.
- **`assets!()`** in the head, with `index.ts` and `index.css` next to
  `main.rs`. `assets!()` looks for an entry in the caller's own directory, and
  the default glob only covers `src/pages/**`, so `build.rs` names
  `src/index.ts` itself. `assets_router!()` serves the bundle from the binary.

`axum-extra` is a direct dependency because the `TypedPath` derive behind
`#[view_path]` and `#[action]` expands to `::axum_extra::..` paths. `maud` is
one because its own `html!` expands to `extern crate maud;`, which no re-export
satisfies.
