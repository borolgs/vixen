use vixen::{
    Fragment, Id, fragment, id,
    maud::{Markup, html},
    partial,
};

#[id]
struct CartId;

#[fragment(CartId)]
fn cart(n: usize) -> Markup {
    html! { div id=(Self) { (n) } }
}

#[id]
struct TodoId(u64);

struct Todo {
    id: u64,
    title: String,
}

// `Self` is the value `TodoId(todo.id)`, in plain code and inside `html!`.
#[fragment(TodoId(todo.id))]
fn todo_item(todo: &Todo) -> Markup {
    let sel = Self.sel();
    html! { li id=(Self) hx-target=(sel) { (todo.title) } }
}

// The id is built before the body, which moves `title`.
#[fragment(TodoId(id))]
async fn todo_async(id: u64, title: String) -> Markup {
    html! { li id=(Self) { (title) } }
}

fn main() {
    let _: Fragment<CartId> = cart(1);
    let _ = CartId::slot();

    let todo = Todo {
        id: 7,
        title: "milk".into(),
    };
    let _: Fragment<TodoId> = todo_item(&todo);
    let _ = html! { (todo_item(&todo)) };
    let _ = partial!(todo_item(&todo), cart(2));
    let _ = TodoId(7).slot();
    let _ = todo_async(7, "milk".into());
}
