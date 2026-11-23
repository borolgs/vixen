use axum::{Router, http::StatusCode, response::IntoResponse};
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
        form hx-action=(Greet::action()) id=(Self) {
            input name=(Greet::FIELD.name) placeholder="Your name"
                aria-label="Your name" maxlength="80";
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

#[fragment(GreetingId)]
fn warning(message: &str) -> Markup {
    html! { p id=(Self) role="alert" { (message) } }
}

#[action("/hello/greet")]
pub struct Greet {
    name: String,
}

pub async fn greet(Greet { name }: Greet) -> impl IntoResponse {
    if name.trim().is_empty() {
        return partial! {
            StatusCode::UNPROCESSABLE_ENTITY,
            warning("A name is required."),
        };
    }
    partial! {
        greeting(&name),
        form(),
    }
}
