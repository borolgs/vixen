use axum::{Router, http::StatusCode, response::IntoResponse};
use axum_extra::extract::Query;
use strum::IntoEnumIterator;
use vixen::{
    HxCurrentQuery, HxPartialResponse, Part, RouterExt, SyncStrategy, action, fragment, id,
    maud::{DOCTYPE, Markup, html},
    partial, replace_query, route,
};

use crate::{
    model::{Priority, Todo, TodoFilter},
    state::{AppState, ctx},
};

pub fn router() -> Router<AppState> {
    Router::new()
        .view(home)
        .action(search)
        .action(add)
        .action(toggle)
        .action(edit)
        .action(save)
        .action(cancel)
}

#[route("/")]
struct HomePath;

async fn home(_: HomePath, Query(search): Query<SearchTodos>) -> Markup {
    let todos = ctx().todos.lock_owned().await;

    page(
        html! {
            title { "Todos · vixen" }
            (vixen::assets!())
        },
        html! {
            h1 { "Todos" }
            div.status {
                (todo_count(&todos))
                (todo_message::slot())
            }
            (add_todo_form())
            (search_form(&search))
            (todo_list(&todos, &search))
        },
    )
}

#[action("/todos/search")]
struct SearchTodos {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    q: String,
    #[serde(default)]
    filter: TodoFilter,
}

impl SearchTodos {
    fn matches(&self, todo: &Todo) -> bool {
        self.filter.matches(todo) && todo.title.to_lowercase().contains(&self.q.to_lowercase())
    }
}

async fn search(search: SearchTodos) -> impl IntoResponse {
    let todos = ctx().todos.lock_owned().await;

    (
        replace_query(&search),
        partial! {
            todo_list(&todos, &search)
        },
    )
}

#[action("/todos/add")]
struct AddTodo {
    title: String,
    priority: Priority,
}

async fn add(
    HxCurrentQuery(search): HxCurrentQuery<SearchTodos>,
    AddTodo { title, priority }: AddTodo,
) -> HxPartialResponse {
    if title.is_empty() {
        return partial! {
            StatusCode::UNPROCESSABLE_ENTITY,
            todo_message(Message::Error("A todo needs a title.")),
        };
    }

    let mut todos = ctx().todos.lock_owned().await;

    let todo = Todo {
        id: todos.len(),
        title,
        done: false,
        priority,
    };
    todos.push(todo.clone());

    if !search.matches(&todo) {
        return partial! {
            add_todo_form(),
            todo_count(&todos),
            todo_message(Message::Info("Todo created.")),
        };
    }

    partial! {
        add_todo_form(),
        Part::before_end(todo_list::sel(), todo_item(&todo)),
        todo_count(&todos),
        todo_message(Message::Info("Todo created.")),
    }
}

#[action("/todos/toggle")]
struct ToggleTodo {
    id: usize,
}

async fn toggle(
    HxCurrentQuery(search): HxCurrentQuery<SearchTodos>,
    ToggleTodo { id }: ToggleTodo,
) -> HxPartialResponse {
    let mut todos = ctx().todos.lock_owned().await;

    let Some(todo) = todos.get_mut(id) else {
        return partial!(
            StatusCode::NOT_FOUND,
            todo_message(Message::Error("That todo is gone.")),
        );
    };

    todo.done = !todo.done;

    if !search.matches(todo) {
        return partial! {
            Part::delete(TodoItemId(id)),
            todo_count(&todos),
        };
    }

    partial! {
        todo_item(todo),
        todo_count(&todos),
    }
}

#[action("/todos/edit")]
struct EditTodo {
    id: usize,
}

async fn edit(EditTodo { id }: EditTodo) -> HxPartialResponse {
    let todos = ctx().todos.lock_owned().await;
    let Some(todo) = todos.get(id) else {
        return partial!(
            StatusCode::NOT_FOUND,
            todo_message(Message::Error("That todo is gone.")),
        );
    };

    partial!(todo_item_edit_form(todo))
}

#[action("/todos/save")]
struct SaveTodo {
    id: usize,
    title: String,
    #[serde(default)]
    done: bool,
    priority: Priority,
}

async fn save(
    HxCurrentQuery(search): HxCurrentQuery<SearchTodos>,
    SaveTodo {
        id,
        title,
        done,
        priority,
    }: SaveTodo,
) -> HxPartialResponse {
    if title.is_empty() {
        return partial! {
            StatusCode::UNPROCESSABLE_ENTITY,
            todo_message(Message::Error("A todo needs a title.")),
        };
    }

    let mut todos = ctx().todos.lock_owned().await;
    let Some(todo) = todos.get_mut(id) else {
        return partial!(
            StatusCode::NOT_FOUND,
            todo_message(Message::Error("That todo is gone.")),
        );
    };
    todo.title = title;
    todo.done = done;
    todo.priority = priority;

    if !search.matches(todo) {
        return partial! {
            Part::delete(TodoItemId(id)),
            todo_message(Message::Info("Todo updated.")),
        };
    }

    partial! {
        todo_item(todo),
        todo_message(Message::Info("Todo updated.")),
    }
}

#[action("/todos/cancel")]
struct CancelEdit {
    id: usize,
}

async fn cancel(CancelEdit { id }: CancelEdit) -> HxPartialResponse {
    let todos = ctx().todos.lock_owned().await;
    let Some(todo) = todos.get(id) else {
        return partial!(
            StatusCode::NOT_FOUND,
            todo_message(Message::Error("That todo is gone.")),
        );
    };

    partial!(todo_item(todo))
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
            small { (todo.priority.as_ref()) }
            button type="button" hx-action=(EditTodo::action().id(todo.id)) { "Edit" }
        }
    }
}

#[fragment(TodoItemId(todo.id))]
fn todo_item_edit_form(todo: &Todo) -> Markup {
    html! {
        li id=(Self) {
            form hx-action=(SaveTodo::action().id(todo.id)) {
                input type="checkbox" name=(SaveTodo::FIELD.done) value="true" checked[todo.done];
                input name=(SaveTodo::FIELD.title) value=(todo.title) required autofocus;
                (priority_select(SaveTodo::FIELD.priority, todo.priority))
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
            input name=(AddTodo::FIELD.title) placeholder="What needs doing?" autofocus;
            (priority_select(AddTodo::FIELD.priority, Priority::default()))
            button type="submit" { "Add" }
        }
    }
}

fn priority_select(name: &str, selected: Priority) -> Markup {
    html! {
        select name=(name) aria-label="Priority" {
            @for priority in Priority::iter() {
                option value=(priority.as_ref()) selected[priority == selected] {
                    (priority.as_ref())
                }
            }
        }
    }
}

fn search_form(search: &SearchTodos) -> Markup {
    let action = SearchTodos::action()
        .hx()
        .trigger("input delay:200ms, submit")
        .sync(SyncStrategy::Replace);

    html! {
        form hx-action=(action) {
            input type="search" name=(SearchTodos::FIELD.q) value=(search.q) placeholder="Search";
            fieldset {
                @for filter in TodoFilter::iter() {
                    label {
                        input type="radio" name=(SearchTodos::FIELD.filter) value=(filter.as_ref())
                            checked[filter == search.filter];
                        (filter.as_ref())
                    }
                }
            }
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

enum Message {
    Info(&'static str),
    Error(&'static str),
}

#[fragment]
fn todo_message(message: Message) -> Markup {
    html! {
        @match message {
            Message::Info(text) => {
                p id=(Self) role="status" hx-on="load delay:3s -> this.replaceChildren()" {
                    (text)
                }
            }
            Message::Error(text) => {
                p id=(Self) role="alert" { (text) }
            }
        }
    }
}

#[fragment]
fn todo_list(todos: &[Todo], search: &SearchTodos) -> Markup {
    html! {
        ul id=(Self) {
            @for todo in todos.iter().filter(|todo| search.matches(todo)) {
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
                (vixen::agent::skill("A todo list on one page. No login."))
            }
            body { (content) }
        }
    }
}
