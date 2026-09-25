# Changelog

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
