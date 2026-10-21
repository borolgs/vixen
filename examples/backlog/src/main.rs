use std::sync::LazyLock;

use axum::{Router, middleware};
use vixen::agent::Skill;

use crate::{
    db::Db,
    schema::{SCHEMA, seed},
    state::{AppState, Ctx},
};

mod db;
mod pages;
mod schema;
mod shared;
mod sort;
mod state;

const ADDR: &str = "127.0.0.1:4006";

/// Every page's head carries it; the router serves it at `/llms.txt`.
static SKILL: LazyLock<Skill> = LazyLock::new(|| {
    Skill::new("An issue tracker on one page at GET /. No login.")
        .name("backlog")
        .description("Drive the backlog issue tracker over HTTP with curl.")
        .index(pages::issues::actions())
});

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let db = Db::open_in_memory().expect("failed to open the in-memory sqlite database");

    db.call(|conn| {
        conn.execute_batch(SCHEMA)?;
        seed(conn)?;
        Ok(())
    })
    .await
    .expect("failed to prepare the sqlite database");

    let state = AppState { db };

    let router = Router::new()
        .merge(pages::issues::routes())
        .merge(SKILL.router())
        .merge(vixen::assets_router!())
        .layer(middleware::from_fn_with_state(
            state.clone(),
            Ctx::middleware,
        ))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(ADDR)
        .await
        .unwrap_or_else(|e| panic!("failed to bind {ADDR}: {e}"));

    tracing::info!("listening on http://{ADDR}/");

    axum::serve(listener, router).await.expect("server error");
}
