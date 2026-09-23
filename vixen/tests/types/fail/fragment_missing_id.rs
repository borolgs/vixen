use vixen::fragment;

#[fragment]
fn todo_list() -> vixen::maud::Markup {
    vixen::maud::html! { ul { li { "todo" } } }
}

fn main() {}
