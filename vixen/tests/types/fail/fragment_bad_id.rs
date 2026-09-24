use vixen::fragment;

#[fragment("no spaces")]
fn todo_list() -> vixen::maud::Markup {
    vixen::maud::html! { ul id=(Self) {} }
}

fn main() {}
