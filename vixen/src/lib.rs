#![doc = include_str!("../README.md")]
//!
//! ## Quick reference
//!
//! | To | Use |
//! |---|---|
//! | define a page route | [`#[route]`](macro@route) |
//! | define an htmx endpoint and call it from markup | [`#[action]`](macro@action), [`HxAction`], [`SyncStrategy`] |
//! | register pages and actions on a router | [`RouterExt`] |
//! | share an element ID between a page and its responses | [`#[id]`](macro@id), [`Selector`] |
//! | render and replace an element by its typed ID | [`#[fragment]`](macro@fragment), [`Fragment`] |
//! | return a main swap and targeted parts | [`partial!`], [`HxPartial`], [`Part`], [`Parts`] |
//! | bundle and serve page-local TS and CSS | [`assets!`], [`assets_router!`], and [`build`] in `build.rs` |
//! | resolve a static file to its content-hashed URL | [`asset!`] |
//! | serve the app under a base path | [`Config::base_path`], [`mount!`], [`href!`], [`base_path!`] |
//! | show a Basecoat toast from a handler | [`ui`], behind the `basecoatui` feature |
//!
//! ## Re-exports
//!
//! vixen re-exports the view-layer crates it builds on so the app can use the
//! same versions:
//!
//! - [`maud`] 0.27, with its `axum` feature, provides `html!` and `Markup`.
//!   Because `html!` expands to `extern crate maud;`, any crate that invokes
//!   it must also depend on `maud` directly.
//! - [`routing`] re-exports `axum_extra::routing` 0.12 for the `TypedPath`
//!   derives used by [`#[route]`](macro@route) and [`#[action]`](macro@action).
//!   Register handlers with [`RouterExt`].
//! - `hx` re-exports `axum_htmx` 0.8 header types and `SwapOption`; see [htmx 4
//!   compatibility](#htmx-4-compatibility) for caveats.
//!
//! [`build`] and [`Config`] come from `axum-vixen-bundler`, the `build.rs`
//! half. To call them, list `axum-vixen` under `[build-dependencies]` as well.

#![warn(missing_docs)]

// Lets unit tests use the macros, which emit `::vixen::` paths.
#[cfg(test)]
extern crate self as vixen;

// Public for paths emitted by `assets_router!`.
#[doc(hidden)]
pub mod assets;

mod action;
mod base_path;
mod fragment;
mod href;
mod id;
mod partial;
mod router;
#[cfg(feature = "testing")]
pub mod testing;
pub mod ui;

// Keep their docs above. Depending on whether rustdoc inlines a re-export,
// docs here are either hidden or appended to the original item's docs.
pub use axum_extra::routing;
pub use axum_htmx as hx;
pub use fragment::Fragment;
pub use href::{Asset, Href};
pub use id::Id;
pub use maud;
pub use vixen_bundler::{Config, build};

pub use partial::{HxPartial, HxPartialResponse, Part, Parts, Selector};
pub use router::{LastElementIs, RouterExt};

// Docs live on the definitions, where IDE hover finds them.
pub use vixen_macros::{ReqCtx, action, asset, assets, assets_router, fragment, id, route};

pub mod markers {
    //! Type-state markers used by [`HxPartial`](crate::HxPartial).
    //!
    //! `S` tracks whether the response has any content; `M` tracks whether it
    //! has a main body. These states keep empty responses and duplicate main
    //! bodies from compiling. Users do not construct the markers themselves.

    pub use crate::partial::{Empty, Filled, HasMain, NoMain};
}

pub use action::{HxAction, HxSync, SyncStrategy};

#[doc(hidden)]
pub mod __private {
    pub use {serde, serde_json, tokio};

    pub use crate::base_path::{base_path, mount};
}
