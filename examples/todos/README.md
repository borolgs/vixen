# todos

A small vixen app: a todo list on one page, kept in axum state. Read
`src/pages/todos/mod.rs` top to bottom. `examples/counter` is the step before it.

```bash
bun install           # once, for htmx
cargo run -p todos    # http://127.0.0.1:4001/
```

## What to look for

- **`#[route("/")]`** makes `HomePath` the typed route used by `.view(home)`.
  `router()` lists the page's handlers.
- **`#[action("/todos/add")]`** makes `AddTodo` the POST route and form
  extractor. It follows `State` because it reads the body. `.action(add)` gets
  its route from that last argument. `AddTodo::action()` renders the path and
  method, `AddTodo::FIELD.title` names the form field, and
  `ToggleTodo::action().id(todo.id)` puts `id` in `hx-vals`.
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
`#[route]` and `#[action]` expands to `::axum_extra::..` paths. `maud` is
one because its own `html!` expands to `extern crate maud;`, which no re-export
satisfies.
