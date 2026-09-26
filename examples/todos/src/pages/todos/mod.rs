use std::sync::Mutex;

use axum::{Router, response::IntoResponse};
use vixen::{
    action, id,
    maud::{DOCTYPE, Markup, html},
    partial, route,
    routing::RouterExt,
};

static TODOS: Mutex<Vec<Todo>> = Mutex::new(Vec::new());

struct Todo {
    id: usize,
    title: String,
    done: bool,
}

#[id]
struct TodoListId;

#[id]
struct TodoCountId;

pub fn router() -> Router {
    Router::new()
        .typed_get(home)
        .typed_post(add)
        .typed_post(toggle)
}

#[route("/")]
struct HomePath;

async fn home(_: HomePath) -> Markup {
    let todos = TODOS.lock().unwrap();

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

async fn add(AddTodo { title }: AddTodo) -> impl IntoResponse {
    let mut todos = TODOS.lock().unwrap();
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

async fn toggle(ToggleTodo { id }: ToggleTodo) -> impl IntoResponse {
    let mut todos = TODOS.lock().unwrap();
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
