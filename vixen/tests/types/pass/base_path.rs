use axum::Router;
use vixen::route;

#[route("/")]
struct Root;

fn main() {
    let _: &'static str = vixen::base_path!();
    let _: Router = vixen::mount!(Router::new());
    let _: String = vixen::href!(Root).to_string();
}
