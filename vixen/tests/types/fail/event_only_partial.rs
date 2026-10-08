use axum::response::IntoResponse;
use vixen::{hx::HxEvent, partial};

fn main() {
    let _ = partial!(HxEvent::new("saved")).into_response();
}
