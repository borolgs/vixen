use axum::Router;
use vixen::{
    fragment,
    maud::{Markup, html},
    routing::RouterExt,
    view_path,
};

use crate::shared::page;

pub fn router() -> Router {
    Router::new().typed_get(home)
}

#[fragment]
async fn hello() -> Markup {
    html! { div id=(Self) { "Hello" } }
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
            main {
                (hello().await)
            }
        },
    )
}
