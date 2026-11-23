use std::sync::Arc;

use axum::{
    Router,
    extract::{FromRequestParts, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, EnumIter, IntoEnumIterator};
use tokio::sync::Mutex;
use vixen::{
    HxPartialResponse, Part, ReqCtx, RouterExt, SyncStrategy, action, fragment,
    id,
    maud::{Markup, html},
    partial, route,
};

pub fn router() -> Router<Todos> {
    Router::new()
        .view(view)
        .action(search)
        .action(add)
        .action(toggle)
        .action(edit)
        .action(save)
        .action(cancel)
}

#[route("/todos")]
pub struct Index;

pub async fn view(_: Index) -> Markup {
    let state = ctx().todos.lock_owned().await;
    html! {
        div class="todo-status" {
            (todo_count(&state.todos))
            (MessageId::slot())
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

async fn search(search: SearchTodos) -> HxPartialResponse {
    let mut state = ctx().todos.lock_owned().await;
    state.search = search;

    partial!(todo_list(&state.todos, &state.search))
}

/// The new todo's item, the count and a message.
#[action("/todos/add")]
struct AddTodo {
    title: String,
    priority: Priority,
}

async fn add(todo: AddTodo) -> HxPartialResponse {
    if todo.title.trim().is_empty() {
        return partial! {
            StatusCode::UNPROCESSABLE_ENTITY,
            warning("A todo needs a title."),
        };
    }

    let mut state = ctx().todos.lock_owned().await;
    let created = state.create(todo);

    partial! {
        add_todo_form(), todo_count(&state.todos),
        if state.is_visible(&created) {
            partial! {
                Part::before_end(todo_list::sel(), todo_item(&created)),
                success("Todo created.")
            }
        } else {
            partial! { warning("Todo created (hidden by filters).") }
        }
    }
}

/// The item with `done` flipped, and the count.
#[action("/todos/toggle")]
struct ToggleTodo {
    id: usize,
}

async fn toggle(toggle: ToggleTodo) -> HxPartialResponse {
    let mut state = ctx().todos.lock_owned().await;

    let Some(todo) = state.toggle(toggle) else {
        return partial!(StatusCode::NOT_FOUND, warning("That todo is gone."),);
    };

    partial! {
        if state.is_visible(&todo) {
            todo_item(&todo).into()
        } else {
            Part::delete(TodoItemId(todo.id))
        },
        todo_count(&state.todos),
    }
}

/// The item as an edit form posting `/todos/save`.
#[action("/todos/edit")]
struct EditTodo {
    id: usize,
}

async fn edit(EditTodo { id }: EditTodo) -> HxPartialResponse {
    let state = ctx().todos.lock_owned().await;
    let todos = &state.todos;
    let Some(todo) = todos.get(id) else {
        return partial!(StatusCode::NOT_FOUND, warning("That todo is gone."),);
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

async fn save(save: SaveTodo) -> HxPartialResponse {
    if save.title.trim().is_empty() {
        return partial! {
            StatusCode::UNPROCESSABLE_ENTITY,
            warning("A todo needs a title."),
        };
    }

    let mut state = ctx().todos.lock_owned().await;
    let Some(todo) = state.update(save) else {
        return partial!(StatusCode::NOT_FOUND, warning("That todo is gone."),);
    };

    partial! {
        if state.is_visible(&todo) {
            todo_item(&todo).into()
        } else {
            Part::delete(TodoItemId(todo.id))
        },
        todo_count(&state.todos),
        success("Todo updated."),
    }
}

/// The item, unchanged.
#[action("/todos/cancel")]
struct CancelEdit {
    id: usize,
}

async fn cancel(CancelEdit { id }: CancelEdit) -> HxPartialResponse {
    let state = ctx().todos.lock_owned().await;
    let todos = &state.todos;
    let Some(todo) = todos.get(id) else {
        return partial!(StatusCode::NOT_FOUND, warning("That todo is gone."),);
    };

    partial!(todo_item(todo))
}

// state

#[derive(Clone, FromRequestParts, ReqCtx)]
#[from_request(state(Todos))]
pub(super) struct Ctx {
    #[from_request(via(State))]
    pub todos: Todos,
}

fn ctx() -> Ctx {
    Ctx::current()
}

// model

pub type Todos = Arc<Mutex<TodoState>>;

pub struct TodoState {
    todos: Vec<Todo>,
    search: SearchTodos,
}

impl TodoState {
    fn create(&mut self, todo: AddTodo) -> Todo {
        let todo = Todo {
            id: self.todos.len(),
            title: todo.title,
            done: false,
            priority: todo.priority,
        };
        self.todos.push(todo.clone());

        todo
    }

    fn update(&mut self, edit: SaveTodo) -> Option<Todo> {
        let Some(todo) = self.todos.get_mut(edit.id) else {
            return None;
        };

        todo.done = edit.done;
        todo.title = edit.title;
        todo.priority = edit.priority;

        Some(todo.to_owned())
    }

    fn toggle(&mut self, edit: ToggleTodo) -> Option<Todo> {
        let Some(todo) = self.todos.get_mut(edit.id) else {
            return None;
        };

        todo.done = !todo.done;

        Some(todo.to_owned())
    }

    fn is_visible(&self, todo: &Todo) -> bool {
        self.search.matches(todo)
    }
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

// ui

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
        form hx-action=(AddTodo::action()) id=(Self) class="todo-add" {
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

#[id]
struct MessageId;

#[fragment(MessageId)]
fn success(text: impl Into<String>) -> Markup {
    html! { p id=(Self) role="status" {(text.into())} }
}

#[fragment(MessageId)]
fn warning(text: impl Into<String>) -> Markup {
    html! { p id=(Self) role="alert" {(text.into())} }
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
