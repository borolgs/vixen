use std::sync::LazyLock;

use axum::{Router, middleware};
use vixen::agent::Skill;

use crate::{
    db::Db,
    schema::SCHEMA,
    state::{AppState, Ctx},
};

mod db;
mod models;
mod pages;
mod schema;
mod state;

const ADDR: &str = "127.0.0.1:4005";

/// Every page's head carries it; the router serves it at `/llms.txt`.
static SKILL: LazyLock<Skill> = LazyLock::new(|| {
    Skill::new(
        "A shop: the catalog at GET /, product admin at GET /admin/products, \
         material admin at GET /admin/materials. No login. A product's materials is \
         one field holding a JSON array, [{\"value\": \"<id>\", \"label\": \"<name>\"}], \
         ids and names from POST /admin/materials/options.",
    )
    .name("shop")
    .description("Drive the shop's catalog and admin pages over HTTP with curl.")
    .index(pages::catalog::actions())
    .index(pages::admin_products::actions())
    .index(pages::admin_materials::actions())
});

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let db = Db::open_in_memory().expect("failed to open the in-memory sqlite database");

    db.call(|conn| {
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.execute_batch(SCHEMA)?;
        Ok(())
    })
    .await
    .expect("failed to prepare the sqlite database");

    let state = AppState { db };

    let router = Router::new()
        .merge(pages::catalog::router())
        .merge(pages::admin_products::router())
        .merge(pages::admin_materials::router())
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
