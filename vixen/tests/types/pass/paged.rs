use axum::response::Response;
use vixen::{
    HxAction, Page, Paged, PagedAction, Part, Selector, action,
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
    .loading(|next| html! { tr hx-action=(next) {} })
    .retry(|again| html! { tr hx-action=(again) { td { button { "Try again" } } } })
    .search_trigger("submit")
    .retry_trigger("click");

fn load(_: &SearchTodos) -> Result<Page<Todo, u32>, Part> {
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
        @if let Ok(page) = load(&search) { (TODOS.render(&search, page)) }
        (TODOS.shell(&search, html! { tr {} }))
    };

    // The error side is anything `Into<Parts>`.
    let _: Response = TODOS.respond(&search, load(&search));
    let _ = TODOS.respond(&search, Err(Vec::<Part>::new()));

    let _ = partial!(TODOS.refresh(), "#status" => html! { "Added" });
    let _: Selector = TODOS.into();
}
