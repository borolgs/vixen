#  vixen

An axum-based library for rapid web development, currently focused on the view
layer.

Conceived as a thin glue layer over [maud](https://github.com/lambda-fairy/maud)
and the `axum-*` crates. Under active development: the API is unstable and large
parts will still be reworked.

## The ideas behind vixen

1. **Views are rendered right in the handlers.**
    - Something JSX-like may come later — more flexible, more extensible. For
      now, maud's simplicity and reliability win.
    - vixen provides two helper macros: [`#[id]`][id] and [`#[fragment]`][fragment].
    - Optional [Basecoat](#basecoat) components are available under
      `vixen::ui::basecoatui` with the `basecoatui` feature.

2. **htmx 4 handles frontend interactivity.** vixen adds two basic
   abstractions:
    - [`#[action]`][action] uses one type for the route, its form fields, and
      the markup that calls it.
    - [`partial!`][partial] puts the main swap and any number of targeted parts
      into one response.

3. **TS and CSS live next to the page they belong to.** The browser is a
   JavaScript platform, and vixen embraces that boundary. When custom
   client-side code is needed, it uses the JavaScript ecosystem directly — if
   we have to write JS, we might as well do it properly.
    - [`vixen::build`][build] finds the entry points, bundles their TS and CSS
      with Bun, and passes the asset manifest to Rust.
    - [`assets!`][assets] emits the `<script>` and `<link>` tags for the current
      page.
    - [`asset!`][asset] resolves page-local static files to content-hashed URLs.
    - [`assets_router!`][assets_router] embeds the bundled files in the binary
      and returns an axum router that serves them.
    - For now, that bundler is Bun: simple and fast. Later there may be an
      option without an external runtime, possibly from the Oxc ecosystem.

The first two ideas fit in one file:

```rust
use std::sync::atomic::{AtomicI64, Ordering};

use axum::{Router, response::IntoResponse};
use vixen::{
    action, fragment,
    maud::{Markup, html},
    partial,
    routing::RouterExt,
    view_path,
};

static COUNT: AtomicI64 = AtomicI64::new(0);

#[view_path("/")]
struct HomePath;

async fn home(_: HomePath) -> Markup {
    let count = COUNT.load(Ordering::Relaxed);
    html! {
        (heading(count))
        button hx-action=(Add::action().by(-1)) { "−" }
        (counter(count))
        button hx-action=(Add::action().by(1)) { "+" }
    }
}

#[action("/add")]
struct Add {
    by: i64,
}

async fn add(Add { by }: Add) -> impl IntoResponse {
    let count = COUNT.fetch_add(by, Ordering::Relaxed) + by;

    partial! {
        counter(count),
        heading(count),
    }
}

#[fragment]
fn heading(count: i64) -> Markup {
    let title = match count {
        0 => "Zero",
        n if n % 2 == 0 => "Even",
        _ => "Odd",
    };
    html! { h1 id=(Self) { (title) } }
}

#[fragment]
fn counter(count: i64) -> Markup {
    html! { output id=(Self) { (count) } }
}

fn router() -> Router {
    Router::new().typed_get(home).typed_post(add)
}
```

`Add` is the route, the form extractor, and what the buttons render. Each
`#[fragment]` owns its element ID: in page markup it renders the element; in
`partial!` it becomes an `outerHTML` update for that element.
[`examples/counter`][counter] adds surrounding page markup and bundles htmx.

## Installation

Add vixen together with the crates its macros need to resolve in the calling
crate:

```toml
[dependencies]
axum-vixen = "0.1"
axum = "0.8"
axum-extra = "0.12"
maud = "0.27"

[build-dependencies]
axum-vixen = "0.1"  # only for build.rs below
```

The package is `axum-vixen`; the crate it provides is `vixen`, so code says
`use vixen::…`.

### Page-local assets

[Bun](https://bun.sh) must be on `PATH` while the app builds.

```bash
bun init
bun add htmx.org@4
```

```rust,ignore
// build.rs
fn main() {
    vixen::build(vixen::Config::default());
}
```

By default, [`build`][build] bundles every `src/pages/**/{page,index}.ts`, so a page is a
directory:

```text
src/pages/todos/
├── mod.rs      # handlers
├── index.ts    # import 'htmx.org'; import './index.css';
├── index.css
└── assets/     # files referenced by asset!
```

```rust,ignore
// src/pages/todos/mod.rs — <script> and <link> for the entry next to this file
head { (vixen::assets!()) }

// src/pages/todos/mod.rs — content-hashed URL for a nearby file
img src=(vixen::asset!("./assets/logo.svg"));

// src/main.rs — serves the bundle from the binary under /assets/
Router::new().merge(pages::todos::router()).merge(vixen::assets_router!())
```

### Config

[`Config`][config] controls bundling. `Config::default()` reads
`VIXEN_<FIELD>` variables from the build environment.

| Field | Default | Purpose |
|---|---|---|
| `base_path` | none | URL prefix |
| `bun_cmd` | `bun` | Bun executable |
| `root` | `src` | prefix stripped from entry paths |
| `entry_glob` | `src/pages/**/{page,index}.ts` | input files |
| `static_glob` | `src/**/assets/**/*.{svg,png,jpg,jpeg,gif,webp,avif,ico}` | files copied for `asset!` |
| `assets_prefix` | `assets` | bundle directory and URL |
| `config` | `build.ts` | Bun config |

```rust,ignore
// build.rs
fn main() {
    vixen::build(vixen::Config {
        entry_glob: "src/index.ts".into(),
        ..Default::default()
    });
}
```

The default `build.ts`, next to `Cargo.toml`, exports `Bun.build` options or a
function returning them. Use it for plugins, `define` and extra `entrypoints`;
vixen owns `root`, `outdir`, `metafile` and `naming`. Leave `publicPath` unset.

For an app served from `/app/`, set `base_path` to `"/app"`. Keep route paths
unprefixed and wrap the router in `vixen::mount!`; it serves the router at
`/app/` and redirects `/app` to `/app/`. `#[action]`, `assets!`, `asset!` and
`#[view_path]` include the prefix in rendered URLs. Use [`href!`][href] where a
string is required, such as `Redirect::to`. See
[`examples/config`][config-example] for a complete app served from `/config/`.

### Basecoat

Enable `basecoatui` to render [Basecoat](https://basecoatui.com) components
from Rust:

```toml
[dependencies]
axum-vixen = { version = "0.1", features = ["basecoatui"] }
```

The components use the `basecoat-css` npm package and Tailwind:

```bash
bun add basecoat-css
bun add -d tailwindcss bun-plugin-tailwind
```

Enable Tailwind in [`build.ts`](#config):

```ts
// build.ts
import tailwind from 'bun-plugin-tailwind';

export default { entrypoints: [], plugins: [tailwind] };
```

Import the styles from the page's CSS:

```css
/* src/pages/index.css */
@import "tailwindcss";
@import "basecoat-css";
@source "../";
```

Load htmx, the Basecoat scripts, and that stylesheet from the page entry:

```ts
// src/pages/index.ts
import 'htmx.org';
import 'basecoat-css/basecoat';
import 'basecoat-css/toast';
import './index.css';
```

Place `HEAD` before the page's assets and render one `Toaster` shell in the
body. An action can return a toast alongside other partial updates:

```rust,ignore
use vixen::ui::basecoatui::{HEAD, Toaster};

const TOASTER: Toaster = Toaster::new();

async fn home(_: HomePath) -> Markup {
    html! {
        head { (HEAD) (vixen::assets!()) }
        body {
            (counter(0))
            button hx-action=(Add::action().by(1)) { "+" }
            (TOASTER.shell())
        }
    }
}

async fn add(Add { by }: Add) -> impl IntoResponse {
    let count = COUNT.fetch_add(by, Ordering::Relaxed) + by;

    partial! {
        counter(count),
        TOASTER.success("Added", format!("Now at {count}.")),
    }
}
```

See [`examples/components`][components] for the complete setup.

## htmx 4 compatibility

vixen targets htmx 4, but `vixen::hx` re-exports
[axum-htmx](https://github.com/robertwayne/axum-htmx) 0.8 until
[v4 support][axum-htmx-v4] ships. vixen's own output already uses htmx 4.

- Morph `SwapOption`s, `HxSource` and `HxRequestType` are unavailable; use raw
  attributes and headers instead.
- `HxTrigger`, `HxTriggerName` and usually `HxPrompt` extract `None`; `HxTarget`
  contains `tag#id` or `tag` rather than a bare id.

## Crates

| crate | what |
|---|---|
| [`axum-vixen`][vixen] | the facade crate, imported as `vixen` — re-exports `maud`, `axum_extra::routing`, `axum_htmx` as `hx`, the bundler's `build` and `Config`, plus the macros |
| [`axum-vixen-macros`][vixen-macros] | [`#[action]`][action], [`#[view_path]`][view_path], [`#[id]`][id], [`#[fragment]`][fragment], [`assets!`][assets], [`asset!`][asset], [`assets_router!`][assets_router] |
| [`axum-vixen-bundler`][vixen-bundler] | the `build.rs` helper behind `vixen::build` and `vixen::Config`; bundles per-page TS and CSS with Bun and copies static files under hashed names |

## Examples

[`examples/counter`][counter] — a counter in one `main.rs` that fits on a
screen, with its `index.ts` and `index.css` beside it. Start here.

[`examples/todos`][todos] — a todo list on one page: every macro once, plus a
per-page `index.ts`.

[`examples/components`][components] — the `basecoatui` toaster, drawer and
dialog, styled with Tailwind.

[`examples/config`][config-example] — demonstrates `Config::base_path`,
`mount!`, and prefixed action, asset, and page links.

```bash
bun install                      # once, for the frontend deps
cargo run -p counter             # http://127.0.0.1:4002/
cargo run -p todos               # http://127.0.0.1:4001/
cargo run -p components          # http://127.0.0.1:4003/
cargo run -p config              # http://127.0.0.1:4004/config/
```

[action]: https://docs.rs/axum-vixen/latest/vixen/attr.action.html
[view_path]: https://docs.rs/axum-vixen/latest/vixen/attr.view_path.html
[id]: https://docs.rs/axum-vixen/latest/vixen/attr.id.html
[fragment]: https://docs.rs/axum-vixen/latest/vixen/attr.fragment.html
[partial]: https://docs.rs/axum-vixen/latest/vixen/macro.partial.html
[assets]: https://docs.rs/axum-vixen/latest/vixen/macro.assets.html
[asset]: https://docs.rs/axum-vixen/latest/vixen/macro.asset.html
[assets_router]: https://docs.rs/axum-vixen/latest/vixen/macro.assets_router.html
[href]: https://docs.rs/axum-vixen/latest/vixen/macro.href.html
[build]: https://docs.rs/axum-vixen/latest/vixen/fn.build.html
[config]: https://docs.rs/axum-vixen/latest/vixen/struct.Config.html
[vixen]: https://docs.rs/axum-vixen/latest/vixen/
[vixen-macros]: https://docs.rs/axum-vixen-macros/latest/vixen_macros/
[vixen-bundler]: https://docs.rs/axum-vixen-bundler/latest/vixen_bundler/
[axum-htmx-v4]: https://github.com/robertwayne/axum-htmx/pull/38
[counter]: https://github.com/borolgs/vixen/tree/main/examples/counter
[todos]: https://github.com/borolgs/vixen/tree/main/examples/todos
[components]: https://github.com/borolgs/vixen/tree/main/examples/components
[config-example]: https://github.com/borolgs/vixen/tree/main/examples/config
