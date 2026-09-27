use axum::extract::FromRef;

use crate::pages::todos::Todos;

#[derive(FromRef, Clone)]
pub struct AppState {
    pub todos: Todos,
}
