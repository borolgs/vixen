use axum::response::IntoResponse;
use vixen::{
    action, fragment,
    maud::{Markup, html},
    partial,
};

use super::form::form;

#[action("/greet")]
pub struct Greet {
    name: String,
}

pub async fn greet(Greet { name }: Greet) -> impl IntoResponse {
    partial! {
        greeting(&name),
        form(),
    }
}

#[fragment]
pub fn greeting(name: &str) -> Markup {
    html! {
        p id=(Self) { "Hello, " strong { (name) } "!" }
    }
}
