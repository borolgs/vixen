use axum::response::IntoResponse;
use vixen::HxPartial;

fn main() {
    let _ = HxPartial::new().into_response();
}
