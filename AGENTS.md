# vixen

An axum-based library for rapid web development, currently scoped to the **view
layer**: axum + maud + htmx. A thin glue layer over the `axum-*` ecosystem —
experimental, opinionated, and still moving.

Cargo workspace, resolver 3, edition 2024, stable toolchain, MSRV 1.97.
syn is **3.x**.

`vixen` is taken on crates.io, so the packages are `axum-vixen`,
`axum-vixen-macros` and `axum-vixen-bundler` (what `-p` takes), each with
`[lib] name` pinning the crate to `vixen`, `vixen_macros`, `vixen_bundler`.
Macros emit `::vixen::` paths, so the facade's lib name must stay `vixen`.

## Layout

- `vixen/` — the facade crate. Its own types (`HxPartial`, `HxAction`, …) and
  the macros sit at the root. It re-exports `vixen::maud`, all of `axum_htmx`
  as `vixen::hx`, `axum_extra::routing` as `vixen::routing`, and
  `vixen_bundler::{build, Config}`. Runtime support for the macros lives here
  (`action`, `fragment`, `partial`, `assets`, `base_path`).
- `vixen-macros/` — proc macros: `#[action]`, `#[view_path]`, `#[id]`,
  `#[fragment]`, `assets!`, `assets_router!`.
- `vixen-bundler/` — the `build.rs` helper behind `vixen::{build, Config}`.
  Apps also list `vixen` under `[build-dependencies]`. It runs Bun over
  per-page entry points and passes `VIXEN_MANIFEST` to rustc, plus
  `VIXEN_BASE_PATH` when `Config::base_path` is set.
- `examples/todos/` — a small app: a todo list in `pages/todos/mod.rs` with
  its `index.ts` beside it. `cargo run -p todos` → <http://127.0.0.1:4001/>.
- `examples/counter/` — the smallest app: a counter in one `main.rs` with
  `index.ts` and `index.css` beside it; `build.rs` points `entry_glob` at
  `src/index.ts`. `cargo run -p counter` → <http://127.0.0.1:4002/>.
- `examples/config/` — demonstrates `Config::base_path` and `vixen::mount!`.
  `cargo run -p config` → <http://127.0.0.1:4004/config/>.
