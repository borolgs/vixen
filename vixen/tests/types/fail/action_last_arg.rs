use axum::{Form, Router};
use serde::Deserialize;
use vixen::RouterExt;

#[derive(Deserialize)]
struct Rename {
    name: String,
}

// `.action()` needs a trailing typed route, not a plain `Form`.
async fn rename(Form(Rename { name }): Form<Rename>) -> String {
    name
}

fn main() {
    let _: Router = Router::new().action(rename);
}
