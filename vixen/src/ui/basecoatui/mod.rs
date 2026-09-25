//! Widgets for [Basecoat](https://basecoatui.com), behind the `basecoatui`
//! feature.
//!
//! Basecoat itself is not bundled. Import its CSS from the page stylesheet and
//! `basecoat-css/basecoat` from the page entry. Widgets with their own scripts
//! need those too: `basecoat-css/toast` for [`Toaster`] and
//! `basecoat-css/drawer` for [`Drawer`]. [`Dialog`] needs no script.
//!
//! Render [`HEAD`] in `<head>` before the page's assets, and render each
//! widget's shell once in the body:
//!
//! ```
//! use vixen::{HxPartial, maud::html, ui::basecoatui::{Drawer, HEAD, Toaster}};
//!
//! const TOASTER: Toaster = Toaster::new();
//! const DRAWER: Drawer = Drawer::new("profile");
//!
//! let page = html! {
//!     head { (HEAD) }
//!     body { (TOASTER.shell()) (DRAWER.shell()) }
//! };
//!
//! // A handler appends a toast as one part of its response.
//! let response = HxPartial::new().part(TOASTER.success("Saved", "Milk is on the list."));
//! assert!(response.render().into_string().starts_with(
//!     r##"<hx-partial hx-target="#toaster" hx-swap="beforeend"><div class="toast" role="status""##
//! ));
//!
//! // Filling a drawer slot opens the drawer.
//! let response = HxPartial::new()
//!     .parts(DRAWER.header(html! { h2 { "Profile" } }));
//! assert_eq!(
//!     response.render().into_string(),
//!     r##"<hx-partial hx-target="#profile-header"><h2>Profile</h2></hx-partial>"##
//! );
//! ```
//!
//! `examples/components` has the full wiring.

mod dialog;
mod drawer;
mod head;
mod modal;
mod toast;

pub use dialog::Dialog;
pub use drawer::{Drawer, Side};
pub use head::HEAD;
pub use modal::Slots;
pub use toast::{Action, Align, Category, Duration, Toast, Toaster};
