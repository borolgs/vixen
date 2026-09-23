use axum::Router;
use maud::DOCTYPE;
use vixen::{
    maud::{Markup, html},
    routing::RouterExt,
    view_path,
};

pub fn router() -> Router {
    Router::new().typed_get(home)
}

#[view_path("/")]
struct HomePath;

async fn home(_: HomePath) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { "Components · vixen" }
                (vixen::assets!())
            }
            body {
                h1 { "Components" }
                main {

                }
            }
        }
    }
}
