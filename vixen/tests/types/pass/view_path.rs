use vixen::{routing::TypedPath, view_path};

#[view_path("/")]
struct Root;

/// Already documented; the macro appends the route below.
#[view_path("/items/{id}")]
struct Item {
    id: u32,
}

fn main() {
    let _: &str = Root::PATH;
    let _ = Item { id: 7 }.to_uri();
}
