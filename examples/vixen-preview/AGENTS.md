# vixen-preview

## Runtime

- `src/preview.rs` renders `hello`, `counter`, and `todos` and embeds Rust source
  with `include_str!`. Tab switches preserve DOM and unfinished edits.
- Each mounted preview owns a WASM `PreviewApp` (`src/browser.rs`). Requests
  clone its router, sharing counter/todo/search/filter state; reload resets everything.
- `web/main.ts` keeps htmx serialization and swaps, replacing only `ctx.fetch`.
  `web/fetch.ts` dispatches same-origin `/__axum__/…` requests to WASM; other
  requests use network fetch. Swap targets stay inside the requesting preview.
- `src/hello.rs` contains the greeting action and its UI; `src/lib.rs` assembles
  the preview router. Interactive examples share the `UI | Code` layout.
- `src/counter.rs` uses `State<Arc<Mutex<i64>>>`, not a global static. One action
  returns the value and Zero / Even / Odd fragments; each preview starts at zero.
- `src/todos.rs` returns targeted `partial!` responses, including validation
  errors. Toggle/save emit either the item or its deletion, never both.

## Flow

- `web/flow/flow.ts` is the player. Pass scenarios to `connectFlow`; keep
  example-specific ranges and selectors in `web/flow/scenarios/{hello,counter,todos}.ts`.
  All interactive examples use this player.
- The player takes over `htmx:before:swap` after validating the scenario.
  Each step waits for its code animation, delivers matching prepared tasks
  through the injected `swap: ctx => htmx.swap(ctx)`, then marks the UI.
  Keep htmx's insertion, cleanup, and processing; don't implement DOM swaps.
  `ctx.extensionPromise` keeps the request open until delivery finishes.
- Deliver each task once, even if several steps explain it. Flush all remaining
  tasks on interruption or playback errors, including tasks with no matching
  step. Clone swap specifications to disable htmx's own animation delays.
- The demo is `inert` from request start through the final softening animation.
  Reject additional requests while locked; unlock at `htmx:finally:request`,
  including transport and playback errors. Keep the source and tabs interactive.
- `Animate` sits at the right of `.preview-tabs`, outside the tablist's ARIA
  group. Flows sharing a `controls` container share one checkbox and setting,
  checked by default (`animated: true`).
  It changes the next run's mode without replaying marks or requests and is
  disabled only while an animated request in that preview is active. Static
  requests keep it enabled; each request keeps the mode it started with.
  With it off, deliver the whole response,
  mark selected ranges/UI in their pale state, then scroll instantly to the
  last selected step with a `button` (no bookmark means no scrolling). Skip
  intermediate scrolling, pauses, and CSS animations.
  Keep response branch selection and `highlight: false`; bookmarks still work.
- Ranges are inclusive, 1-based Unicode line/columns in the example's Rust file,
  including imports. Arrays mark separate expressions together. Use the typed
  `parseRange` / `SourceRange` in `web/flow/ranges.ts`; update coordinates when
  Rust moves.
  `target` matches incoming fragments (the old target for delete); `swap`
  only filters the response branch. Missing targets mean source-only steps.
- `to: 'top'` aligns below the sticky header; otherwise visible ranges don't
  scroll. Manual source input stops following for that run. Marks draw left
  to right and stay until the next run fades their backgrounds, not their text.
  Cleanup restores a single saved syntax-highlighted HTML snapshot.
- `button` adds a plain header bookmark after completion. It only scrolls,
  never sends requests or repeats swaps. A new request removes the old buttons.
  `highlight: false` skips marking; `duration` overrides the step pause,
  including zero. `connectFlow` defaults to a 450 ms step and 250 ms fade.
- `selectTab` calls each flow's `reset()` directly after switching panels.
  Reset stops playback, clears marks and bookmarks, and restores `startLine`
  when visible. Keep the demo DOM and WASM state; late responses still swap
  without restarting playback. Don't observe DOM mutations for tab changes.
- An abort, hidden tab/document, or reduced motion stops animated playback.
  The `finally` block delivers remaining swaps, including after rendering errors.
  Validate before taking over; bad scenarios mustn't discard a response or turn
  validation responses into transport errors. Don't restore queued-request or
  overlapping-playback handling while the demo is locked.

The old player, planner, and trace CSS are removed. Keep playback out of the
fetch adapter: it only dispatches requests and preserves abort/response behavior.
The CSS entry, `web/flow/flow.css`, imports the common base styles.

## Build and checks

From this directory: `bun run build`, then `bun run serve` (port 4007).
`build` runs `build:wasm` and `build:web`; the latter includes `typecheck` and
bundles JS and CSS without rebuilding WASM. Edit `src/` and `web/`, not `dist/`.
Rust source changes need a WASM rebuild even when only displayed code changed.

Permissive `any` annotations belong to the mechanically migrated legacy code,
not new code. The new flow uses explicit contracts, inferred DOM/event types,
and `satisfies` for scenarios. `typecheck` also checks `tests/*.types.ts`.
`web/wasm.d.ts` declares the external WASM import, so type checking doesn't
need generated WASM files.

`bun run test:flow` runs flow selection, Unicode range parsing, and fetch-adapter
checks without WASM. `tests/flow.browser.js` checks both animation modes against real WASM,
including the UI lock, response delivery, errors, and bookmarks.
`tests/hello.browser.js` and `tests/counter.browser.js` cover the smaller examples.
Counter also checks isolated mounts and state retention. Check
scrolling and narrow screens. `bun test ./tests` additionally uses built WASM;
`cargo test -p vixen-preview` checks native helpers.
Build prerequisites and layout: `README.md`.
