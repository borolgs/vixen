use std::marker::PhantomData;

use axum_htmx::SwapOption;
use maud::{Markup, Render};

use crate::{HxPartial, Id, Part, PartialEntry, Parts, Selector, markers::Filled};

/// Markup for one element with a typed id.
///
/// `Fragment<T>` renders as plain markup. In a [`partial!`](crate::partial!)
/// response, it targets its id with an `outerHTML` swap.
/// [`#[fragment]`](macro@crate::fragment) constructs it automatically;
/// [`Fragment::new`] is for markup the attribute can't wrap, such as a method's.
///
/// ```
/// use vixen::{Fragment, id, maud::html, partial};
///
/// #[id]
/// struct TodoId(u32);
///
/// struct Todo {
///     id: u32,
///     title: &'static str,
/// }
///
/// impl Todo {
///     fn view(&self) -> Fragment<TodoId> {
///         let id = TodoId(self.id);
///         Fragment::new(&id, html! { li id=(id) { (self.title) } })
///     }
/// }
///
/// let todo = Todo { id: 7, title: "milk" };
/// assert_eq!(html! { (todo.view()) }.into_string(), r#"<li id="todo-7">milk</li>"#);
/// let todo = Todo { title: "oat milk", ..todo };
/// assert_eq!(
///     partial!(todo.view()).render().into_string(),
///     concat!(
///         r##"<hx-partial hx-target="#todo-7" hx-swap="outerHTML">"##,
///         r#"<li id="todo-7">oat milk</li></hx-partial>"#,
///     )
/// );
/// ```
///
/// A fragment also converts into [`Part`], [`Parts`] or [`Markup`].
pub struct Fragment<T: Id> {
    sel: Selector,
    markup: Markup,
    _id: PhantomData<fn() -> T>,
}

impl<T: Id> Fragment<T> {
    /// Wraps markup whose root element has `id`.
    pub fn new(id: &T, markup: Markup) -> Self {
        Self {
            sel: id.sel(),
            markup,
            _id: PhantomData,
        }
    }
}

impl<T: Id> Render for Fragment<T> {
    fn render_to(&self, buffer: &mut String) {
        buffer.push_str(&self.markup.0);
    }
}

impl<T: Id> From<Fragment<T>> for Part {
    fn from(value: Fragment<T>) -> Self {
        Part::new(value.sel, value.markup).swap(SwapOption::OuterHtml)
    }
}

impl<T: Id> From<Fragment<T>> for Parts {
    fn from(value: Fragment<T>) -> Self {
        Part::from(value).into()
    }
}

impl<T: Id> PartialEntry for Fragment<T> {
    type State<S> = Filled;

    fn add_to<S, M>(self, partial: HxPartial<S, M>) -> HxPartial<Filled, M> {
        partial.part(self)
    }
}

impl<T: Id> From<Fragment<T>> for Markup {
    fn from(value: Fragment<T>) -> Self {
        value.markup
    }
}
