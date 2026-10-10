use vixen::{
    Page, Paged, action,
    maud::{Markup, html},
};

#[action("/todos/search")]
struct SearchTodos {
    #[cursor]
    after: Option<u32>,
}

const TODOS: Paged<SearchTodos, String> =
    Paged::new("todos", |title, _| html! { tr { td { (title) } } }).table::<1>();

fn main() {
    let search = SearchTodos { after: None };
    let page = Page {
        items: Vec::new(),
        next: None,
    };

    let _: Markup = TODOS.render(&search, page);
    let _: Markup = TODOS.view(&search, Err("down"));
}
