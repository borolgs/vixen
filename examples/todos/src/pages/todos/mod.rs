use std::sync::Arc;

use axum::{Router, extract::State, response::IntoResponse};
use tokio::sync::Mutex;
use vixen::{
    RouterExt, action, id,
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

#[id]
struct TodoListId;

#[id]
struct TodoCountId;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .view(home)
        .action(add)
        .action(toggle)
        .with_state(state)
}

#[route("/")]
struct HomePath;

async fn home(_: HomePath, State(todos): State<Todos>) -> Markup {
    let todos = todos.lock().await;

    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { "Todos · vixen" }
                (vixen::assets!())
            }
            body {
                h1 { "Todos" }
                p id=(TodoCountId) { (todo_count(&todos)) }
                form hx-action=(AddTodo::action()) { (add_todo_fields()) }
                nav {
                    button type="button" data-show="all" { "All" }
                    button type="button" data-show="active" { "Active" }
                    button type="button" data-show="done" { "Done" }
                }
                ul id=(TodoListId) { (todo_list(&todos)) }
            }
        }
    }
}

#[action("/todos/add")]
struct AddTodo {
    title: String,
}

async fn add(
    _: AddTodoPath,
    State(todos): State<Todos>,
    AddTodo { title }: AddTodo,
) -> impl IntoResponse {
    let mut todos = todos.lock().await;

    let id = todos.len();
    todos.push(Todo {
        id,
        title,
        done: false,
    });

    partial!(
        _ => add_todo_fields(),
        TodoListId => todo_list(&todos),
        TodoCountId => todo_count(&todos),
    )
}

#[action("/todos/toggle")]
struct ToggleTodo {
    id: usize,
}

async fn toggle(State(todos): State<Todos>, ToggleTodo { id }: ToggleTodo) -> impl IntoResponse {
    let mut todos = todos.lock().await;
    if let Some(todo) = todos.get_mut(id) {
        todo.done = !todo.done;
    }

    partial! {
        TodoListId => todo_list(&todos),
        TodoCountId => todo_count(&todos)
    }
}

fn add_todo_fields() -> Markup {
    html! {
        input name=(AddTodo::FIELD.title) placeholder="What needs doing?" required autofocus;
        button type="submit" { "Add" }
    }
}

fn todo_count(todos: &[Todo]) -> Markup {
    let left = todos.iter().filter(|todo| !todo.done).count();

    html! { (left) " left" }
}

fn todo_list(todos: &[Todo]) -> Markup {
    html! {
        @for todo in todos {
            li {
                label {
                    input type="checkbox" checked[todo.done]
                        hx-action=(ToggleTodo::action().id(todo.id));
                    (todo.title)
                }
            }
        }
    }
}
