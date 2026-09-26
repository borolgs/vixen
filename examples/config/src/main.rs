use axum::Router;
use vixen::{
    maud::{DOCTYPE, Markup, html},
    routing::RouterExt,
    view_path,
};

const ADDR: &str = "127.0.0.1:4004";

#[view_path("/")]
struct HomePath;

async fn home(_: HomePath) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { "Counter · vixen" }
                (vixen::assets!())
            }
            body {
                main {
                    "TODO"
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let router = Router::new().typed_get(home).merge(vixen::assets_router!());

    let listener = tokio::net::TcpListener::bind(ADDR)
        .await
        .unwrap_or_else(|e| panic!("failed to bind {ADDR}: {e}"));

    println!("listening on http://{ADDR}/");

    axum::serve(listener, router).await.expect("server error");
}
