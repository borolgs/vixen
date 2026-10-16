# todos

A small vixen app: a todo list on one page, kept in axum state. Read
`src/pages/todos/mod.rs` top to bottom. `examples/counter` is the step before it.

```bash
bun install           # once, for htmx
cargo run -p todos    # http://127.0.0.1:4001/
```

## What to look for

- **`router()`** at the top lists the page's handlers. `#[route("/")]` makes
  `HomePath` the typed route used by `.view(home)`.
- **`ctx()`** hands every handler the todos. `Ctx` in `state.rs` derives
  `ReqCtx`, and `Ctx::middleware` in `main.rs` sets it for each request.
- **`#[action("/todos/add")]`** makes `AddTodo` the POST route and form
  extractor. It comes last because it reads the body. `.action(add)` gets its
  route from that last argument. `AddTodo::action()` renders the path and
  method, `AddTodo::FIELD.title` names the form field, and
  `ToggleTodo::action().id(todo.id)` puts `id` in `hx-vals`.
- **`#[fragment]`** on `todo_list`, `todo_count`, `todo_message` and
  `add_todo_form`: the same function renders the element in the page and
  replaces it in a response.
- **`#[id]`** on `TodoItemId(usize)`: a dynamic id, `todo-item-3`.
  `#[fragment(TodoItemId(todo.id))]` gives it to both `todo_item` and
  `todo_item_edit_form`, so toggle, edit, save and cancel each replace just
  that row.
- **`partial!`** in the actions lists fragments, each filled by the same
  function the page used. `Part::before_end` appends a new row and
  `Part::delete` drops a toggled or renamed one, each only when the search says
  so.
- **`todo_message`** sits next to the count. The page renders
  `todo_message::slot()`; `add` and `save` fill it with a `Message::Info` that
  empties itself after 3s via `hx-on="load delay:3s -> …"`. An empty title or a
  stale id gets a `Message::Error` instead of a status code, and it stays until
  the next message.
- **The search lives in the URL.** `search_form` is one `hx-action` for the
  text and the All / Active / Done radios, sent on input. `replace_query`
  writes `?q=milk&filter=Active`. `home` reads it back with `Query`, `add` and
  `toggle` with `HxCurrentQuery`. `SearchTodos::matches` is the one rule they
  share.
- **`assets!()`** in the head, with `index.ts` next to `mod.rs`: that is the
  whole setup. `build.rs` finds `src/pages/**/index.ts` by itself, there is no
  `build.ts`. The script imports htmx from npm and a plain `index.css`, nothing
  else. `assets_router!()` in `main.rs` serves the bundle from the binary.

`axum-extra` is a direct dependency because the `TypedPath` derive behind
`#[route]` and `#[action]` expands to `::axum_extra::..` paths, and its `query`
feature gives `Query`. `maud` is one because its own `html!` expands to
`extern crate maud;`, which no re-export satisfies.
