use axum::response::IntoResponse;
use vixen::{After, Page, action, href, partial};

use super::{
    model::{Issue, SortField, Status},
    page::BacklogPath,
    queries::search_issues,
    ui::ISSUES,
};
use crate::{shared::TOASTER, sort::SortDir, state::ctx};

#[derive(Clone, Default, Debug)]
#[action("/issues/search")]
pub struct IssueSearch {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub q: String,
    pub status: Option<Status>,
    #[serde(default)]
    pub sort: SortField,
    #[serde(default)]
    pub dir: SortDir,
    #[cursor]
    pub after: Option<After<i64>>,
}

impl IssueSearch {
    /// The filters of `search_issues`, for a row that was just edited.
    pub fn lists(&self, issue: &Issue) -> bool {
        let q = self.q.trim().to_ascii_lowercase();

        self.status.is_none_or(|status| status == issue.status)
            && issue.title.to_ascii_lowercase().contains(&q)
    }
}

pub async fn issue_search(search: IssueSearch) -> impl IntoResponse {
    let page = load(&search).await;
    let toast = page
        .is_err()
        .then(|| TOASTER.error("That didn't go through", "Try again in a moment."));

    (
        ISSUES.replace_url(&search, href!(BacklogPath)),
        partial!(_ => ISSUES.view(&search, page), toast),
    )
}

pub async fn load(search: &IssueSearch) -> anyhow::Result<Page<Issue, After<i64>>> {
    let search = search.clone();

    ctx()
        .db
        .call(move |conn| search_issues(conn, &search))
        .await
        .inspect_err(|err| tracing::error!("search issues: {err:#}"))
}
