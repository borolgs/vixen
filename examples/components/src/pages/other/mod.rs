use axum::Router;
use vixen::{
    RouterExt,
    maud::{Markup, html},
    route,
};

use crate::shared::page;

pub fn router() -> Router {
    Router::new().view(other)
}

#[route("/other")]
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
