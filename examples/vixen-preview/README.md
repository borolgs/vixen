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
Search and filter live in the preview state rather than the host page's URL.
Todos shows the full Rust source, including imports. Hello keeps its original
animation.

## Render trace

WASM produces a response first. The trace then pairs its htmx fragments with
source ranges: scroll to the handler, draw a code mark, and reveal the matching UI
180 ms later. The next mark follows after another 120 ms by default. Earlier
marks stay visible. After a 1400 ms reading beat, all marks soften over 400 ms;
hover a mark on either side to brighten its counterpart.

Add Todo first visits `add_todo_form()` and marks `hx-action` and the title's
`name` binding together, then moves to `add()`. These source-only steps don't
apply any response fragments. Other actions still start at their handler.

A side rail shows the function, marked expressions, and viewport. Scroll glides use cubic
easing. `top` places the range 28 px below the sticky header; `bottom` follows
ranges outside the viewport's 10–80% zone, placing them near 42% height.
If the whole function and highlighted range are already visible, `bottom` leaves
scrolling alone. Explicit `top` alignment still applies.
Manual input stops automatic scrolling for the rest of that request.
Steps with a `button` label get a bookmark to the left of their rail marker,
visible only after the flow finishes. The small diamond expands into a label
with a flat left edge and pointed right end on hover or keyboard focus,
without changing color.
Clicking it jumps to the step's range
without sending a request or replaying swaps. A new flow removes the old buttons.

htmx still performs every insertion, replacement, and deletion. The trace groups
the prepared tasks through `htmx.swap()`, and the original request waits on
`ctx.extensionPromise`. Text values cross-slide in a shared grid cell; other
fragments swap in place. Deletions get a 320 ms exit before removal.
Controls stay inert until the trace finishes, but the source remains scrollable.
Changing tabs or enabling reduced motion stops animation and applies every
remaining swap. Missing or invalid scenarios fall back to ordinary htmx swaps.

### Customize a scenario

Scenarios in `web/preview-flow.js` are keyed by pathname and HTTP status.
Defaults apply by position; every field can be overridden:

| Step | `to` | `highlight` | `duration` (ms) | `scroll` |
| --- | --- | --- | --- | --- |
| First | `top` | `false` | 140 | `0` |
| All later steps | `bottom` | `true` | 300 | `auto` |

Only `range` is required. Ranges use inclusive, 1-based Unicode line/column
coordinates in `src/todos.rs`; imports count toward line numbers. Whole-line
ranges and arrays of separate spans also work. Update them when Rust lines move.

`focus` sets the function bounds for this and following steps, until another
step changes them. Use `to: 'top'` and `highlight: false` to enter a new function.
`target` is a CSS selector matched against incoming fragment content, or the
old target for a deletion. Set `target: false` for a source-only highlight.
Unmatched steps are skipped, so a filtered-out insertion doesn't light up its
code. Use `swap` to distinguish `outerHTML` from `delete`. Without `target`, a
highlighted step applies all remaining tasks together; `target: false` claims none.

`duration` is the post-scroll beat: up to 180 ms before the UI swap, then the
remaining time afterward. `duration: 0` skips those waits. `scroll: 'auto'` uses
`clamp(380, distance × 1.1, 900)` milliseconds; a non-negative number overrides
it, and `0` jumps instantly. `highlight: false` leaves that step unmarked without
erasing earlier marks. `note` remains reserved for captions.

```js
[
  { range: '342:4-342:16', focus: '342-351', button: 'add_todo_form()' },
  { range: ['344:41-344:69', '345:19-345:45'], target: false, duration: 1500 }, // hx-action + name
  { range: '69:10-69:12', focus: '69-91', button: 'add()', to: 'top', highlight: false, duration: 140, scroll: 0 },
  { range: '81:9-81:23', target: '#add-todo-form' }, // add_todo_form() in the response
]
```

`timing` controls the code/UI delay, follow pause, value animation, and resting
tint. The trace illustrates returned fragments, not Rust execution or database
timing.

## Test

`bun test ./examples/vixen-preview/tests/preview-trace.test.js` checks range
coordinates, step defaults, branch/task binding, scroll timing, and response
preservation without a WASM build.

`bun run --cwd examples/vixen-preview test` also runs the WASM tests (after
building WASM). `cargo test -p vixen-preview` checks the source-panel helpers.
