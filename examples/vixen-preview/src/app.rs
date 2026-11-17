use axum::{Router, response::IntoResponse};
use vixen::{RouterExt, action, partial};

use super::{
    preview::preview,
    ui::{form, greeting},
};

#[action("/greet")]
pub struct Greet {
    name: String,
}

pub async fn greet(Greet { name }: Greet) -> impl IntoResponse {
    tracing::info!("greeting");
    partial! {
        greeting(&name),
        form(),
    }
}

pub fn router() -> Router {
    Router::new().view(preview).action(greet)
}
