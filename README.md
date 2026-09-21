#  vixen

An axum-based library for rapid web development, currently focused on the view
layer. Experimental and opinionated — think of it as `axum-extra-extra`.

Conceived as a thin glue layer over [maud](https://github.com/lambda-fairy/maud)
and the `axum-*` crates. Under active development: the API is unstable and large
parts will still be reworked.

## The ideas behind vixen

1. **Views are rendered right in the handlers.** maud rules.
    - Something JSX-like may come later — more flexible, more extensible. For
      now, simple and reliable wins.
    - vixen does not add anything on top of maud yet. A component library
      built on [Basecoat](https://basecoatui.com) is planned.

2. **htmx 4 handles frontend interactivity.** vixen adds two basic
   abstractions:
    - [`#[action]`][action] uses one type for the route, its form fields, and
      the markup that calls it.
    - [`partial!`][partial] puts the main swap and any number of targeted HTML
      fragments into one response.

3. **TS and CSS live next to the page they belong to.** The browser is a
   JavaScript platform, and vixen embraces that boundary. When custom
   client-side code is needed, it uses the JavaScript ecosystem directly — if
   we have to write JS, we might as well do it properly.
    - [`vixen::bundler::build`][build] finds the entry points, bundles their TS
      and CSS with Bun, and passes the asset manifest to Rust.
    - [`assets!`][assets] emits the `<script>` and `<link>` tags for the current
      page.
    - [`assets_router!`][assets_router] embeds the bundled files in the binary
      and returns an axum router that serves them.
    - For now, that bundler is Bun: simple and fast. Later there may be an
      option without an external runtime, possibly from the Oxc ecosystem.

The first two ideas fit in one file:

```rust
use std::sync::atomic::{AtomicI64, Ordering};

use axum::{Router, response::IntoResponse};
use vixen::{action, id, maud::{Markup, html}, partial, routing::RouterExt, view_path};

static COUNT: AtomicI64 = AtomicI64::new(0);

#[id]
struct CountId;

#[view_path("/")]
struct HomePath;

async fn home(_: HomePath) -> Markup {
    html! {
        button hx-action=(Add::action().by(-1)) { "−" }
        output id=(CountId) { (COUNT.load(Ordering::Relaxed)) }
        button hx-action=(Add::action().by(1)) { "+" }
    }
}

#[action("/add")]
struct Add {
    by: i64,
}

async fn add(Add { by }: Add) -> impl IntoResponse {
    let count = COUNT.fetch_add(by, Ordering::Relaxed) + by;

    partial!(CountId => html! { (count) })
}

fn router() -> Router {
    Router::new().typed_get(home).typed_post(add)
}
```

`Add` is the route, the form extractor, and what the buttons render. `CountId`
is both the `id` in the page and the target in the response.
[`examples/counter`][counter] adds the surrounding page and bundles htmx.

## Installation

vixen is not on crates.io yet. Add it from GitHub, together with the crates its
macros need to resolve in the calling crate:

```toml
[dependencies]
axum-vixen = { git = "https://github.com/borolgs/vixen" }
axum = "0.8"
axum-extra = "0.12"
maud = "0.27"

[build-dependencies]
axum-vixen = { git = "https://github.com/borolgs/vixen" }
```

The package is `axum-vixen`; the crate it provides is `vixen`, so code says
`use vixen::…`. The build dependency is needed only when `build.rs` calls
[`vixen::bundler::build`][build]. The bundler requires [Bun](https://bun.sh) to
be installed and available on `PATH` while the app is built.

## Crates

| crate | what |
|---|---|
| [`axum-vixen`][vixen] | the facade crate, imported as `vixen` — re-exports `maud`, `axum_extra::routing`, `axum_htmx` as `hx`, `axum-vixen-bundler` as `bundler`, plus the macros |
| [`axum-vixen-macros`][vixen-macros] | [`#[action]`][action], [`#[view_path]`][view_path], [`#[id]`][id], [`assets!`][assets] / [`assets_router!`][assets_router] |
| [`axum-vixen-bundler`][vixen-bundler] | `build.rs` helper that bundles per-page TS/CSS with bun; apps reach it as `vixen::bundler` |

## Examples

[`examples/counter`][counter] — a counter in one `main.rs` that fits on a
screen, with its `index.ts` and `index.css` beside it. Start here.

[`examples/todos`][todos] — a todo list on one page: every macro once, plus a
per-page `index.ts`.

```bash
bun install                      # once, for the frontend deps
cargo run -p counter             # http://127.0.0.1:4002/
cargo run -p todos               # http://127.0.0.1:4001/
```

[action]: https://borolgs.github.io/vixen/vixen/attr.action.html
[view_path]: https://borolgs.github.io/vixen/vixen/attr.view_path.html
[id]: https://borolgs.github.io/vixen/vixen/attr.id.html
[partial]: https://borolgs.github.io/vixen/vixen/macro.partial.html
[assets]: https://borolgs.github.io/vixen/vixen/macro.assets.html
[assets_router]: https://borolgs.github.io/vixen/vixen/macro.assets_router.html
[build]: https://borolgs.github.io/vixen/vixen_bundler/fn.build.html
[vixen]: https://borolgs.github.io/vixen/vixen/
[vixen-macros]: https://borolgs.github.io/vixen/vixen_macros/
[vixen-bundler]: https://borolgs.github.io/vixen/vixen_bundler/
[counter]: https://github.com/borolgs/vixen/tree/main/examples/counter
[todos]: https://github.com/borolgs/vixen/tree/main/examples/todos
