use std::fmt::{self, Display, Write as _};

use axum::{Router, http::Uri, response::Redirect, routing::get};
use maud::{Escaper, Render};

/// Normalizes and validates `VIXEN_BASE_PATH` at compile time.
/// Missing, empty, and `/` values mean no base path; trailing slashes are removed.
pub const fn base_path(raw: Option<&'static str>) -> &'static str {
    let Some(raw) = raw else { return "" };
    let bytes = raw.as_bytes();
    let mut end = bytes.len();
    while end > 0 && bytes[end - 1] == b'/' {
        end -= 1;
    }
    if end == 0 {
        return "";
    }
    if bytes[0] != b'/' {
        panic!("base path must start with `/`, like `/app`");
    }
    raw.split_at(end).0
}

/// Mounts `router` at `base` and redirects bare `base` requests to `base/`.
#[track_caller]
pub fn mount<S: Clone + Send + Sync + 'static>(base: &str, router: Router<S>) -> Router<S> {
    if base.is_empty() {
        return router;
    }
    let root = format!("{base}/");
    let redirect = move |uri: Uri| async move {
        match uri.query() {
            Some(query) => Redirect::permanent(&format!("{root}?{query}")),
            None => Redirect::permanent(&root),
        }
    };
    Router::new().nest(&format!("{base}/"), router).route(base, get(redirect))
}

/// A route path prefixed with the app's base path.
///
/// `Display` supports string contexts such as `Redirect::to`; `Render` supports
/// Maud markup.
#[derive(Clone, Copy, Debug)]
pub struct Href<T>(&'static str, T);

impl<T> Href<T> {
    /// Prefixes `path` with `base`.
    pub fn new(base: &'static str, path: T) -> Self {
        Self(base, path)
    }
}

impl<T: Display> Display for Href<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)?;
        self.1.fmt(f)
    }
}

impl<T: Display> Render for Href<T> {
    fn render_to(&self, buffer: &mut String) {
        let _ = write!(Escaper::new(buffer), "{self}");
    }
}

/// The app's base path, `""` when there is none.
///
/// Set it through [`Config::base_path`](crate::Config) in `build.rs`, or with
/// the `VIXEN_BASE_PATH` build environment variable when the config leaves it
/// empty. Trailing slashes are dropped; anything else must start with `/`.
/// Routes stay unprefixed: [`mount!`](macro@crate::mount) nests them, and
/// [`#[action]`](macro@crate::action), [`assets!`](crate::assets) and
/// [`href!`](macro@crate::href) prefix the URLs they emit.
///
/// ```
/// assert_eq!(vixen::base_path!(), "");
/// ```
#[macro_export]
macro_rules! base_path {
    () => {
        const { $crate::__private::base_path(::core::option_env!("VIXEN_BASE_PATH")) }
    };
}

/// Serves `router` under [`base_path!`]: its `/` becomes `{base}/`, and the
/// bare `{base}` redirects there. Without a base path, `router` is returned
/// as is.
///
/// ```
/// use axum::Router;
///
/// let router: Router = vixen::mount!(Router::new());
/// ```
#[macro_export]
macro_rules! mount {
    ($router:expr) => {
        $crate::__private::mount($crate::base_path!(), $router)
    };
}

/// `path` behind [`base_path!`], for `Redirect::to` and other string
/// contexts. In markup, a [`#[view_path]`](macro@crate::view_path) type already
/// renders as its link.
///
/// ```
/// use vixen::{href, view_path};
///
/// #[view_path("/items/{id}")]
/// struct ItemPath {
///     id: u32,
/// }
///
/// assert_eq!(href!(ItemPath { id: 7 }).to_string(), "/items/7");
/// ```
#[macro_export]
macro_rules! href {
    ($path:expr) => {
        $crate::Href::new($crate::base_path!(), $path)
    };
}
