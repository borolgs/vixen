use std::marker::PhantomData;

use axum_htmx::SwapOption;
use maud::{Markup, Render};

use crate::{Id, Part, Parts};

/// Markup for one element with a typed id.
///
/// `Fragment<T>` renders as plain markup. In a [`partial!`](crate::partial!)
/// response, it targets `T::SEL` with an `outerHTML` swap.
/// [`#[fragment]`](macro@crate::fragment) constructs it automatically;
/// [`Fragment::new`] is available when the id type is declared separately.
///
/// ```
/// use vixen::{Fragment, id, maud::html, partial};
///
/// #[id]
/// struct NotesId;
///
/// fn notes(n: usize) -> Fragment<NotesId> {
///     Fragment::new(html! { p id=(NotesId) { (n) " notes" } })
/// }
///
/// assert_eq!(html! { (notes(2)) }.into_string(), r#"<p id="notes">2 notes</p>"#);
/// assert_eq!(
///     partial!(notes(3)).render().into_string(),
///     concat!(
///         r##"<hx-partial hx-target="#notes" hx-swap="outerHTML">"##,
///         r#"<p id="notes">3 notes</p></hx-partial>"#,
///     )
/// );
/// ```
///
/// A fragment also converts into [`Part`], [`Parts`] or [`Markup`].
pub struct Fragment<T: Id>(Markup, PhantomData<fn() -> T>);

impl<T: Id> Fragment<T> {
    /// Wraps markup whose root element has `T`'s id.
    pub fn new(markup: Markup) -> Self {
        Self(markup, PhantomData)
    }
}

impl<T: Id> Render for Fragment<T> {
    fn render_to(&self, buffer: &mut String) {
        buffer.push_str(&self.0.0);
    }
}

impl<T: Id> From<Fragment<T>> for Part {
    fn from(value: Fragment<T>) -> Self {
        Part::new(T::SEL, value.0).swap(SwapOption::OuterHtml)
    }
}

impl<T: Id> From<Fragment<T>> for Parts {
    fn from(value: Fragment<T>) -> Self {
        Part::from(value).into()
    }
}

impl<T: Id> From<Fragment<T>> for Markup {
    fn from(value: Fragment<T>) -> Self {
        value.0
    }
}
