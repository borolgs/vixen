use axum::{Router, response::IntoResponse};
use vixen::{
    action, fragment,
    maud::{DOCTYPE, Markup, html},
    partial,
    routing::RouterExt,
    view_path,
};

mod assets;

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
                title { "Config · vixen" }
                (vixen::assets!())
            }
            body {
                img src=(assets::SHIP) alt="ship" width="160" height="88";
                h1 { "Mounted at " code { (vixen::base_path!()) "/" } }
                main {
                    (pong(0))
                    a href=(HomePath) { "Reload" }
                }
            }
        }
    }
}

#[action("/ping")]
struct Ping {
    n: u32,
}

async fn ping(Ping { n }: Ping) -> impl IntoResponse {
    partial!(pong(n))
}

#[fragment]
fn pong(n: u32) -> Markup {
    html! {
        p id=(Self) {
            button hx-action=(Ping::action().n(n + 1)) { "Ping" }
            " pong #" (n)
        }
    }
}

#[tokio::main]
async fn main() {
    let router = vixen::mount!(
        Router::new()
            .typed_get(home)
            .typed_post(ping)
            .merge(vixen::assets_router!())
    );

    let listener = tokio::net::TcpListener::bind(ADDR)
        .await
        .unwrap_or_else(|e| panic!("failed to bind {ADDR}: {e}"));

    println!("listening on http://{ADDR}{}/", vixen::base_path!());

    axum::serve(listener, router).await.expect("server error");
}
