//! The proc macros behind `vixen`.
//!
//! Use them through the `vixen` crate. Their expansions refer to `::vixen::`,
//! so `axum-vixen-macros` is not a standalone API.

// Docs live here, not on the re-exports, so IDE hover shows them. Intra-doc
// links cannot reach `vixen` from this crate, hence the relative HTML links.

use proc_macro::TokenStream;

mod action;
mod assets;
mod fragment;
mod id;
mod req_ctx;
mod route;

/// Declares an htmx endpoint. One struct is the route, the form extractor, and
/// the source of the `hx-action` value that calls it.
///
/// ```
/// use axum::Router;
/// use vixen::{RouterExt, action, maud::{Markup, html}};
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
/// let router: Router = Router::new().action(rename);
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
///   `axum_extra::extract::Form`, and `TypedPath` for `Name`.
///   [`RouterExt::action`] infers the route from the last handler argument. The
///   macro also adds `Route: POST /path` to the struct's docs.
/// - `Name::action()`, which returns a `NameAction` builder with one setter per
///   field. `String` setters accept `impl Into<String>`; `Option<T>` setters
///   accept `T` and also have a `maybe_<field>(Option<T>)` variant that ignores
///   `None`. Other setters use the declared type, which must implement
///   `Serialize`. Unset fields are omitted from `hx-vals`. The rendered path
///   is prefixed with [`base_path!`].
/// - `NameAction`, which renders as the value of `hx-action`. Call `.hx()` to
///   add `trigger`, `target`, `swap`, or `sync` through [`HxAction`].
/// - `Name::FIELD`, a set of `&'static str` field names for form controls.
/// - `Name::PATH`, `name.path()`, and the generated `NamePath` unit struct.
///
/// # Requirements
///
/// `#[action]` accepts a struct with named fields and no generics.
///
/// `Name` consumes the request body and must be the handler's last argument.
/// Put extractors such as `State` before it.
///
/// Generated code refers to `::axum` and `::axum_extra`, so both must be direct
/// dependencies of the calling crate. vixen enables the required features:
/// `macros` for axum and `form`, `typed-routing` for axum-extra.
///
/// [`RouterExt::action`]: ../vixen/trait.RouterExt.html#tymethod.action
/// [`base_path!`]: ../vixen/macro.base_path.html
/// [`HxAction`]: ../vixen/struct.HxAction.html
#[proc_macro_attribute]
pub fn action(attr: TokenStream, item: TokenStream) -> TokenStream {
    action::expand(attr.into(), item.into()).into()
}

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
/// A one-field tuple struct makes the id dynamic: the field, any `Display`, is
/// appended after a `-`.
///
/// ```
/// use vixen::{id, maud::html, partial};
///
/// #[id]
/// struct TodoId(u32);
///
/// let id = TodoId(7);
/// assert_eq!(html! { li id=(id) {} }.into_string(), r#"<li id="todo-7"></li>"#);
/// assert_eq!(
///     partial!(id => html! { "milk" }).render().into_string(),
///     r##"<hx-partial hx-target="#todo-7">milk</hx-partial>"##
/// );
/// ```
///
/// Use `#[id]` when the page writes the element and a response replaces its
/// contents. Use [`#[fragment]`](../vixen/attr.fragment.html) when one
/// function renders and replaces the whole element.
///
/// # Generated API
///
/// - `Render`, which writes the bare id, escaped.
/// - [`Id`], whose blanket `From<T: Id> for Selector` makes the type usable
///   anywhere a [`Selector`] is accepted: [`partial!`], [`HxPartial::target`],
///   [`HxAction::target`], or [`SyncStrategy::on`].
/// - `slot()`, which renders `<div id="todo-list"></div>` for a later response
///   to fill.
/// - An `HTML id: todo-list` line in the struct's docs, or `todo-{u32}`.
///
/// `slot()` invokes maud's `html!`, so the calling crate must depend on `maud`
/// directly; see [re-exports].
///
/// [`Id`]: ../vixen/trait.Id.html
/// [`Selector`]: ../vixen/struct.Selector.html
/// [`partial!`]: ../vixen/macro.partial.html
/// [`HxPartial::target`]: ../vixen/struct.HxPartial.html#method.target
/// [`HxAction::target`]: ../vixen/struct.HxAction.html#method.target
/// [`SyncStrategy::on`]: ../vixen/enum.SyncStrategy.html#method.on
/// [re-exports]: ../vixen/index.html#re-exports
#[proc_macro_attribute]
pub fn id(attr: TokenStream, item: TokenStream) -> TokenStream {
    id::expand(attr.into(), item.into()).into()
}

/// Declares a page route: `#[derive(TypedPath, Deserialize)]` and
/// `#[typed_path("...")]` in one line.
///
/// ```
/// use axum::Router;
/// use vixen::{RouterExt, maud::{Markup, html}, route};
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
/// let router: Router = Router::new().view(item);
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
///
/// [`base_path!`]: ../vixen/macro.base_path.html
#[proc_macro_attribute]
pub fn route(attr: TokenStream, item: TokenStream) -> TokenStream {
    route::expand(attr.into(), item.into()).into()
}

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
///     .view(home)
///     .merge(vixen::assets_router!());
/// ```
///
/// The generated router is generic over its state and can be merged into any
/// `Router<S>`.
///
/// Like [`assets!`], this macro needs the manifest from [`build`], so the
/// example is ignored.
///
/// [`mount!`]: ../vixen/macro.mount.html
/// [`assets!`]: ../vixen/macro.assets.html
/// [`build`]: ../vixen/fn.build.html
#[proc_macro]
pub fn assets_router(item: TokenStream) -> TokenStream {
    assets::expand_assets_router(item.into()).into()
}

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
///
/// [`build`]: ../vixen/fn.build.html
/// [`base_path!`]: ../vixen/macro.base_path.html
/// [`assets_router!`]: ../vixen/macro.assets_router.html
#[proc_macro]
pub fn assets(item: TokenStream) -> TokenStream {
    assets::expand_assets_head(item.into()).into()
}

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
///
/// [`build`]: ../vixen/fn.build.html
/// [`Config::static_glob`]: ../vixen/struct.Config.html#structfield.static_glob
/// [`assets_router!`]: ../vixen/macro.assets_router.html
/// [`Href`]: ../vixen/struct.Href.html
/// [`base_path!`]: ../vixen/macro.base_path.html
#[proc_macro]
pub fn asset(item: TokenStream) -> TokenStream {
    assets::expand_asset(item.into()).into()
}

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
/// `Self` is `CounterId`.
///
/// [`Fragment`] converts into [`Part`], [`Parts`] and `Markup`, and can be used
/// with `html!`, [`partial!`], [`HxPartial::part`] or [`HxPartial::main`].
///
/// The attribute supports sync or async free functions returning `Markup`. It
/// does not support methods, where `Self` already has a meaning.
/// `#[fragment("sidebar")]` overrides the id, with the rules of
/// [`#[id("sidebar")]`](../vixen/attr.id.html).
///
/// `#[fragment(CartId)]` uses an existing unit `#[id]` struct.
/// `#[fragment(TodoId(id))]` takes a dynamic one: the argument can use the
/// params, runs before the body, and `Self` is its value. Neither form
/// generates `slot()`; the struct has its own.
///
/// ```
/// use vixen::{fragment, id, maud::{Markup, html}};
///
/// #[id]
/// struct TodoId(u64);
///
/// #[fragment(TodoId(id))]
/// fn todo(id: u64, title: &str) -> Markup {
///     html! { li id=(Self) { (title) } }
/// }
///
/// assert_eq!(html! { (todo(7, "milk")) }.into_string(), r#"<li id="todo-7">milk</li>"#);
/// ```
///
/// The element must use `id=(Self)` at its root. The macro rejects a body with
/// no `Self`, but cannot verify that it occurs in the root or that the markup
/// has exactly one root element.
///
/// [`Fragment`]: ../vixen/struct.Fragment.html
/// [`Part`]: ../vixen/struct.Part.html
/// [`Parts`]: ../vixen/struct.Parts.html
/// [`partial!`]: ../vixen/macro.partial.html
/// [`HxPartial::part`]: ../vixen/struct.HxPartial.html#method.part
/// [`HxPartial::main`]: ../vixen/struct.HxPartial.html#method.main
#[proc_macro_attribute]
pub fn fragment(attr: TokenStream, item: TokenStream) -> TokenStream {
    fragment::expand(attr.into(), item.into()).into()
}

/// Adds request-local access to an axum extractor.
///
/// `Ctx::middleware` extracts the context once per request. Downstream code
/// reads it with `Ctx::current()`. Tests can set it directly with
/// `ctx.scope(future)`.
///
/// ```
/// use axum::{
///     Router,
///     extract::{FromRef, FromRequestParts, State},
///     middleware,
///     routing::get,
/// };
/// use vixen::ReqCtx;
///
/// #[derive(Clone, FromRef)]
/// struct AppState {
///     shop_name: String,
/// }
///
/// #[derive(Clone, FromRequestParts, ReqCtx)]
/// #[from_request(state(AppState))]
/// struct Ctx {
///     #[from_request(via(State))]
///     shop_name: String,
/// }
///
/// async fn shop_name() -> String {
///     Ctx::current().shop_name
/// }
///
/// # #[tokio::main(flavor = "current_thread")]
/// # async fn main() {
/// let state = AppState { shop_name: "Vixen Goods".into() };
/// let app: Router = Router::new()
///     .route("/", get(shop_name))
///     .layer(middleware::from_fn_with_state(state.clone(), Ctx::middleware))
///     .with_state(state);
///
/// let name = Ctx { shop_name: "Vixen Goods".into() }
///     .scope(shop_name())
///     .await;
/// assert_eq!(name, "Vixen Goods");
/// # }
/// ```
///
/// The context must implement `Clone` and `FromRequestParts` and cannot be
/// generic. Spawned Tokio tasks do not inherit it; move a context into them
/// with `tokio::spawn(Ctx::current().scope(future))`.
///
/// The expanded code refers to `::axum`, which must be a direct dependency.
#[proc_macro_derive(ReqCtx)]
pub fn req_ctx(item: TokenStream) -> TokenStream {
    req_ctx::expand(item.into()).into()
}
