use axum::Router;
use vixen::view_path;

#[view_path("/")]
struct Root;

fn main() {
    let _: &'static str = vixen::base_path!();
    let _: Router = vixen::mount!(Router::new());
    let _: String = vixen::href!(Root).to_string();
}
