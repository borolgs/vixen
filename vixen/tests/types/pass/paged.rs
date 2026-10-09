use vixen::{
    Page, Paged, Selector, action,
    maud::{Markup, html},
    partial,
};

#[action("/todos/search")]
struct SearchTodos {
    q: String,
    #[cursor]
    after: Option<String>,
}

struct Todo {
    title: String,
}

fn row(todo: &Todo) -> Markup {
    html! { tr { td { (todo.title) } } }
}

// Every slot is a `fn`, so the list is a `const`.
const TODOS: Paged<SearchTodos, Todo> = Paged::new("todos", row)
    .list(|id, _, rows| html! { tbody id=(id) { (rows) } })
    // A closure that reads the search names its type.
    .empty(|search: &SearchTodos| html! { tr { td { "Nothing for " (search.q) } } })
    .failed(|_| html! { tr { td { "Down" } } })
    .loading(|next| html! { tr hx-action=(next) {} })
    .retry(|again| html! { tr hx-action=(again) { td { button { "Try again" } } } })
    .search_trigger("submit")
    .retry_trigger("click");

fn load(_: &SearchTodos) -> std::io::Result<Page<Todo, String>> {
    Ok(Page {
        items: Vec::new(),
        next: Some("10".into()),
    })
}

fn main() {
    let search = SearchTodos {
        q: "milk".into(),
        after: None,
    };

    let _ = html! {
        form hx-action=(TODOS.search(SearchTodos::action())) {}
        (TODOS.view(&search, load(&search)))
    };

    // The error side is any type.
    let _: Markup = TODOS.view(&search, Err("down"));
    // A page that cannot fail skips the `Result`.
    if let Ok(page) = load(&search) {
        let _: Markup = TODOS.render(&search, page);
    }

    let _ = partial!(TODOS.refresh(), "#status" => html! { "Added" });
    let _: Selector = TODOS.into();
}
