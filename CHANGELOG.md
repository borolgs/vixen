# Changelog

## Unreleased

### Added

- `#[id]` on a one-field tuple struct: a dynamic id, `TodoId(7)` is `todo-7`.
- `#[fragment(CartId)]` and `#[fragment(TodoId(todo.id))]`: a fragment with an existing `#[id]` struct, static or dynamic.

### Changed

- `Id::sel(&self)` replaces the `ID` and `SEL` consts, which `#[id]` types lose too.
- `Fragment::new` takes the id: `Fragment::new(&id, markup)`.
- `Selector` renders escaped.

## 0.2.0 - 2026-09-27

### Added

- Base-path support through `Config::base_path` or `VIXEN_BASE_PATH`, with
  `base_path!`, `mount!`, `href!` and `HxAction::base`.
- Prefix handling for URLs rendered by `#[action]`, `assets!` and `#[route]`.
- A base-path example in `examples/config`.
- `VIXEN_<FIELD>` environment defaults for every `Config` field.
- `asset!` for compile-time-checked URLs to static files with hashed names,
  picked up by `Config::static_glob`.
- `RouterExt::{view, action}` register routes from the first and last handler
  arguments.

### Changed

- Renamed `#[view_path]` to `#[route]`.
- Re-exported `build` and `Config` at the `vixen` crate root instead of under
  `vixen::bundler`; `build` now takes `Config` by value.
- `assets!()` now expands to `Markup`.
- New `Config` fields break struct literals without `..Default::default()`.

## 0.1.2 - 2026-09-26

### Added

- `basecoatui` feature: `vixen::ui::basecoatui` with `Toaster`, `Drawer` and `Dialog`.
- `#[fragment]` exposes `<name>::slot()`, an empty placeholder to swap into later.
- `partial!` accepts `(target, swap) => content`.

### Changed

- `HxPartial::main` and `HxPartial::part` take `impl Into<Markup>` / `impl Into<Part>`.
- `Fragment` converts into `Markup` instead of `HxPartialResponse`.

## 0.1.1 - 2026-09-24

### Added

- `#[fragment]`: markup that works both inline and as a `partial!` part.
- `Id` trait for `#[id]` types.
- `examples/components`.

## 0.1.0 - 2026-09-22

Initial release.
