use axum::Router;
use vixen::{
    maud::{Markup, html},
    routing::RouterExt,
    view_path,
};

use crate::shared::page;

pub fn router() -> Router {
    Router::new().typed_get(home)
}

#[view_path("/")]
struct HomePath;

async fn home(_: HomePath) -> Markup {
    page(
        html! {
            title { "Components · vixen" }
            (vixen::assets!())
        },
        html! {
            h1 { "Components" }
            main {}
        },
    )
}
