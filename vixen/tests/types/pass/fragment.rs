use vixen::{Fragment, Part, maud::html, partial};

mod list {
    use vixen::{
        fragment,
        maud::{Markup, html},
    };

    // `pub fn` gives `pub struct TodoListId;`, so the id is usable outside.
    #[fragment]
    pub fn todo_list(todos: &[&str]) -> Markup {
        // `Self` is the id type in plain code and inside `html!`.
        let clear = Self::SEL;
        if todos.is_empty() {
            return html! { ul id=(Self) {} };
        }
        html! {
            ul id=(Self) {
                @for todo in todos {
                    li hx-target=(Self::SEL) { (todo) }
                }
                button hx-target=(clear) { "clear" }
            }
        }
    }

    #[fragment]
    pub async fn todo_count(n: usize) -> Markup {
        html! { output id=(Self) { (n) } }
    }

    // A literal overrides the id, as with `#[id("..")]`.
    #[fragment("sidebar-nav")]
    pub fn sidebar() -> Markup {
        html! { nav id=(Self) {} }
    }
}

use list::{SidebarId, TodoCountId, TodoListId, sidebar, todo_count, todo_list};

async fn count() -> Fragment<TodoCountId> {
    todo_count(1).await
}

fn main() {
    let _: &str = TodoListId::ID;
    let _: &str = TodoListId::SEL;
    assert_eq!(SidebarId::ID, "sidebar-nav");
    let _ = sidebar();
    let _ = count();

    let frag: Fragment<TodoListId> = todo_list(&["a"]);
    // A fragment is the bare element in markup, and one part in a partial.
    let _ = html! { (todo_list(&[])) };
    let _: Part = frag.into();
    let _ = partial!(todo_list(&["a"]));
    let _ = partial!(todo_list(&["a"]), TodoListId => html! { "b" });
    let _ = partial!(_ => html! { "main" }, todo_list(&[]));
}
