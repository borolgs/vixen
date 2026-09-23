use axum::Router;
use vixen::{
    maud::{Markup, html},
    routing::RouterExt,
    view_path,
};

use crate::shared::page;

pub fn router() -> Router {
    Router::new().typed_get(other)
}

#[view_path("/other")]
struct OtherPath;

async fn other(_: OtherPath) -> Markup {
    page(
        html! {
            title { "Other · vixen" }
            (vixen::assets!())
        },
        html! {
            h1 { "Other" }
            main {
                p { "Second page — shares the htmx/app.css chunk with the home page." }
            }
        },
    )
}
