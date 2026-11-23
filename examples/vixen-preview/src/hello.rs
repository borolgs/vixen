use axum::{Router, response::IntoResponse};
use vixen::{
    RouterExt, action, fragment,
    maud::{Markup, html},
    partial, route,
};

pub fn router() -> Router {
    Router::new().view(view).action(greet)
}

#[route("/hello")]
pub struct Index;

pub async fn view(_: Index) -> Markup {
    html! {
        (form())
        p { "Submit a name to update the greeting below." }
        div role="status" aria-live="polite" {
            (greeting::slot())
        }
    }
}

#[fragment]
fn form() -> Markup {
    html! {
        form id=(Self) hx-action=(Greet::action()) {
            input name=(Greet::FIELD.name) placeholder="Your name"
                aria-label="Your name" required maxlength="80";
            button type="submit" { "Greet" }
        }
    }
}

#[fragment]
fn greeting(name: &str) -> Markup {
    html! {
        p id=(Self) { "Hello, " strong { (name) } "!" }
    }
}

#[action("/hello/greet")]
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
