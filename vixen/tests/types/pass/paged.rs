use vixen::{
    HxAction, Page, Paged, PagedAction, Selector, action,
    maud::{Markup, html},
    partial,
};

#[action("/todos/search")]
struct SearchTodos {
    q: String,
    after: Option<u32>,
}

impl PagedAction for SearchTodos {
    type Cursor = u32;

    fn cursor(&self) -> Option<u32> {
        self.after
    }

    fn next(&self, after: u32) -> HxAction {
        SearchTodos::action().q(&self.q).after(after).hx()
    }
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

fn load(_: &SearchTodos) -> std::io::Result<Page<Todo, u32>> {
    Ok(Page {
        items: Vec::new(),
        next: Some(10),
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
