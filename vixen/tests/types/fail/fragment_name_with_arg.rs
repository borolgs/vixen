use vixen::fragment;

#[fragment("todo", id)]
fn todo(id: u64) -> vixen::maud::Markup {
    vixen::maud::html! { li id=(Self) {} }
}

fn main() {}
