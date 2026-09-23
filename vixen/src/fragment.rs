use std::marker::PhantomData;

use axum_htmx::SwapOption;
use maud::{Markup, Render};

use crate::{HxPartial, HxPartialResponse, Id, Part, Parts};

/// One element whose root carries `T`'s id; a response swaps it by `outerHTML`
/// so the id survives.
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

impl<T: Id> From<Fragment<T>> for HxPartialResponse {
    fn from(value: Fragment<T>) -> Self {
        HxPartial::new().part(Part::from(value))
    }
}
