# config

A vixen app served under a base path, `/config`.

```bash
bun install           # once, for htmx
cargo run -p config   # http://127.0.0.1:4004/config/
```

## What it shows

- `base_path: "/config"` in `build.rs`. Without that override,
  `Config::default()` reads `VIXEN_BASE_PATH`.
- `vixen::mount!(router)` nests the whole router under `/config/` and redirects
  `/config` there. The routes themselves stay unprefixed.
- `Ping::action()`, `assets!()`, `asset!("./assets/ship.png")` and
  `a href=(HomePath)` all render `/config/…` URLs. `href!(HomePath)` does the
  same outside markup, e.g. for `Redirect::to`.
