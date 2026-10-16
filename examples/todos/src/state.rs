use axum::extract::{FromRef, FromRequestParts, State};
use vixen::ReqCtx;

use crate::model::Todos;

#[derive(FromRef, Clone)]
pub struct AppState {
    pub todos: Todos,
}

#[derive(Clone, FromRequestParts, ReqCtx)]
#[from_request(state(AppState))]
pub struct Ctx {
    #[from_request(via(State))]
    pub todos: Todos,
}

pub fn ctx() -> Ctx {
    Ctx::current()
}
