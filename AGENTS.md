# vixen

An axum-based library for rapid web development, currently scoped to the **view
layer**: axum + maud + htmx. A thin glue layer over the `axum-*` ecosystem —
experimental, opinionated, and still moving.

Cargo workspace, resolver 3, edition 2024, stable toolchain, MSRV 1.97.
syn is **3.x**.

## Layout

- `vixen/` — the facade crate. Its own types (`HxPartial`, `HxAction`, …) and
  the macros sit at the root; `vixen::maud`, `vixen::hx` (all of
  `axum_htmx`, nothing else), `vixen::routing` (`axum_extra::routing`) and
  `vixen::bundler` (`vixen-bundler`) are plain re-exports. Runtime halves of the
  macros live here (`action`, `partial`, `assets`).
- `vixen-macros/` — proc macros: `#[action]`, `#[view_path]`, `#[id]`,
  `assets!`, `assets_router!`.
- `vixen-bundler/` — `build.rs` helper, reached as `vixen::bundler`: apps list
  `vixen` under `[build-dependencies]` too. Runs bun over per-page entrypoints
  and hands the resulting manifest to rustc as `VIXEN_MANIFEST`.
- `examples/todos/` — a small app: a todo list in `pages/todos/mod.rs` with
  its `index.ts` beside it. `cargo run -p todos` → <http://127.0.0.1:4001/>.
- `examples/counter/` — the smallest app: a counter in one `main.rs` with
  `index.ts` and `index.css` beside it; `build.rs` points `entry_glob` at
  `src/index.ts`. `cargo run -p counter` → <http://127.0.0.1:4002/>.
