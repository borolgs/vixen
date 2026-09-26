# config

A vixen app served under a base path, `/config`.

```bash
bun install           # once, for htmx
cargo run -p config   # http://127.0.0.1:4004/config/
```

## What it shows

- `base_path: "/config"` in `build.rs`. Left empty, `VIXEN_BASE_PATH` from the
  build environment is used instead.
- `vixen::mount!(router)` nests the whole router under `/config/` and redirects
  `/config` there. The routes themselves stay unprefixed.
- `Ping::action()`, `assets!()` and `a href=(HomePath)` emit `/config/…` URLs;
  `href!(HomePath)` does the same outside markup, e.g. for `Redirect::to`.
