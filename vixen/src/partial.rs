use std::marker::PhantomData;

use axum::{http::HeaderValue, response::IntoResponse};
use axum_htmx::SwapOption;
use maud::{Markup, Render, html};

use crate::Id;

/// An htmx response with an optional main body and any number of targeted
/// `<hx-partial>` blocks.
///
/// Most code uses [`partial!`](crate::partial!); `HxPartial` is the builder
/// behind it.
///
/// ```
/// use vixen::{HxPartial, hx::SwapOption, id, maud::html};
///
/// #[id]
/// struct NotesId;
///
/// let response = HxPartial::new()
///     .main(html! { "swapped into the request target" }) // optional
///     .target(NotesId, html! { li { "a note" } })
///     .target_swap("#toaster", SwapOption::BeforeEnd, html! { "saved" });
///
/// assert_eq!(
///     response.render().into_string(),
///     concat!(
///         "swapped into the request target",
///         r##"<hx-partial hx-target="#notes"><li>a note</li></hx-partial>"##,
///         r##"<hx-partial hx-target="#toaster" hx-swap="beforeend">saved</hx-partial>"##,
///     )
/// );
/// ```
///
/// # Type state
///
/// `S` tracks whether the response has any content, and `M` tracks whether its
/// main body is set. The marker types are exported from
/// [`markers`](crate::markers). Only `HxPartial<Filled, M>` implements
/// `IntoResponse`, and [`main`](HxPartial::main) is available only while `M` is
/// [`NoMain`]. Empty responses and a second main body therefore fail to compile.
///
/// Use [`HxPartialResponse`] when a handler needs a concrete return type for a
/// response with targeted parts only.
pub struct HxPartial<S = Empty, M = NoMain> {
    state: State,
    _state: PhantomData<fn() -> (S, M)>,
}

/// A concrete handler return type for one or more targeted parts and no main
/// body.
///
/// A [`Part`] converts directly into this type, so a branch can end in
/// `.into()`:
///
/// ```
/// use vixen::{HxPartialResponse, Part, maud::html};
///
/// fn saved() -> HxPartialResponse {
///     Part::new("#status", html! { "saved" }).into()
/// }
/// ```
///
/// The concrete type for a response with a main body is
/// `HxPartial<Filled, HasMain>`; it is usually returned as `impl IntoResponse`.
pub type HxPartialResponse = HxPartial<Filled>;

/// Type state before any content has been added.
pub struct Empty;
/// Type state after at least one piece of content has been added.
pub struct Filled;

/// Type state before the main body has been set.
pub struct NoMain;
/// Type state after the main body has been set.
pub struct HasMain;

struct State {
    main: Option<Markup>,
    parts: Vec<Part>,
}

/// Content for an htmx target, with an optional swap strategy.
pub struct Part {
    target: Selector,
    swap: Option<SwapOption>,
    content: Markup,
}

impl Part {
    /// Creates a part for `target`.
    pub fn new(target: impl Into<Selector>, content: Markup) -> Self {
        Part {
            target: target.into(),
            swap: None,
            content,
        }
    }

    /// Sets the swap strategy.
    pub fn swap(mut self, swap: SwapOption) -> Self {
        self.swap = Some(swap);
        self
    }

    /// Returns the content without its `<hx-partial>` wrapper, for rendering
    /// the same fragment as part of a full page.
    pub fn render(self) -> Markup {
        self.content
    }
}

impl Render for Part {
    fn render_to(&self, buffer: &mut String) {
        buffer.push_str(&self.content.0);
    }
}

impl IntoResponse for Part {
    fn into_response(self) -> axum::response::Response {
        HxPartial::from(self).into_response()
    }
}

/// A collection of [`Part`]s.
pub struct Parts {
    /// Parts in response order.
    pub parts: Vec<Part>,
}

impl From<Part> for Parts {
    fn from(part: Part) -> Self {
        Self { parts: vec![part] }
    }
}

impl From<Part> for HxPartial<Filled> {
    fn from(part: Part) -> Self {
        HxPartial::new().part(part)
    }
}

impl From<Vec<Part>> for Parts {
    fn from(parts: Vec<Part>) -> Self {
        Self { parts }
    }
}

/// A CSS selector used by `hx-target` and `hx-sync`.
///
/// APIs that target an element accept `impl Into<Selector>`. Strings are kept
/// as written (`"#list"`, `"closest form"`); an [`#[id]`](macro@crate::id) type
/// converts to its `#id` selector.
#[derive(Clone)]
pub struct Selector(pub String);

impl From<Selector> for String {
    fn from(value: Selector) -> Self {
        value.0
    }
}

impl<T: Id> From<T> for Selector {
    fn from(_: T) -> Self {
        Self(T::SEL.to_string())
    }
}

impl From<&str> for Selector {
    fn from(value: &str) -> Self {
        Self(value.to_string())
    }
}

impl From<String> for Selector {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl Render for Selector {
    fn render(&self) -> Markup {
        maud::PreEscaped(self.0.clone())
    }
}

impl<S, M> HxPartial<S, M> {
    fn cast<S2, M2>(self) -> HxPartial<S2, M2> {
        HxPartial {
            state: self.state,
            _state: PhantomData,
        }
    }
}

impl HxPartial<Empty, NoMain> {
    /// Creates an empty builder. Add a main body or part before returning it.
    pub fn new() -> Self {
        HxPartial {
            state: State {
                main: None,
                parts: Vec::new(),
            },
            _state: PhantomData,
        }
    }
}

impl Default for HxPartial<Empty, NoMain> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S> HxPartial<S, NoMain> {
    /// Sets the markup for the request's original target. It is rendered
    /// outside the `<hx-partial>` blocks.
    ///
    /// Omit the main body to leave the requesting element untouched. This
    /// method can only be called once.
    pub fn main(mut self, content: Markup) -> HxPartial<Filled, HasMain> {
        self.state.main = Some(content);
        self.cast()
    }
}

impl<S, M> HxPartial<S, M> {
    /// One `<hx-partial>` block, aimed at `target`.
    ///
    /// ```html
    /// <hx-partial hx-target="#notes">...</hx-partial>
    /// ```
    pub fn target(self, target: impl Into<Selector>, content: Markup) -> HxPartial<Filled, M> {
        self.part(Part::new(target, content))
    }

    /// One `<hx-partial>` block, aimed at `target` with the swap spelled out.
    ///
    /// ```html
    /// <hx-partial hx-target="#toast" hx-swap="beforeend">...</hx-partial>
    /// ```
    pub fn target_swap(
        self,
        target: impl Into<Selector>,
        swap: SwapOption,
        content: Markup,
    ) -> HxPartial<Filled, M> {
        self.part(Part::new(target, content).swap(swap))
    }

    /// Adds a preconfigured [`Part`].
    pub fn part(mut self, part: Part) -> HxPartial<Filled, M> {
        self.state.parts.push(part);
        self.cast()
    }

    /// Adds a group of response parts.
    pub fn parts(mut self, parts: impl Into<Parts>) -> HxPartial<Filled, M> {
        self.state.parts.extend(parts.into().parts);
        self.cast()
    }
}

impl<M> HxPartial<Filled, M> {
    /// Renders the main body first, followed by one `<hx-partial>` block per
    /// part in insertion order.
    pub fn render(self) -> Markup {
        html! {
            @if let Some(main) = self.state.main {
                (main)
            }
            @for Part { target, swap, content } in self.state.parts {
                hx-partial hx-target=(target) hx-swap=[swap.map(swap_attr)] {
                    (content)
                }
            }
        }
    }
}

impl<M> IntoResponse for HxPartial<Filled, M> {
    fn into_response(self) -> axum::response::Response {
        self.render().into_response()
    }
}

pub(crate) fn swap_attr(swap: SwapOption) -> String {
    HeaderValue::from(swap)
        .to_str()
        .unwrap_or_default()
        .to_owned()
}

/// Builds an [`HxPartial`] response.
///
/// Each entry can be:
///
/// - `_ => content` for the main swap;
/// - `target => content` for a targeted fragment;
/// - a [`Part`] or anything convertible to [`Parts`].
///
/// The main swap, if present, must come first.
///
/// ```
/// use vixen::{Part, hx::SwapOption, id, maud::{Markup, html}, partial};
///
/// #[id]
/// struct ReviewListId;
///
/// fn reviews_section() -> Markup {
///     html! { li { "Sturdy." } }
/// }
///
/// fn toast(text: &str) -> Part {
///     Part::new("#toaster", html! { (text) }).swap(SwapOption::BeforeEnd)
/// }
///
/// fn shelf_updates() -> Vec<Part> {
///     vec![Part::new("#shelf-count", html! { "12" })]
/// }
///
/// let response = partial!(
///     _ => html! { div.alert { "Thanks!" } },
///     ReviewListId => reviews_section(),
///     toast("Saved"),
///     shelf_updates(),
/// );
/// ```
#[macro_export]
macro_rules! partial {
    (@acc $p:expr;) => {
        $p
    };
    (@acc $p:expr; $target:expr => $content:expr $(, $($rest:tt)*)?) => {
        $crate::partial!(@acc $p.target($target, $content); $($($rest)*)?)
    };
    (@acc $p:expr; $parts:expr $(, $($rest:tt)*)?) => {
        $crate::partial!(@acc $p.parts($parts); $($($rest)*)?)
    };
    (_ => $main:expr $(, $($rest:tt)*)?) => {
        $crate::partial!(@acc $crate::HxPartial::new().main($main); $($($rest)*)?)
    };
    ($($entry:tt)+) => {
        $crate::partial!(@acc $crate::HxPartial::new(); $($entry)+)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    fn toast() -> Part {
        Part::new("#toaster", html! { "saved" }).swap(SwapOption::BeforeEnd)
    }

    fn rows() -> Markup {
        html! { tr { td { "row" } } }
    }

    fn both() -> Parts {
        vec![Part::new("#rows", rows()), toast()].into()
    }

    #[test]
    fn a_bare_part_and_a_targeted_entry_render_in_order() {
        let html = partial!(toast(), "#rows" => rows()).render().into_string();
        assert_eq!(
            html,
            concat!(
                r##"<hx-partial hx-target="#toaster" hx-swap="beforeend">saved</hx-partial>"##,
                r##"<hx-partial hx-target="#rows"><tr><td>row</td></tr></hx-partial>"##,
            )
        );
    }

    #[test]
    fn parts_spread_in_place() {
        let html = partial!(toast(), both()).render().into_string();
        assert_eq!(
            html,
            concat!(
                r##"<hx-partial hx-target="#toaster" hx-swap="beforeend">saved</hx-partial>"##,
                r##"<hx-partial hx-target="#rows"><tr><td>row</td></tr></hx-partial>"##,
                r##"<hx-partial hx-target="#toaster" hx-swap="beforeend">saved</hx-partial>"##,
            )
        );
    }
}
