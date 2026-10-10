mod edit;
mod list;
mod model;
mod page;
mod queries;
mod ui;

use axum::Router;
use vixen::RouterExt;

use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .view(page::backlog)
        .action(list::issue_search)
        .merge(edit::routes())
}
