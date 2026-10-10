use axum_extra::extract::Query;
use vixen::{
    maud::{Markup, html},
    route,
};

use super::{
    list::{IssueSearch, load},
    queries,
    ui::{ISSUES, issue_bulk_bar, issue_search_form, issue_stats},
};
use crate::{shared::layout, state::ctx};

#[route("/")]
pub struct BacklogPath;

pub async fn backlog(_: BacklogPath, Query(search): Query<IssueSearch>) -> Markup {
    let search = IssueSearch {
        after: None,
        ..search
    };
    let issues = load(&search).await;
    let stats = ctx()
        .db
        .call(|conn| queries::stats(conn))
        .await
        .inspect_err(|err| tracing::error!("backlog stats: {err:#}"))
        .unwrap_or_default();

    layout(
        "Backlog",
        html! {
            // htmx 4.0.0 settles a focused input back to its old value.
            meta name="htmx-config" content=r#"{"defaultSettleDelay":0}"#;
            (vixen::assets!())
        },
        html! {
            h1 class="text-3xl font-semibold tracking-tight" { "Backlog" }
            p class="text-muted-foreground mt-3" {
                "Every cell saves as you leave it. Enter commits, Escape puts it back."
            }

            (issue_stats(&stats))
            (issue_search_form(&search))
            div class="table-container mt-4 rounded-lg border" {
                (ISSUES.view(&search, issues))
            }
            (issue_bulk_bar())
        },
    )
}
