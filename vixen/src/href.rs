use std::fmt::{self, Display, Write as _};

use maud::{Escaper, Render};

/// A route path prefixed with the app's base path.
///
/// `Display` supports string contexts such as `Redirect::to`; `Render` supports
/// Maud markup.
#[derive(Clone, Copy, Debug)]
pub struct Href<T>(&'static str, T);

/// The type of [`asset!`](crate::asset), for storing asset URLs in `const`s.
pub type Asset = Href<&'static str>;

impl<T> Href<T> {
    /// Prefixes `path` with `base`.
    pub const fn new(base: &'static str, path: T) -> Self {
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

/// `path` behind [`base_path!`](crate::base_path!), for `Redirect::to` and
/// other string contexts. In markup, a [`#[view_path]`](macro@crate::view_path)
/// type already renders as its link.
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

#[cfg(test)]
mod tests {
    use maud::html;

    use super::*;

    #[test]
    fn href_displays_and_renders_escaped() {
        let href = Href::new("/app", "/q?a=1&b=2");
        assert_eq!(href.to_string(), "/app/q?a=1&b=2");
        assert_eq!(
            html! { a href=(href) {} }.into_string(),
            r#"<a href="/app/q?a=1&amp;b=2"></a>"#
        );
    }
}
