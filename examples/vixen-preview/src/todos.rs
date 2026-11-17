use std::sync::{Arc, Mutex};

use axum::{Router, extract::State, http::StatusCode};
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, IntoEnumIterator};
use vixen::{
    HxPartialResponse, Part, RouterExt, SyncStrategy, action, fragment, id,
    maud::{Markup, html},
    partial,
    routing::TypedPath,
};

pub type Todos = Arc<Mutex<TodoState>>;

pub struct TodoState {
    todos: Vec<Todo>,
    search: SearchTodos,
}

impl Default for TodoState {
    fn default() -> Self {
        let todos = [
            ("Buy milk", true, Priority::Normal),
            ("Water the plants", false, Priority::Low),
            ("Read the htmx 4 docs", false, Priority::High),
        ]
        .into_iter()
        .enumerate()
        .map(|(id, (title, done, priority))| Todo {
            id,
            title: title.into(),
            done,
            priority,
        })
        .collect();

        Self {
            todos,
            search: SearchTodos::default(),
        }
    }
}

#[derive(Clone)]
struct Todo {
    id: usize,
    title: String,
    done: bool,
    priority: Priority,
}

#[derive(
    Clone, Copy, Default, PartialEq, Serialize, Deserialize, AsRefStr, EnumIter,
)]
enum Priority {
    Low,
    #[default]
    Normal,
    High,
}

#[derive(
    Clone, Copy, Default, PartialEq, Serialize, Deserialize, AsRefStr, EnumIter,
)]
enum TodoFilter {
    #[default]
    All,
    Active,
    Done,
}

impl TodoFilter {
    fn matches(self, todo: &Todo) -> bool {
        match self {
            Self::All => true,
            Self::Active => !todo.done,
            Self::Done => todo.done,
        }
    }
}

pub fn router() -> Router<Todos> {
    Router::new()
        .action(search)
        .action(add)
        .action(toggle)
        .action(edit)
        .action(save)
        .action(cancel)
}

pub fn view(state: &TodoState) -> Markup {
    html! {
        h2 { "Todos" }
        div class="todo-status" {
            (todo_count(&state.todos))
            (todo_message::slot())
        }
        (add_todo_form())
        (search_form(&state.search))
        (todo_list(&state.todos, &state.search))
    }
}

/// The todo list for the query and filter.
#[derive(Default)]
#[action("/todos/search")]
struct SearchTodos {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    q: String,
    #[serde(default)]
    filter: TodoFilter,
}

impl SearchTodos {
    fn matches(&self, todo: &Todo) -> bool {
        self.filter.matches(todo)
            && todo.title.to_lowercase().contains(&self.q.to_lowercase())
    }
}

async fn search(
    State(state): State<Todos>,
    search: SearchTodos,
) -> HxPartialResponse {
    let mut state = state.lock().expect("todo state poisoned");
    state.search = search;

    partial!(todo_list(&state.todos, &state.search))
}

/// The new todo's item, the count and a message.
#[action("/todos/add")]
struct AddTodo {
    title: String,
    priority: Priority,
}

async fn add(
    State(state): State<Todos>,
    AddTodo { title, priority }: AddTodo,
) -> HxPartialResponse {
    if title.trim().is_empty() {
        return partial! {
            StatusCode::UNPROCESSABLE_ENTITY,
            todo_message(Message::Error("A todo needs a title.")),
        };
    }

    let mut state = state.lock().expect("todo state poisoned");
    let TodoState { todos, search } = &mut *state;

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
            todo_count(todos),
            todo_message(Message::Info("Todo created.")),
        };
    }

    partial! {
        add_todo_form(),
        Part::before_end(todo_list::sel(), todo_item(&todo)),
        todo_count(todos),
        todo_message(Message::Info("Todo created.")),
    }
}

/// The item with `done` flipped, and the count.
#[action("/todos/toggle")]
struct ToggleTodo {
    id: usize,
}

async fn toggle(
    State(state): State<Todos>,
    ToggleTodo { id }: ToggleTodo,
) -> HxPartialResponse {
    let mut state = state.lock().expect("todo state poisoned");
    let TodoState { todos, search } = &mut *state;

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
            todo_count(todos),
        };
    }

    partial! {
        todo_item(todo),
        todo_count(todos),
    }
}

/// The item as an edit form posting `/todos/save`.
#[action("/todos/edit")]
struct EditTodo {
    id: usize,
}

async fn edit(
    State(state): State<Todos>,
    EditTodo { id }: EditTodo,
) -> HxPartialResponse {
    let state = state.lock().expect("todo state poisoned");
    let todos = &state.todos;
    let Some(todo) = todos.get(id) else {
        return partial!(
            StatusCode::NOT_FOUND,
            todo_message(Message::Error("That todo is gone.")),
        );
    };

    partial!(todo_item_edit_form(todo))
}

/// The item and a message.
#[action("/todos/save")]
struct SaveTodo {
    id: usize,
    title: String,
    #[serde(default)]
    done: bool,
    priority: Priority,
}

async fn save(
    State(state): State<Todos>,
    SaveTodo {
        id,
        title,
        done,
        priority,
    }: SaveTodo,
) -> HxPartialResponse {
    if title.trim().is_empty() {
        return partial! {
            StatusCode::UNPROCESSABLE_ENTITY,
            todo_message(Message::Error("A todo needs a title.")),
        };
    }

    let mut state = state.lock().expect("todo state poisoned");
    let TodoState { todos, search } = &mut *state;
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
            todo_count(todos),
            todo_message(Message::Info("Todo updated.")),
        };
    }

    partial! {
        todo_item(todo),
        todo_count(todos),
        todo_message(Message::Info("Todo updated.")),
    }
}

/// The item, unchanged.
#[action("/todos/cancel")]
struct CancelEdit {
    id: usize,
}

async fn cancel(
    State(state): State<Todos>,
    CancelEdit { id }: CancelEdit,
) -> HxPartialResponse {
    let state = state.lock().expect("todo state poisoned");
    let todos = &state.todos;
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
        li id=(Self) data-done=(todo.done.to_string()) {
            label {
                input type="checkbox" checked[todo.done]
                    hx-action=(ToggleTodo::action().id(todo.id));
                span { (todo.title) }
            }
            small { (todo.priority.as_ref()) }
            button type="button"
                hx-action=(EditTodo::action().id(todo.id)) { "Edit" }
        }
    }
}

#[fragment(TodoItemId(todo.id))]
fn todo_item_edit_form(todo: &Todo) -> Markup {
    html! {
        li id=(Self) {
            form hx-action=(SaveTodo::action().id(todo.id)) {
                input type="checkbox" name=(SaveTodo::FIELD.done)
                    value="true" checked[todo.done] aria-label="Done";
                input name=(SaveTodo::FIELD.title) value=(todo.title)
                    required autofocus aria-label="Todo title" maxlength="160";
                (priority_select(SaveTodo::FIELD.priority, todo.priority))
                button type="submit" { "Save" }
                button type="button"
                    hx-action=(CancelEdit::action().id(todo.id)) { "Cancel" }
            }
        }
    }
}

#[fragment]
fn add_todo_form() -> Markup {
    html! {
        form id=(Self) class="todo-add" hx-action=(AddTodo::action()) {
            input name=(AddTodo::FIELD.title) placeholder="What needs doing?"
                aria-label="New todo" maxlength="160";
            (priority_select(AddTodo::FIELD.priority, Priority::default()))
            button type="submit" { "Add" }
        }
    }
}

fn priority_select(name: &str, selected: Priority) -> Markup {
    html! {
        select name=(name) aria-label="Priority" {
            @for priority in Priority::iter() {
                option value=(priority.as_ref())
                    selected[priority == selected]
                {
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
        form class="todo-search" hx-action=(action) {
            input type="search" name=(SearchTodos::FIELD.q) value=(search.q)
                placeholder="Search" aria-label="Search todos";
            fieldset {
                legend class="preview-sr-only" { "Filter todos" }
                @for filter in TodoFilter::iter() {
                    label {
                        input type="radio" name=(SearchTodos::FIELD.filter)
                            value=(filter.as_ref())
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
                p id=(Self) role="status" {
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
        ul id=(Self) class="todo-list" {
            @for todo in todos.iter().filter(|todo| search.matches(todo)) {
                (todo_item(todo))
            }
        }
    }
}
