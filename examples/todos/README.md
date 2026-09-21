# todos

A small vixen app: a todo list on one page, kept in a `static`. Read `src/pages/todos/mod.rs` top to
bottom. `examples/counter` is the step before it.

```bash
bun install           # once, for htmx
cargo run -p todos    # http://127.0.0.1:4001/
```

## What to look for

- **`#[view_path("/")]`** on `HomePath`: the route is a type, and
  `.typed_get(home)` reads it off the handler's argument. `router()` is the
  table of contents.
- **`#[action("/todos/add")]`** on `AddTodo`: one struct is the route, the form
  extractor (`async fn add(AddTodo { title }: AddTodo)`) and the view's
  spelling of both. `hx-action=(AddTodo::action())` renders the path and the
  method, `name=(AddTodo::FIELD.title)` is the field's wire name, and
  `ToggleTodo::action().id(todo.id)` puts a value in `hx-vals`.
- **`#[id]`** on `TodoListId` and `TodoCountId`: the same type is
  `id=(TodoCountId)` in the page and `#todo-count` wherever a response aims at
  it.
- **`partial!`** in `add`: `_ =>` is the main swap (the form re-renders
  empty), and `TodoListId =>` / `TodoCountId =>` are aimed by the handler, each
  filled by the same function the page used. `toggle` answers with parts only,
  so the checkbox that asked is left alone.
- **`assets!()`** in the head, with `index.ts` next to `mod.rs`: that is the
  whole setup. `build.rs` finds `src/pages/**/index.ts` by itself, there is no
  `build.ts`. The script imports htmx from npm and a plain `index.css`, and
  runs the All / Active / Done filter, which is view state the server never
  hears about. `assets_router!()` in `main.rs` serves the bundle from the
  binary.

`axum-extra` is a direct dependency because the `TypedPath` derive behind
`#[view_path]` and `#[action]` expands to `::axum_extra::..` paths. `maud` is
one because its own `html!` expands to `extern crate maud;`, which no re-export
satisfies.
