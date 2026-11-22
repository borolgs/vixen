# vixen-preview

## Runtime

- `src/preview.rs` renders the `hello` and `todos` tabs and embeds Rust source
  with `include_str!`. Tab switches preserve DOM and unfinished edits.
- Each mounted preview owns a WASM `PreviewApp` (`src/browser.rs`). Requests
  clone its router, sharing todo/search/filter state; reload resets everything.
- `web/main.ts` keeps htmx serialization and swaps, replacing only `ctx.fetch`.
  `web/fetch.ts` dispatches same-origin `/__axum__/…` requests to WASM; other
  requests use network fetch. Swap targets stay inside the requesting preview.
- `src/todos.rs` returns targeted `partial!` responses, including validation
  errors. Toggle/save emit either the item or its deletion, never both.

## Todos trace (`web/preview-flow.ts`)

WASM runs first. The fetch hook only compiles the scenario; playback starts once
htmx has prepared swap tasks. It illustrates returned fragments, not Rust
execution. Hello keeps its separate animation in `web/main.ts`.

- Scenarios use pathname/status keys. Steps accept `range`, optional `focus`
  (function bounds, inherited until changed), `target` (incoming-content selector;
  old target for delete), `swap` (style filter), `to`, `highlight`, `duration`,
  `scroll`, and `button` (post-flow source bookmark label). `target: false` marks source without claiming any response tasks,
  as in Add Todo's form prelude.
  `note` is reserved. Coordinates are inclusive, 1-based Unicode line/columns in
  `src/todos.rs`, including imports. Update coordinates when Rust moves.
- `stepDefaults`: first step → top, no mark, 140 ms; later → bottom, mark,
  300 ms. Scroll defaults to `0` on the first step and `auto` (distance-based
  380–900 ms) afterward; numeric overrides and zero remain valid. `duration` includes the code→UI lead (up to 180 ms).
  Range arrays mark spans together; unmatched target branches are skipped.
- htmx owns the DOM. Intercept the original before-swap, retain its task objects,
  and run batches through public `htmx.swap()`. The original request awaits
  `ctx.extensionPromise`; batch contexts are identified by a WeakMap. Never
  recursively intercept a batch or replay a consumed task. Unclaimed tasks and
  interrupted runs must still apply their remaining swaps.
- Marks accumulate and soften after the final reading beat; hover links code/UI.
  The rail tracks the function and marks. Manual input stops auto-follow for
  that request. Bookmark buttons stay hidden until completion and only scroll;
  they must never send requests or replay swaps. Reset removes the old buttons.
  Keep controls inert until completion and restore focus afterward.
- Reduced motion or hidden panels/documents skip animation, not state updates.
  Invalid scenarios must preserve the response. Todo 4xx responses still swap;
  they aren't transport failures. Animation failures must not block deletions.
- `web/preview-trace-plan.ts` contains range validation and task binding without
  browser globals; `tests/preview-trace.test.js` runs without WASM. Use browser
  checks for sequencing, input/focus behavior, interruption, and mobile layout.

## Build and checks

From this directory: `bun run build`, then `bun run serve` (port 4007).
`build` runs `build:wasm` and `build:web`; the latter includes `typecheck` and
can rebuild the frontend alone. Edit `src/` and `web/`, not generated `dist/`.
Rust source changes need a WASM rebuild even when only the displayed code changed.

`web/*.ts` uses permissive `any` annotations for now. `bun run typecheck` checks
browser code without generated WASM; `web/wasm.d.ts` declares the external import.
`build:web` copies HTML/CSS assets and the JS bundle into `dist/`, not TS sources.

`cargo test -p vixen-preview` checks native helpers; `bun test ./tests` uses
built WASM. `bun run test:trace` runs the trace tests alone, without WASM.
Also check browser choreography after animation changes.
Build prerequisites and layout: `README.md`.
