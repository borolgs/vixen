use axum::extract::{FromRef, FromRequestParts, State};
use vixen::ReqCtx;

use crate::db::Db;

#[derive(FromRef, Clone)]
pub struct AppState {
    pub db: Db,
}

#[derive(Debug, Clone, FromRequestParts, ReqCtx)]
#[from_request(state(AppState))]
pub struct Ctx {
    #[from_request(via(State))]
    pub db: Db,
}

pub fn ctx() -> Ctx {
    Ctx::current()
}
