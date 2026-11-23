mod hello;
mod preview;
mod todos;

#[cfg(target_arch = "wasm32")]
mod browser;

use axum::{Router, middleware};
use vixen::RouterExt;

pub fn router() -> Router {
    let state = todos::Todos::default();

    Router::new()
        .view(preview::preview)
        .merge(todos::router())
        .layer(middleware::from_fn_with_state(
            state.clone(),
            todos::Ctx::middleware,
        ))
        .with_state(state)
        .merge(hello::router())
}
