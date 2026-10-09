//! Widgets for [Basecoat](https://basecoatui.com), behind the `basecoatui`
//! feature.
//!
//! Basecoat itself is not bundled. Import its CSS from the page stylesheet and
//! `basecoat-css/basecoat` from the page entry. Widgets with scripts require
//! their own modules too: `basecoat-css/toast` for [`Toaster`],
//! `basecoat-css/drawer` for [`Drawer`], and `basecoat-css/combobox` for
//! [`Combobox`]. [`Dialog`] needs no script.
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
//! // Updating a drawer opens it and clears any omitted slots.
//! let response = HxPartial::new()
//!     .parts(DRAWER.header(html! { h2 { "Profile" } }));
//! assert_eq!(
//!     response.render().into_string(),
//!     concat!(
//!         r##"<hx-partial hx-target="#profile-header"><h2>Profile</h2></hx-partial>"##,
//!         r##"<hx-partial hx-target="#profile-content"></hx-partial>"##,
//!         r##"<hx-partial hx-target="#profile-footer"></hx-partial>"##,
//!     )
//! );
//! ```
//!
//! [`Combobox`] renders inline and does not have a shell.
//!
//! See `examples/components` for the modal and toast setup, and `examples/shop`
//! for comboboxes.

mod combobox;
mod dialog;
mod drawer;
mod head;
mod modal;
mod table;
mod toast;

pub use combobox::Combobox;
pub use dialog::Dialog;
pub use drawer::{Drawer, Side};
pub use head::HEAD;
pub use modal::Slots;
pub use toast::{Action, Align, Category, Duration, Toast, Toaster};
