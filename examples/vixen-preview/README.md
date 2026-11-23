# vixen-preview

`fetch` → WASM-compiled `axum::Router` → `Response`.

Still a rough draft: the player, scenarios, and UI are work in progress.

## Run

From the repository root (Rust, Bun, and Python 3 required):

```bash
bun install --frozen-lockfile
rustup target add wasm32-unknown-unknown
cargo install wasm-pack --version 0.15.1 --locked
bun run --cwd examples/vixen-preview build
bun run --cwd examples/vixen-preview serve
```

Open <http://127.0.0.1:4007/>. Deploy `examples/vixen-preview/dist/` to a static host.

## Frontend

Browser code lives under `web/` and uses TypeScript. The new flow and its
scenarios are typed; permissive `any` annotations remain in migrated legacy code.

From the repository root:

```bash
bun run --cwd examples/vixen-preview typecheck
bun run --cwd examples/vixen-preview build:web
```

`build:web` checks TypeScript, bundles `web/main.ts` into `dist/main.js`, and
bundles `web/flow/flow.css` into `dist/style.css`. It copies HTML and theme CSS,
not TypeScript sources. WASM stays an external import from
`dist/pkg/vixen_preview.js`; build it once with `build:wasm` before serving.
`build` runs both build steps. Type checking doesn't need generated WASM files.

## Examples and state

The Hello, Counter, and Todos tabs put the UI beside one Rust file, or above
it on narrow screens.
Hello's form, explanatory text, and greeting response live in `src/hello.rs`.
Blank or whitespace-only names return HTTP 422 with a warning in the greeting
slot, leaving the form unchanged. Its flow covers both success and validation.
Counter lives in `src/counter.rs`. Its buttons bind `Add::action().by(-1)`
and `.by(1)`; one response updates the number and its Zero / Even / Odd heading.
It introduces `axum::State` with an `Arc<tokio::sync::Mutex<i64>>`, starting at
zero for each mounted preview. Addition saturates at the `i64` limits.

Todos lives in `src/todos.rs`; `src/lib.rs` assembles the shared preview router.
Source text scales to fit the code panel. Switching tabs keeps the
demo DOM, including unfinished edits and Rust state. It resets the flow's
marks, bookmarks, and source scrolling.

Each mounted preview owns a WASM `PreviewApp` with one `axum::Router`.
Requests clone that router, sharing its `axum::State` rather than rebuilding
the application. `Ctx::middleware` exposes that state through `ctx()` in todo
handlers. `ReqCtx` uses Tokio's task-local scope without a Tokio runtime;
`wasm-bindgen-futures` polls the request futures. Todos live in an
`Arc<tokio::sync::Mutex<TodoState>>`. Handlers acquire the lock with
`ctx().todos.lock_owned().await`, then update state and render without further
awaits.
New previews start with the standalone example's sample todos. Reloading the
page restores those samples. Nothing uses localStorage.

The todo actions follow [the standalone example](../todos/src/pages/todos/mod.rs).
Search and filter live in the preview state rather than the host page's URL.
All interactive examples show their full Rust source, including imports, and use the
same flow player.

## Flow

The player holds the prepared WASM response and delivers it step by step:
finish the code highlight, apply matching swaps through htmx, then highlight
the UI. The demo stays `inert` from request start through the final softening
animation, so users can't edit fields or start another action mid-flow. The
source and tabs stay interactive. Completion and error paths unlock the demo.

Marks draw from left to right and accumulate. After the walkthrough, they wait
400 ms, then fade to 35% of their original background opacity over 400 ms.
Text stays unchanged. On the next run, the backgrounds fade out over 250 ms,
then the player restores the original syntax markup. The default step pause
is 450 ms.

Each action first visits the UI function containing its `hx-action`, then
highlights the action builder and field bindings together. Toggle, Edit, and
Cancel bind their ID through `.id(todo.id)` rather than `FIELD`. The walkthrough
then highlights the handler's action type and visits the response expressions,
including error branches. Each action also has a skipped step linking to its
struct declaration through a bookmark.
Rust has already produced the response; the walkthrough controls when its
fragments reach the UI, not when the server executes.

The right side of the tabs row has one **Animate** checkbox shared by all
examples in that preview, enabled by default. Uncheck it
to deliver the whole response and immediately mark all selected code ranges and
UI in the final pale color, then scroll instantly to the last selected step
with a `button` and without `skip: true`. Without such a step, the scroll
position stays unchanged.
There are no intermediate scrolls, pauses, or draw/fade animations. Branch
selection and `highlight: false` still apply; bookmarks navigate instantly.

The checkbox changes the next run, without replaying requests or marks. It's
disabled only while an animated request in that preview is active. During a
static request it stays enabled, without dimming; changing it applies to the
next request. Flows sharing a `controls` container share the checkbox. The
first `connectFlow` call's `animated` option sets its initial state; the
default is `true`.

Source jumps use a short cut: the code scrolls instantly, then settles from
a 32 px offset in the scroll direction over 140 ms. Text stays fully opaque,
and the header stays fixed. Highlighting starts after the cut. Visible ranges stay still
unless `to: 'top'` aligns them 28 px below the sticky header. A fully visible range
within three line heights of that position stays put. Farther visible ranges
align instantly, without a cut or smooth scrolling. Manual input cancels the
cut and stops automatic following for the rest of the run. Steps with a `button` label get
plain header bookmarks after completion. These only scroll, without repeating
requests or swaps.

Hidden tabs/documents, request aborts, and reduced motion stop playback and
flush pending swaps. Playback errors also flush the response. Invalid scenarios
fall back to htmx's normal swap before the player takes ownership.
There is no minimap, paired hover, counter animation, or deletion animation.
The old player, planner, and trace styles are removed. Typed range parsing
lives in `web/flow/ranges.ts`. The fetch adapter has no playback hooks.

### Customize a scenario

Scenarios in `web/flow/scenarios/{hello,counter,todos}.ts` are keyed by
pathname and HTTP status.
The player in `web/flow/flow.ts` receives them as an argument.

Each module also exports `startLine`, a 1-based line number passed to
`connectFlow` as an option. On the tab's first display and after switching back,
the source scrolls there instantly to skip imports, without highlighting or
sending a request. Initial positioning and resets are exact, without the
three-line tolerance used during playback. Without `startLine`, it returns to the top. `connectFlow`
returns `reset()`, which the tab handler calls directly after switching panels.

Only `range` is required. It uses inclusive, 1-based Unicode line/column
coordinates in the corresponding Rust file; imports count toward line numbers. Whole-line
ranges and arrays of separate spans also work. Update them when Rust lines move.

`to: 'top'` aligns with a three-line tolerance for fully visible ranges;
otherwise only clipped ranges scroll.
`scroll: 'cut'` is the default; `'instant'` skips the cut, and `'smooth'` animates
the scroll itself. Visible ranges outside the tolerance align instantly in every mode.
The step's bookmark uses the same mode. Disabling Animate or
preferring reduced motion makes bookmark navigation instant.
`highlight: false` skips marking, not swaps, without clearing earlier marks.
`duration` overrides the pause after that step; zero removes the pause.
`skip: true` keeps the step out of playback in both modes: no scrolling,
marks, or pause. Its `button` still appears after completion; any tasks not
already delivered by other steps are flushed at the end.
`button` supplies the bookmark label. There are no positional defaults,
inherited function bounds, or implicit UI targets.

`target` selects an element in the returned fragment, not an arbitrary existing
element in the demo. Unmatched steps are omitted. `swap` distinguishes response
branches such as replacement and deletion; it filters tasks without changing
their swap style. Each task is delivered once; unmatched tasks are flushed at
the end. For deletion, selection uses the outgoing target; the item is removed
after its code highlight. Without `target`, a step only visits the source.

```ts
[
  { range: '71:8-71:14', skip: true, button: 'AddTodo' },
  { range: '353:4-353:16', to: 'top', highlight: false, button: 'add_todo_form()' },
  { range: ['355:25-355:41', '356:25-356:44'], duration: 1000 }, // action + field
  { range: '76:20-76:26', to: 'top', button: 'add()' },
  { range: '88:9-88:23', target: '#add-todo-form input[name="title"]' },
]
```

Pass `swap: ctx => htmx.swap(ctx)` to `connectFlow` so htmx owns DOM updates.
`connectFlow` also accepts `duration`, `fade`, `rest`, and `soften` in milliseconds,
all allowing zero. `rest` is the delay before marks soften; `soften` controls
that transition. The demo unlocks once softening finishes. Static mode ignores
these timings.
The CSS entry is `web/flow/flow.css`, which imports the existing base stylesheet.

## Test

`bun run --cwd examples/vixen-preview test:flow` checks scenario coordinates,
response selection, delivery deduplication, Unicode range parsing, and the
fetch adapter without a WASM build. Browser checks cover code-before-swap ordering,
scrolling, cancellation, the UI lock, and both animation modes.

With the preview server running and `playwright-cli` installed, run the browser
regression from this directory:

```bash
NO_PROXY='*' no_proxy='*' playwright-cli -s=flow open http://127.0.0.1:4007/
NO_PROXY='*' no_proxy='*' playwright-cli -s=flow run-code "$(< tests/flow.browser.js)"
NO_PROXY='*' no_proxy='*' playwright-cli -s=flow run-code "$(< tests/hello.browser.js)"
NO_PROXY='*' no_proxy='*' playwright-cli -s=flow run-code "$(< tests/counter.browser.js)"
NO_PROXY='*' no_proxy='*' playwright-cli -s=flow run-code "$(< tests/scroll.browser.js)"
NO_PROXY='*' no_proxy='*' playwright-cli -s=flow run-code "$(< tests/start-line.browser.js)"
NO_PROXY='*' no_proxy='*' playwright-cli -s=flow run-code "$(< tests/tabs.browser.js)"
playwright-cli -s=flow close
```

These checks use the UI checkbox to exercise both modes with real WASM responses.

`bun run --cwd examples/vixen-preview test` also runs the WASM tests (after
building WASM). `cargo test -p vixen-preview` checks the source-panel helpers.
