use std::sync::Arc;

use axum::{Router, extract::State, response::IntoResponse};
use tokio::sync::Mutex;
use vixen::{
    RouterExt, action, fragment,
    maud::{Markup, html},
    partial, route,
};

pub fn router() -> Router<Counter> {
    Router::new().view(view).action(add)
}

pub type Counter = Arc<Mutex<i64>>;

#[route("/counter")]
pub struct Index;

pub async fn view(_: Index, State(count): State<Counter>) -> Markup {
    let count = *count.lock().await;

    html! {
        (counter_kind(count))
        div class="counter-controls" {
            button type="button" aria-label="Decrease"
                hx-action=(Add::action().by(-1)) { "−" }
            (counter_value(count))
            button type="button" aria-label="Increase"
                hx-action=(Add::action().by(1)) { "+" }
        }
        p { "Each click updates Rust state and returns two HTML fragments." }
    }
}

#[action("/counter/add")]
struct Add {
    by: i64,
}

async fn add(
    State(count): State<Counter>,
    Add { by }: Add,
) -> impl IntoResponse {
    let mut count = count.lock().await;
    *count = count.saturating_add(by);

    partial! {
        counter_value(*count),
        counter_kind(*count),
    }
}

#[fragment]
fn counter_value(count: i64) -> Markup {
    html! { output id=(Self) aria-label="Count" { (count) } }
}

#[fragment]
fn counter_kind(count: i64) -> Markup {
    let kind = match count {
        0 => "Zero",
        n if n % 2 == 0 => "Even",
        _ => "Odd",
    };
    html! { h2 id=(Self) { (kind) } }
}
