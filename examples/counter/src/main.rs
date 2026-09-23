use std::sync::atomic::{AtomicI64, Ordering};

use axum::{Router, response::IntoResponse};
use vixen::{
    action, fragment, id,
    maud::{DOCTYPE, Markup, html},
    partial,
    routing::RouterExt,
    view_path,
};

const ADDR: &str = "127.0.0.1:4002";

static COUNT: AtomicI64 = AtomicI64::new(0);

#[id]
struct CountId;

#[view_path("/")]
struct HomePath;

async fn home(_: HomePath) -> Markup {
    let count = COUNT.load(Ordering::Relaxed);

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
                (heading(count))
                main {
                    button hx-action=(Add::action().by(-1)) { "−" }
                    output id=(CountId) { (count) }
                    button hx-action=(Add::action().by(1)) { "+" }
                }
            }
        }
    }
}

#[action("/add")]
struct Add {
    by: i64,
}

async fn add(Add { by }: Add) -> impl IntoResponse {
    let count = COUNT.fetch_add(by, Ordering::Relaxed) + by;

    partial!(
        CountId => html! { (count) },
        heading(count),
    )
}

#[fragment]
fn heading(count: i64) -> Markup {
    let title = match count {
        0 => "Zero",
        n if n % 2 == 0 => "Even",
        _ => "Odd",
    };

    html! {  h1 id=(Self) { (title) } }
}

#[tokio::main]
async fn main() {
    let router = Router::new()
        .typed_get(home)
        .typed_post(add)
        .merge(vixen::assets_router!());

    let listener = tokio::net::TcpListener::bind(ADDR)
        .await
        .unwrap_or_else(|e| panic!("failed to bind {ADDR}: {e}"));

    println!("listening on http://{ADDR}/");

    axum::serve(listener, router).await.expect("server error");
}
