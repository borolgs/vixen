use vixen::{fragment, id};

#[id]
struct TodoId(u64);

#[fragment(TodoId)]
fn todo() -> vixen::maud::Markup {
    vixen::maud::html! { li id=(Self) {} }
}

fn main() {}
