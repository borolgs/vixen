#![doc = include_str!("../README.md")]
//!
//! ## Quick reference
//!
//! | To | Use |
//! |---|---|
//! | define a page route | [`#[view_path]`](macro@view_path) |
//! | define an htmx endpoint and call it from markup | [`#[action]`](macro@action), [`HxAction`], [`SyncStrategy`] |
//! | share an element ID between a page and its responses | [`#[id]`](macro@id), [`Selector`] |
//! | render one element that owns its id | [`#[fragment]`](macro@fragment), [`Fragment`] |
//! | return a main swap and targeted fragments | [`partial!`], [`HxPartial`], [`Part`], [`Parts`] |
//! | bundle and serve page-local TS and CSS | [`assets!`], [`assets_router!`], and [`bundler::build`] in `build.rs` |
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
//!   `TypedPath`, derived by [`#[view_path]`](macro@view_path) and
//!   [`#[action]`](macro@action), and `RouterExt`, whose `typed_get` and
//!   `typed_post` methods register their handlers.
//! - `hx` re-exports `axum_htmx` 0.8 header types and `SwapOption`; see [htmx 4
//!   compatibility](#htmx-4-compatibility) for caveats.
//!
//! [`bundler`] re-exports `axum-vixen-bundler`, the `build.rs` half. To call it,
//! list `axum-vixen` under `[build-dependencies]` as well.

#![warn(missing_docs)]

// Lets unit tests use the macros, which emit `::vixen::` paths.
#[cfg(test)]
extern crate self as vixen;

// Public for paths emitted by `assets_router!`.
#[doc(hidden)]
pub mod assets;

mod action;
mod fragment;
mod id;
mod partial;

// Keep their docs above. Depending on whether rustdoc inlines a re-export,
// docs here are either hidden or appended to the original item's docs.
pub use axum_extra::routing;
pub use axum_htmx as hx;
pub use fragment::Fragment;
pub use id::Id;
pub use maud;
pub use vixen_bundler as bundler;

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
///   `Serialize`. Fields without a value are omitted from `hx-vals`.
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
/// use vixen::{maud::{Markup, html}, routing::RouterExt, view_path};
///
/// #[view_path("/items/{id}")]
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
/// ```
///
/// The macro adds only those derives, the `typed_path` attribute, and a
/// `Route: /items/{id}` line to the struct's docs. Use the raw derives if you
/// need custom rejections, serde attributes, or generics.
///
/// Generated code refers to `::axum` and `::axum_extra`, so both must be direct
/// dependencies of the calling crate.
pub use vixen_macros::view_path;

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
/// By default, [`bundler::build`] builds every entry matching
/// `src/pages/**/{page,index}.ts`. `assets!()` looks for one next to the source
/// file where it is invoked. It expands to `PreEscaped<&'static str>`
/// containing a `<script type="module">` tag and, when that entry imports CSS,
/// a `<link rel="stylesheet">` tag. If no entry matches, it expands to an empty
/// string.
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
/// [`bundler::build`] generates the manifest used by this macro, so it
/// must run from `build.rs`. The example is ignored because it has no build
/// script; see `examples/counter` for a checked version.
pub use vixen_macros::assets;

/// Builds a `Router` that serves bundled assets from the binary.
///
/// `include_bytes!` embeds every file emitted by the bundler. The router serves
/// them under `assets_prefix` (`/assets/` by default), with the correct MIME
/// type and `Cache-Control: public, max-age=31536000, immutable`. Content hashes
/// in the file names make the immutable cache policy safe. Unknown paths under
/// the prefix return 404.
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
/// Like [`assets!`], this macro needs the manifest from
/// [`bundler::build`], so the example is ignored.
pub use vixen_macros::assets_router;

/// Turns a free `fn(..) -> Markup` whose root element has `id=(Self)` into a
/// [`Fragment`]: emits `#[id] struct <Fn>Id;`, makes `Self` that type inside
/// the body, and returns `Fragment<<Fn>Id>`.
pub use vixen_macros::fragment;

#[doc(hidden)]
pub mod __private {
    pub use {serde, serde_json};
}
