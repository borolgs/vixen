use std::sync::Arc;

use axum::{Router, middleware};
use tokio::sync::Mutex;

use crate::{
    model::{Priority, Todo},
    state::{AppState, Ctx},
};

mod model;
mod state;
mod pages {
    pub mod todos;
}

const ADDR: &str = "127.0.0.1:4001";

#[tokio::main]
async fn main() {
    let todos = vec![
        Todo {
            id: 0,
            title: "Buy milk".into(),
            done: true,
            priority: Priority::Normal,
        },
        Todo {
            id: 1,
            title: "Water the plants".into(),
            done: false,
            priority: Priority::Low,
        },
        Todo {
            id: 2,
            title: "Read the htmx 4 docs".into(),
            done: false,
            priority: Priority::High,
        },
    ];

    let state = AppState {
        todos: Arc::new(Mutex::new(todos)),
    };
    let router = Router::new()
        .merge(pages::todos::router())
        .merge(vixen::assets_router!())
        .layer(middleware::from_fn_with_state(
            state.clone(),
            Ctx::middleware,
        ))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(ADDR)
        .await
        .unwrap_or_else(|e| panic!("failed to bind {ADDR}: {e}"));

    println!("listening on http://{ADDR}/");

    axum::serve(listener, router).await.expect("server error");
}
