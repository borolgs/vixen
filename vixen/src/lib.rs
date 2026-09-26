#![doc = include_str!("../README.md")]
//!
//! ## Quick reference
//!
//! | To | Use |
//! |---|---|
//! | define a page route | [`#[route]`](macro@route) |
//! | define an htmx endpoint and call it from markup | [`#[action]`](macro@action), [`HxAction`], [`SyncStrategy`] |
//! | share an element ID between a page and its responses | [`#[id]`](macro@id), [`Selector`] |
//! | render and replace an element by its typed ID | [`#[fragment]`](macro@fragment), [`Fragment`] |
//! | return a main swap and targeted parts | [`partial!`], [`HxPartial`], [`Part`], [`Parts`] |
//! | bundle and serve page-local TS and CSS | [`assets!`], [`assets_router!`], and [`build`] in `build.rs` |
//! | resolve a static file to its content-hashed URL | [`asset!`] |
//! | serve the app under a base path | [`Config::base_path`], [`mount!`], [`href!`], [`base_path!`] |
//! | show a Basecoat toast from a handler | [`ui`], behind the `basecoatui` feature |
//!
//! ## Re-exports
//!
//! vixen re-exports the view-layer crates it builds on so the app can use the
//! same versions:
//!
//! - [`maud`] 0.27, with its `axum` feature, provides `html!` and `Markup`.
//!   Because `html!` expands to `extern crate maud;`, any crate that invokes
//!   it must also depend on `maud` directly.
//! - [`routing`] re-exports `axum_extra::routing` 0.12. It provides
//!   `TypedPath`, derived by [`#[route]`](macro@route) and
//!   [`#[action]`](macro@action), and `RouterExt`, whose `typed_get` and
//!   `typed_post` methods register their handlers.
//! - `hx` re-exports `axum_htmx` 0.8 header types and `SwapOption`; see [htmx 4
//!   compatibility](#htmx-4-compatibility) for caveats.
//!
//! [`build`] and [`Config`] come from `axum-vixen-bundler`, the `build.rs`
//! half. To call them, list `axum-vixen` under `[build-dependencies]` as well.

#![warn(missing_docs)]

// Lets unit tests use the macros, which emit `::vixen::` paths.
#[cfg(test)]
extern crate self as vixen;

// Public for paths emitted by `assets_router!`.
#[doc(hidden)]
pub mod assets;

mod action;
mod base_path;
mod fragment;
mod href;
mod id;
mod partial;
pub mod ui;

// Keep their docs above. Depending on whether rustdoc inlines a re-export,
// docs here are either hidden or appended to the original item's docs.
pub use axum_extra::routing;
pub use axum_htmx as hx;
pub use fragment::Fragment;
pub use href::{Asset, Href};
pub use id::Id;
pub use maud;
pub use vixen_bundler::{Config, build};

pub use partial::{HxPartial, HxPartialResponse, Part, Parts, Selector};

pub mod markers {
    //! Type-state markers used by [`HxPartial`](crate::HxPartial).
    //!
    //! `S` tracks whether the response has any content; `M` tracks whether it
    //! has a main body. These states keep empty responses and duplicate main
    //! bodies from compiling. Users do not construct the markers themselves.

    pub use crate::partial::{Empty, Filled, HasMain, NoMain};
}

pub use action::{HxAction, HxSync, SyncStrategy};

/// Declares an htmx endpoint. One struct is the route, the form extractor, and
/// the source of the `hx-action` value that calls it.
///
/// ```
/// use axum::Router;
/// use vixen::{action, maud::{Markup, html}, routing::RouterExt};
///
/// #[action("/todos/rename")]
/// struct RenameTodo {
///     id: u32,
///     title: String,
/// }
///
/// // RenameTodo is both the route and the form extractor.
/// async fn rename(RenameTodo { id, title }: RenameTodo) -> Markup {
///     html! { li id={ "todo-" (id) } { (title) } }
/// }
///
/// let router: Router = Router::new().typed_post(rename);
///
/// // `id` is sent through `hx-vals`; `title` comes from the input.
/// fn rename_form(id: u32) -> Markup {
///     html! {
///         form hx-action=(RenameTodo::action().id(id)) {
///             input name=(RenameTodo::FIELD.title);
///         }
///     }
/// }
///
/// assert_eq!(
///     rename_form(7).into_string(),
///     concat!(
///         r#"<form hx-action="/todos/rename" hx-vals="{&quot;id&quot;:7}" hx-method="post">"#,
///         r#"<input name="title"></form>"#,
///     )
/// );
/// ```
///
/// # Generated API
///
/// For `#[action("/path")] struct Name { .. }`, the macro generates:
///
/// - `Deserialize`, `Serialize`, axum's `FromRequest` through
///   `axum_extra::extract::Form`, and `TypedPath` for `Name`. This lets
///   `typed_post` infer the route from the handler's argument. The macro also
///   adds `Route: POST /path` to the struct's docs.
/// - `Name::action()`, which returns a `NameAction` builder with one setter per
///   field. `String` setters accept `impl Into<String>`, `Option<T>` setters
///   accept `T`, and other setters use the declared type, which must implement
///   `Serialize`. Fields without a value are omitted from `hx-vals`. The
///   rendered path is prefixed with [`base_path!`].
/// - `NameAction`, which renders as the value of `hx-action`. Call `.hx()` to
///   add `trigger`, `target`, `swap`, or `sync` through [`HxAction`].
/// - `Name::FIELD`, a set of `&'static str` field names for form controls.
/// - `Name::PATH`, `name.path()`, and the generated `NamePath` unit struct.
///
/// # Requirements
///
/// `#[action]` accepts a struct with named fields and no generics.
///
/// `typed_post` requires its `TypedPath` argument first, while axum requires a
/// body-consuming form extractor last. Since `Name` is both, it must be the
/// handler's only extractor.
///
/// Generated code refers to `::axum` and `::axum_extra`, so both must be direct
/// dependencies of the calling crate. vixen enables the required features:
/// `macros` for axum and `form`, `typed-routing` for axum-extra.
pub use vixen_macros::action;

/// Declares a page route: `#[derive(TypedPath, Deserialize)]` and
/// `#[typed_path("...")]` in one line.
///
/// ```
/// use axum::Router;
/// use vixen::{maud::{Markup, html}, route, routing::RouterExt};
///
/// #[route("/items/{id}")]
/// struct ItemPath {
///     id: u32,
/// }
///
/// async fn item(ItemPath { id }: ItemPath) -> Markup {
///     html! { a href=(ItemPath { id: id + 1 }) { "Next" } }
/// }
///
/// let router: Router = Router::new().typed_get(item);
///
/// assert_eq!(ItemPath { id: 7 }.to_string(), "/items/7");
/// assert_eq!(
///     html! { a href=(ItemPath { id: 7 }) {} }.into_string(),
///     r#"<a href="/items/7"></a>"#
/// );
/// ```
///
/// The macro adds those derives, the `typed_path` attribute, a
/// `Route: /items/{id}` line to the struct's docs, and a `Render` impl: in
/// markup the path is its link, prefixed with [`base_path!`], while
/// `to_string()` and `to_uri()` stay the route. Use the raw derives if you need
/// custom rejections, serde attributes, or generics.
///
/// Generated code refers to `::axum` and `::axum_extra`, so both must be direct
/// dependencies of the calling crate.
pub use vixen_macros::route;

/// Declares a typed element ID shared by the page and partial responses.
///
/// ```
/// use vixen::{id, maud::html, partial};
///
/// #[id]
/// struct TodoListId;
///
/// // The page uses it as the `id` attribute.
/// let page = html! { ul id=(TodoListId) {} };
/// assert_eq!(page.into_string(), r#"<ul id="todo-list"></ul>"#);
///
/// // A partial response uses the same type as its target.
/// let response = partial!(TodoListId => html! { li { "milk" } });
/// assert_eq!(
///     response.render().into_string(),
///     r##"<hx-partial hx-target="#todo-list"><li>milk</li></hx-partial>"##
/// );
/// ```
///
/// By default, the id is the struct name in kebab-case with a trailing `-id`
/// removed: `TodoListId` becomes `todo-list`, and `CartBadge` becomes
/// `cart-badge`. `#[id("sidebar")]` overrides it; the value must be nonempty and
/// contain no whitespace or `#`.
///
/// Use `#[id]` when the page writes the element and a response replaces its
/// contents. Use [`#[fragment]`](macro@fragment) when one function renders and
/// replaces the whole element.
///
/// # Generated API
///
/// - `Render`, which writes the bare id.
/// - `ID` and `SEL`, here `"todo-list"` and `"#todo-list"`.
/// - [`Id`], whose blanket `From<T: Id> for Selector` makes the type usable
///   anywhere a [`Selector`] is accepted: [`partial!`], [`HxPartial::target`],
///   [`HxAction::target`], or [`SyncStrategy::on`].
/// - `slot()`, which renders `<div id="todo-list"></div>` for a later response
///   to fill.
/// - An `HTML id: todo-list` line in the struct's docs.
///
/// `slot()` invokes maud's `html!`, so the calling crate must depend on `maud`
/// directly; see [re-exports](crate#re-exports).
pub use vixen_macros::id;

/// The current page's `<script>` and `<link>` tags, resolved at compile time.
///
/// By default, [`build`] builds every entry matching
/// `src/pages/**/{page,index}.ts`. `assets!()` looks for one next to the source
/// file where it is invoked. It expands to `Markup` with a
/// `<script type="module">` tag and, when that entry imports CSS, a
/// `<link rel="stylesheet">` tag, their URLs prefixed with [`base_path!`]. If
/// no entry matches, it expands to empty markup.
///
/// ```ignore
/// // src/pages/todos/mod.rs, next to src/pages/todos/index.ts
/// html! {
///     head {
///         title { "Todos" }
///         (vixen::assets!())
///     }
/// }
/// ```
///
/// [`assets_router!`] serves the content-hashed URLs.
///
/// [`build`] generates the manifest used by this macro, so it must run from
/// `build.rs`. The example is ignored because it has no build script; see
/// `examples/counter` for a checked version.
pub use vixen_macros::assets;

/// Builds a `Router` that serves bundled assets from the binary.
///
/// `include_bytes!` embeds every file emitted by the bundler. The router serves
/// them under `assets_prefix` (`/assets/` by default, `{base}/assets/` inside
/// [`mount!`]), with the correct MIME type and
/// `Cache-Control: public, max-age=31536000, immutable`. Content hashes in the
/// file names make the immutable cache policy safe. Unknown paths under the
/// prefix return 404.
///
/// ```ignore
/// let router = Router::new()
///     .typed_get(home)
///     .merge(vixen::assets_router!());
/// ```
///
/// The generated router is generic over its state and can be merged into any
/// `Router<S>`.
///
/// Like [`assets!`], this macro needs the manifest from [`build`], so the
/// example is ignored.
pub use vixen_macros::assets_router;

/// Resolves a static file to its content-hashed URL at compile time.
///
/// Paths are relative to the calling source file, as with `include_bytes!`.
/// [`build`] copies files matching [`Config::static_glob`] under hashed names;
/// the default glob covers images in `src/**/assets/` directories.
/// [`assets_router!`] serves the copies.
///
/// The macro expands to an [`Href`], so [`base_path!`] is applied and the result
/// can be rendered in markup or used through `Display`.
///
/// ```ignore
/// // src/pages/game/mod.rs, next to src/pages/game/assets/map.jpg
/// html! { img src=(vixen::asset!("./assets/map.jpg")); }
/// // <img src="/assets/map-1a2b3c4d.jpg">
/// ```
///
/// Compilation fails if the file does not exist or is outside the configured
/// glob. This example is ignored because it needs the manifest from [`build`].
pub use vixen_macros::asset;

/// Turns a function that renders one element into a typed [`Fragment`]. In
/// page markup the fragment renders normally; in a response it becomes an
/// `outerHTML` [`Part`] targeting the same element.
///
/// ```
/// use vixen::{fragment, maud::{Markup, html}, partial};
///
/// #[fragment]
/// fn counter(count: i64) -> Markup {
///     html! { output id=(Self) { (count) } }
/// }
///
/// let page = html! { (counter(1)) };
/// assert_eq!(page.into_string(), r#"<output id="counter">1</output>"#);
///
/// let response = partial!(counter(2));
/// assert_eq!(
///     response.render().into_string(),
///     r##"<hx-partial hx-target="#counter" hx-swap="outerHTML"><output id="counter">2</output></hx-partial>"##
/// );
///
/// assert_eq!(
///     counter::slot().into_string(),
///     r#"<div id="counter" style="display: none;"></div>"#
/// );
/// ```
///
/// For `counter`, the macro declares `#[id] struct CounterId`, changes the
/// return type to `Fragment<CounterId>`, and exposes `counter::slot()`. The
/// generated items have the same visibility as the function. Inside the body,
/// `Self` is `CounterId`, so `Self::SEL` works too.
///
/// [`Fragment`] converts into [`Part`], [`Parts`] and [`Markup`](maud::Markup),
/// and can be used with `html!`, [`partial!`], [`HxPartial::part`] or
/// [`HxPartial::main`].
///
/// The attribute supports sync or async free functions returning `Markup`. It
/// does not support methods, where `Self` already has a meaning.
/// `#[fragment("sidebar")]` overrides the id, with the rules of
/// [`#[id("sidebar")]`](macro@id).
///
/// The element must use `id=(Self)` at its root. The macro rejects a body with
/// no `Self`, but cannot verify that it occurs in the root or that the markup
/// has exactly one root element.
pub use vixen_macros::fragment;

#[doc(hidden)]
pub mod __private {
    pub use {serde, serde_json};

    pub use crate::base_path::{base_path, mount};
}
