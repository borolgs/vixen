use vixen::{route, routing::TypedPath};

#[route("/")]
struct Root;

/// Already documented; the macro appends the route below.
#[route("/items/{id}")]
struct Item {
    id: u32,
}

fn main() {
    let _: &str = Root::PATH;
    let _ = Item { id: 7 }.to_uri();
}
