use axum::Router;

mod pages;
mod shared;

const ADDR: &str = "127.0.0.1:4003";

#[tokio::main]
async fn main() {
    let router = Router::new()
        .merge(pages::components::router())
        .merge(pages::other::router())
        .merge(vixen::assets_router!());

    let listener = tokio::net::TcpListener::bind(ADDR)
        .await
        .unwrap_or_else(|e| panic!("failed to bind {ADDR}: {e}"));

    println!("listening on http://{ADDR}/");

    axum::serve(listener, router).await.expect("server error");
}
