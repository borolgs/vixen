//! Widgets for [Basecoat](https://basecoatui.com), behind the `basecoatui`
//! feature.
//!
//! Basecoat itself is not bundled. The page's entry imports its CSS,
//! `basecoat-css/basecoat` and `basecoat-css/toast`; the scripts initialize
//! toasts as htmx appends them. Render [`HEAD`] in `<head>` before the page's
//! own assets, and one [`Toaster`] shell in the body:
//!
//! ```
//! use vixen::{HxPartial, maud::html, ui::basecoatui::{HEAD, Toaster}};
//!
//! const TOASTER: Toaster = Toaster::new();
//!
//! let page = html! {
//!     head { (HEAD) }
//!     body { (TOASTER.shell()) }
//! };
//!
//! // A handler appends a toast as one part of its response.
//! let response = HxPartial::new().part(TOASTER.success("Saved", "Milk is on the list."));
//! assert!(response.render().into_string().starts_with(
//!     r##"<hx-partial hx-target="#toaster" hx-swap="beforeend"><div class="toast" role="status""##
//! ));
//! ```
//!
//! `examples/components` has the full wiring.

mod head;
mod toast;

pub use head::HEAD;
pub use toast::{Action, Align, Category, Duration, Toast, Toaster};
