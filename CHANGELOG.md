# Changelog

## Unreleased

### Added

- Base-path support through `Config::base_path` or `VIXEN_BASE_PATH`, with
  `base_path!`, `mount!`, `href!` and `HxAction::base`.
- Prefix handling for URLs rendered by `#[action]`, `assets!` and
  `#[view_path]`.
- A base-path example in `examples/config`.
- `VIXEN_<FIELD>` environment defaults for every `Config` field.
- `asset!` for compile-time-checked URLs to static files with hashed names.

### Changed

- Re-exported `build` and `Config` at the `vixen` crate root instead of under
  `vixen::bundler`; `build` now takes `Config` by value.
- `assets!()` now expands to `Markup`.

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
