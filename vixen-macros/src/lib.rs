//! The proc macros behind `vixen`.
//!
//! Use them through the `vixen` crate. Their expansions refer to `::vixen::`,
//! so `axum-vixen-macros` is not a standalone API.

// User-facing docs live on the re-exports in `vixen/src/lib.rs`. Doctests here
// cannot resolve `::vixen`, and rustdoc would append these docs to the ones on
// the re-exports.
#![allow(missing_docs)]

use proc_macro::TokenStream;

mod action;
mod assets;
mod fragment;
mod id;
mod view;

#[proc_macro_attribute]
pub fn action(attr: TokenStream, item: TokenStream) -> TokenStream {
    action::expand(attr.into(), item.into()).into()
}

#[proc_macro_attribute]
pub fn id(attr: TokenStream, item: TokenStream) -> TokenStream {
    id::expand(attr.into(), item.into()).into()
}

#[proc_macro_attribute]
pub fn view_path(attr: TokenStream, item: TokenStream) -> TokenStream {
    view::expand(attr.into(), item.into()).into()
}

#[proc_macro]
pub fn assets_router(item: TokenStream) -> TokenStream {
    assets::expand_assets_router(item.into()).into()
}

#[proc_macro]
pub fn assets(item: TokenStream) -> TokenStream {
    assets::expand_assets_head(item.into()).into()
}

#[proc_macro]
pub fn asset(item: TokenStream) -> TokenStream {
    assets::expand_asset(item.into()).into()
}

#[proc_macro_attribute]
pub fn fragment(attr: TokenStream, item: TokenStream) -> TokenStream {
    fragment::expand(attr.into(), item.into()).into()
}
