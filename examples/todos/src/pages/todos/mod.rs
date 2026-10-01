use std::sync::Arc;

use axum::{Router, extract::State, http::StatusCode};
use tokio::sync::Mutex;
use vixen::{
    RouterExt, action, fragment, id,
    maud::{DOCTYPE, Markup, html},
    partial, route,
};

use crate::state::AppState;

pub struct Todo {
    pub id: usize,
    pub title: String,
    pub done: bool,
}

pub type Todos = Arc<Mutex<Vec<Todo>>>;

#[route("/")]
struct HomePath;

#[action("/todos/add")]
struct AddTodo {
    title: String,
}

#[action("/todos/toggle")]
struct ToggleTodo {
    id: usize,
}

#[action("/todos/edit")]
struct EditTodo {
    id: usize,
}

#[action("/todos/save")]
struct SaveTodo {
    id: usize,
    title: String,
}

#[action("/todos/cancel")]
struct CancelEdit {
    id: usize,
}

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .view(async |_: HomePath, State(todos): State<Todos>| {
            let todos = todos.lock().await;

            page(
                html! {
                    title { "Todos · vixen" }
                    (vixen::assets!())
                },
                html! {
                    h1 { "Todos" }
                    (todo_count(&todos))
                    (add_todo_form())
                    nav {
                        button type="button" data-show="all" aria-pressed="true" { "All" }
                        button type="button" data-show="active" aria-pressed="false" { "Active" }
                        button type="button" data-show="done" aria-pressed="false" { "Done" }
                    }
                    (todo_list(&todos))
                },
            )
        })
        .action(
            async |State(todos): State<Todos>, AddTodo { title }: AddTodo| {
                let mut todos = todos.lock().await;

                let id = todos.len();
                todos.push(Todo {
                    id,
                    title,
                    done: false,
                });

                partial! {
                    add_todo_form(),
                    todo_list(&todos),
                    todo_count(&todos)
                }
            },
        )
        .action(
            async |State(todos): State<Todos>, ToggleTodo { id }: ToggleTodo| {
                let mut todos = todos.lock().await;

                let Some(todo) = todos.get_mut(id) else {
                    return partial! {
                        todo_count(&todos)
                    };
                };

                todo.done = !todo.done;

                partial! {
                    todo_item(todo),
                    todo_count(&todos),
                }
            },
        )
        .action(
            async |State(todos): State<Todos>, EditTodo { id }: EditTodo| {
                let todos = todos.lock().await;
                let Some(todo) = todos.get(id) else {
                    return Err(StatusCode::NOT_FOUND);
                };

                Ok(partial!(todo_item_edit_form(todo)))
            },
        )
        .action(
            async |State(todos): State<Todos>, SaveTodo { id, title }: SaveTodo| {
                let mut todos = todos.lock().await;
                let Some(todo) = todos.get_mut(id) else {
                    return Err(StatusCode::NOT_FOUND);
                };
                todo.title = title;

                Ok(partial!(todo_item(todo)))
            },
        )
        .action(
            async |State(todos): State<Todos>, CancelEdit { id }: CancelEdit| {
                let todos = todos.lock().await;
                let Some(todo) = todos.get(id) else {
                    return Err(StatusCode::NOT_FOUND);
                };

                Ok(partial!(todo_item(todo)))
            },
        )
        .with_state(state)
}

#[id]
struct TodoItemId(usize);

#[fragment(TodoItemId(todo.id))]
fn todo_item(todo: &Todo) -> Markup {
    html! {
        li id=(Self) {
            label {
                input type="checkbox" checked[todo.done]
                    hx-action=(ToggleTodo::action().id(todo.id));
                (todo.title)
            }
            button type="button" hx-action=(EditTodo::action().id(todo.id)) { "Edit" }
        }
    }
}

#[fragment(TodoItemId(todo.id))]
fn todo_item_edit_form(todo: &Todo) -> Markup {
    html! {
        li id=(Self) {
            form hx-action=(SaveTodo::action().id(todo.id)) {
                input name=(SaveTodo::FIELD.title) value=(todo.title) required autofocus;
                button type="submit" { "Save" }
                button type="button" hx-action=(CancelEdit::action().id(todo.id)) { "Cancel" }
            }
        }
    }
}

#[fragment]
fn add_todo_form() -> Markup {
    html! {
        form id=(Self) hx-action=(AddTodo::action()) {
            input name=(AddTodo::FIELD.title) placeholder="What needs doing?" required autofocus;
            button type="submit" { "Add" }
        }
    }
}

#[fragment]
fn todo_count(todos: &[Todo]) -> Markup {
    let left = todos.iter().filter(|todo| !todo.done).count();

    html! {
        p id=(Self) {
            (left) " left"
        }
    }
}

#[fragment]
fn todo_list(todos: &[Todo]) -> Markup {
    html! {
        ul id=(Self) {
            @for todo in todos {
                (todo_item(todo))
            }
        }
    }
}

pub fn page(head: Markup, content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                (head)
            }
            body { (content) }
        }
    }
}
