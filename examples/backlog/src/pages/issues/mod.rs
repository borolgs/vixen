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

/// The page's actions for the root skill in `main.rs`.
pub fn actions() -> String {
    vixen::skill![
        list::IssueSearch,
        edit::IssueAdd,
        edit::IssueTitle,
        edit::IssueStatus,
        edit::IssuePriority,
        edit::IssueEstimate,
        edit::IssueDue,
        edit::IssuesMove,
        edit::IssuesTrash,
        edit::IssuesRestore,
    ]
}
