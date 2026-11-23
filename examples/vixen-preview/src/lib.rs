mod counter;
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
        .merge(counter::router())
        .with_state(counter::Counter::default())
        .merge(todos::router().with_state(state.clone()))
        .layer(middleware::from_fn_with_state(
            state,
            todos::Ctx::middleware,
        ))
        .merge(hello::router())
}
