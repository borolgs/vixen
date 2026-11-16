use axum::{Router, routing::get};

#[cfg(target_arch = "wasm32")]
mod browser;

pub fn router() -> Router {
    Router::new().route(
        "/hello",
        get(|| async {
            tracing::info!(route = "/hello", "Request received");

            "Hello from axum, running in your browser!"
        }),
    )
}
