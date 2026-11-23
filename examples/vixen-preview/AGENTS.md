# vixen-preview

WASM demo for the documentation site. No automated tests here; use the build
and check changed flows in the browser. Edit `src/` and `web/`, not `dist/`.

## Files

- `src/{hello,counter,todos}.rs`: each example's UI, actions, and state.
- `src/preview.rs`: tabs, layout, and full Rust listings via `include_str!`.
- `src/lib.rs`, `src/browser.rs`: router and WASM bridge; each mount owns its state.
- `web/main.ts`, `web/fetch.ts`: mounting, tab switching, and `/__axum__/` transport.
- `web/flow/scenarios/{hello,counter,todos}.ts`: per-example `scenarios` and `startLine`.
- `web/flow/flow.ts`, `web/flow/ranges.ts`: shared player, step types, and range parser.
- `web/style.css`, `web/flow/flow.css`: layout and flow styles; `flow.css` is the CSS entry.
- `../../site/build.sh`: builds this preview and copies `dist/` into the site.

## Update an example's flow

Change `src/<name>.rs` and `web/flow/scenarios/<name>.ts` together.
For narration-only changes, edit the scenario, not the shared player.
`web/main.ts` already connects each example to its scenario.

Scenarios map request pathname → HTTP status → ordered steps, e.g.
`scenarios['/hello/greet'][422]`. Keep success and validation branches in sync
with the handler. Use `satisfies Scenarios` and `satisfies readonly Step[]`.

- `range`: inclusive, 1-based Unicode line/column coordinates in the full Rust
  file, including imports. `'26:4-26:7'`, whole lines (`'26-30'`), or arrays of
  spans. Recheck coordinates whenever Rust moves, even after formatting.
- `startLine`: initial source position, also restored when switching tabs.
- `target`: selector in the incoming fragment; for deletion, the outgoing target.
  No match omits the step; no `target` means source-only. `swap` filters response
  branches (e.g. `'delete'`), not how htmx performs the swap.
- `button`: bookmark label. `skip: true` keeps it as a bookmark without playback.
- `to: 'top'`: align below the source header; otherwise scroll only if needed.
  `highlight: false` skips marking, not delivery. `duration` overrides the pause.
- Other options and defaults: `Step` and `FlowOptions` in `web/flow/flow.ts`.

The player highlights code, delivers prepared response fragments through htmx,
then marks the UI. Keep swaps in htmx and playback out of `web/fetch.ts`.
Tab switches reset flow marks, not demo DOM or WASM state.

## Build and preview

Requires Bun, Python 3, Rust with `wasm32-unknown-unknown`, and `wasm-pack`
(version pinned in `../../.github/workflows/ci.yml`). Run `bun install` at the
repository root, then from this directory:

```sh
bun run build
bun run serve
```

Open <http://127.0.0.1:4007/>. After web-only changes, `bun run build:web`
runs TypeScript checking and rebundles JS/CSS. Any Rust change needs
`bun run build`, including changes to displayed source.

Check the changed action and its response branches with Animate on and off;
check bookmarks and tab switching. For the embedded site, see
[site/AGENTS.md](../../site/AGENTS.md).
