mod hello;
mod preview;
mod todos;

#[cfg(target_arch = "wasm32")]
mod browser;

use axum::Router;
use vixen::RouterExt;

pub fn router() -> Router {
    Router::new()
        .view(preview::preview)
        .action(hello::greet)
        .merge(todos::router())
        .with_state(todos::Todos::default())
}
