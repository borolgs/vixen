use anyhow::Context;
use axum::{Router, response::Response};
use vixen::{
    HxCurrentQuery, HxPartialResponse, Part, RouterExt, action,
    hx::SwapOption,
    maud::html,
    partial,
    ui::basecoatui::{Action, Category, Duration, Toast},
};

use super::{
    model::{Change, Priority, Status},
    queries,
    ui::{ISSUES, IssueRowId, IssueRowsId, add_issue_row, issue_row, issue_stats},
};
use crate::{
    pages::issues::list::IssueSearch,
    shared::TOASTER,
    state::{AppState, ctx},
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .action(issue_add)
        .action(issue_title)
        .action(issue_status)
        .action(issue_priority)
        .action(issue_estimate)
        .action(issue_due)
        .action(issues_move)
        .action(issues_trash)
        .action(issues_restore)
}

#[action("/issues/add")]
pub struct IssueAdd {
    pub title: String,
}

async fn issue_add(
    HxCurrentQuery(search): HxCurrentQuery<IssueSearch>,
    IssueAdd { title }: IssueAdd,
) -> HxPartialResponse {
    let status = search.status.unwrap_or(Status::Todo);
    let title = title.trim().to_owned();
    if title.is_empty() {
        return TOASTER.error("Not added", "An issue needs a title.").into();
    }

    let added = ctx()
        .db
        .call(move |conn| {
            let id = queries::insert_issue(conn, &title, status)?;
            let issue = queries::issue_by_id(conn, id)?.context("the issue just inserted")?;

            Ok((issue, queries::stats(conn)?))
        })
        .await;

    match added {
        Ok((issue, stats)) => partial!(
            add_issue_row(),
            (IssueRowsId, SwapOption::AfterBegin) => issue_row(&issue).into(),
            issue_stats(&stats),
        ),
        Err(err) => failed("add issue", err),
    }
}

#[action("/issues/title")]
pub struct IssueTitle {
    pub id: i64,
    pub title: String,
}

async fn issue_title(
    HxCurrentQuery(search): HxCurrentQuery<IssueSearch>,
    IssueTitle { id, title }: IssueTitle,
) -> HxPartialResponse {
    let change = match title.trim() {
        "" => Err("An issue needs a title."),
        title => Ok(Change::Title(title.to_owned())),
    };
    let rejected = change.as_ref().err().copied();

    let res = ctx()
        .db
        .call(move |conn| {
            if let Ok(change) = &change {
                queries::update_issue(conn, id, change)?;
            }
            Ok((queries::issue_by_id(conn, id)?, queries::stats(conn)?))
        })
        .await;

    let (issue, stats) = match res {
        Ok(res) => res,
        Err(err) => return failed("save issue", err),
    };

    let stats = issue_stats(&stats);

    let Some(issue) = issue else {
        return partial! {
            (IssueRowId(id), SwapOption::Delete) => html! {},
            stats,
            TOASTER.info("Already deleted", "Someone got here first."),
        };
    };

    // The row from the database puts the cell back.
    if let Some(why) = rejected {
        return partial! {
            issue_row(&issue),
            stats,
            TOASTER.error("Not saved", why),
        };
    }

    if !search.lists(&issue) {
        return partial! {
            (IssueRowId(id), SwapOption::Delete) => html! {},
            stats,
        };
    }

    partial! {
        issue_row(&issue),
        stats
    }
}

#[action("/issues/status")]
pub struct IssueStatus {
    pub id: i64,
    pub status: Status,
}

async fn issue_status(
    search: Result<HxCurrentQuery<IssueSearch>, Response>,
    IssueStatus { id, status }: IssueStatus,
) -> HxPartialResponse {
    let search = search
        .map(|HxCurrentQuery(search)| search)
        .unwrap_or_default();

    let res = ctx()
        .db
        .call(move |conn| {
            queries::update_issue(conn, id, &Change::Status(status))?;
            Ok((queries::issue_by_id(conn, id)?, queries::stats(conn)?))
        })
        .await;

    let (issue, stats) = match res {
        Ok(res) => res,
        Err(err) => return failed("save issue", err),
    };

    let stats = issue_stats(&stats);

    let Some(issue) = issue else {
        return partial! {
            (IssueRowId(id), SwapOption::Delete) => html! {},
            stats,
            TOASTER.info("Already deleted", "Someone got here first."),
        };
    };

    if !search.lists(&issue) {
        return partial! {
            (IssueRowId(id), SwapOption::Delete) => html! {},
            stats,
        };
    }

    partial! {
        issue_row(&issue),
        stats
    }
}

#[action("/issues/priority")]
pub struct IssuePriority {
    pub id: i64,
    pub priority: Priority,
}

async fn issue_priority(IssuePriority { id, priority }: IssuePriority) -> HxPartialResponse {
    let res = ctx()
        .db
        .call(move |conn| {
            queries::update_issue(conn, id, &Change::Priority(priority))?;
            Ok((queries::issue_by_id(conn, id)?, queries::stats(conn)?))
        })
        .await;

    let (issue, stats) = match res {
        Ok(res) => res,
        Err(err) => return failed("save issue", err),
    };

    let stats = issue_stats(&stats);

    let Some(issue) = issue else {
        return partial! {
            (IssueRowId(id), SwapOption::Delete) => html! {},
            stats,
            TOASTER.info("Already deleted", "Someone got here first."),
        };
    };

    partial! {
        issue_row(&issue),
        stats
    }
}

#[action("/issues/estimate")]
pub struct IssueEstimate {
    pub id: i64,
    pub estimate: Option<String>,
}

async fn issue_estimate(IssueEstimate { id, estimate }: IssueEstimate) -> HxPartialResponse {
    let change = match estimate.as_deref().map(str::parse) {
        None => Ok(Change::Estimate(None)),
        Some(Ok(points @ 0..=99)) => Ok(Change::Estimate(Some(points))),
        Some(_) => Err("An estimate is a whole number up to 99."),
    };
    let rejected = change.as_ref().err().copied();

    let res = ctx()
        .db
        .call(move |conn| {
            if let Ok(change) = &change {
                queries::update_issue(conn, id, change)?;
            }
            Ok((queries::issue_by_id(conn, id)?, queries::stats(conn)?))
        })
        .await;

    let (issue, stats) = match res {
        Ok(res) => res,
        Err(err) => return failed("save issue", err),
    };

    let stats = issue_stats(&stats);

    let Some(issue) = issue else {
        return partial! {
            (IssueRowId(id), SwapOption::Delete) => html! {},
            stats,
            TOASTER.info("Already deleted", "Someone got here first."),
        };
    };

    // The row from the database puts the cell back.
    if let Some(why) = rejected {
        return partial! {
            issue_row(&issue),
            stats,
            TOASTER.error("Not saved", why),
        };
    }

    partial! {
        issue_row(&issue),
        stats
    }
}

#[action("/issues/due")]
pub struct IssueDue {
    pub id: i64,
    pub due: Option<String>,
}

async fn issue_due(IssueDue { id, due }: IssueDue) -> HxPartialResponse {
    let res = ctx()
        .db
        .call(move |conn| {
            queries::update_issue(conn, id, &Change::Due(due))?;
            Ok((queries::issue_by_id(conn, id)?, queries::stats(conn)?))
        })
        .await;

    let (issue, stats) = match res {
        Ok(res) => res,
        Err(err) => return failed("save issue", err),
    };

    let stats = issue_stats(&stats);

    let Some(issue) = issue else {
        return partial! {
            (IssueRowId(id), SwapOption::Delete) => html! {},
            stats,
            TOASTER.info("Already deleted", "Someone got here first."),
        };
    };

    partial! {
        issue_row(&issue),
        stats
    }
}

#[action("/issues/move")]
pub struct IssuesMove {
    #[serde(default)]
    pub ids: Vec<i64>,
    pub status: Status,
}

async fn issues_move(
    search: Result<HxCurrentQuery<IssueSearch>, Response>,
    IssuesMove { ids, status }: IssuesMove,
) -> HxPartialResponse {
    let search = search
        .map(|HxCurrentQuery(search)| search)
        .unwrap_or_default();

    let moved = ctx()
        .db
        .call(move |conn| {
            queries::move_issues(conn, &ids, status)?;

            Ok((queries::issues_by_ids(conn, &ids)?, queries::stats(conn)?))
        })
        .await;

    let (issues, stats) = match moved {
        Ok(moved) => moved,
        Err(err) => {
            return failed("move issues", err);
        }
    };

    let rows: Vec<Part> = issues
        .iter()
        .map(|issue| {
            if !search.lists(issue) {
                return Part::new(IssueRowId(issue.id), html! {}).swap(SwapOption::Delete);
            }
            issue_row(issue).into()
        })
        .collect();

    partial!(
        rows,
        issue_stats(&stats),
        TOASTER.success(
            "Moved",
            format!("{} to {}.", count(issues.len()), status.label())
        ),
    )
}

/// Posted by the bulk bar with the checked rows, and by a row's own button.
#[action("/issues/trash")]
pub struct IssuesTrash {
    #[serde(default)]
    pub ids: Vec<i64>,
}

async fn issues_trash(IssuesTrash { ids }: IssuesTrash) -> HxPartialResponse {
    let rows: Vec<Part> = ids
        .iter()
        .map(|id| Part::new(IssueRowId(*id), html! {}).swap(SwapOption::Delete))
        .collect();

    let trashed = ctx()
        .db
        .call(move |conn| Ok((queries::trash_issues(conn, &ids)?, queries::stats(conn)?)))
        .await;

    match trashed {
        Ok((batch, stats)) => {
            let undo = Toast {
                duration: Duration::Millis(8000),
                action: Some(Action::Custom(html! {
                    button.btn type="button" data-toast-action
                        hx-action=(IssuesRestore::action().batch(batch)) { "Undo" }
                })),
                ..Toast::new(
                    Category::Info,
                    "Deleted",
                    format!("{} left the backlog.", count(rows.len())),
                )
            };

            partial!(rows, issue_stats(&stats), TOASTER.toast(undo))
        }
        Err(err) => failed("delete issues", err),
    }
}

#[action("/issues/restore")]
pub struct IssuesRestore {
    pub batch: i64,
}

async fn issues_restore(IssuesRestore { batch }: IssuesRestore) -> HxPartialResponse {
    let restored = ctx()
        .db
        .call(move |conn| {
            queries::restore_issues(conn, batch)?;

            queries::stats(conn)
        })
        .await;

    match restored {
        Ok(stats) => partial!(issue_stats(&stats), ISSUES.refresh()),
        Err(err) => failed("restore issues", err),
    }
}

fn failed(what: &str, err: anyhow::Error) -> HxPartialResponse {
    tracing::error!("{what}: {err:#}");

    TOASTER
        .error("That didn't go through", "Try again in a moment.")
        .into()
}

fn count(issues: usize) -> String {
    match issues {
        1 => "1 issue".to_owned(),
        n => format!("{n} issues"),
    }
}
