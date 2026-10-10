use strum::IntoEnumIterator;
use vixen::{
    Paged, fragment, id,
    maud::{Markup, html},
};

use super::issue_row;
use crate::{
    pages::issues::{
        edit::{IssueAdd, IssuesMove, IssuesTrash},
        list::IssueSearch,
        model::{Issue, SortField, Status},
    },
    sort::SortHead,
};

#[id]
pub struct IssueSearchId;

#[id]
pub struct IssueRowsId;

#[id]
pub struct IssueBulkId;

pub const ISSUES: Paged<IssueSearch, Issue> = Paged::new("issues", |issue, _| {
    issue_row(issue).into()
})
.table::<8>()
.list(|id, search, rows| {
    let head = SortHead::new(id, IssueSearchId, search.sort, search.dir, |sort, dir| {
        IssueSearch::action().sort(sort).dir(dir).hx()
    });

    html! {
        table id=(id) class="table table-fixed" {
            thead {
                tr {
                    th class="w-10" {
                        (head.state())
                        input type="checkbox" class="input" data-select-all aria-label="Select all";
                    }
                    th class="w-20" { (head.button("Key", SortField::Created)) }
                    th { (head.button("Title", SortField::Title)) }
                    th class="w-36" { (head.button("Status", SortField::Status)) }
                    th class="w-28" { (head.button("Priority", SortField::Priority)) }
                    th class="w-24 text-right" { (head.button("Estimate", SortField::Estimate)) }
                    th class="w-40" { (head.button("Due", SortField::Due)) }
                    th class="w-12" {}
                }
            }
            tbody { (add_issue_row()) }
            tbody id=(IssueRowsId) { (rows) }
        }
    }
});

pub fn issue_search_form(search: &IssueSearch) -> Markup {
    let chip = "btn has-checked:bg-primary has-checked:text-primary-foreground \
        has-focus-visible:ring-ring/50 has-focus-visible:ring-[3px]";

    html! {
        form id=(IssueSearchId) class="mt-8 flex flex-wrap items-center gap-3"
            hx-action=(ISSUES.search(IssueSearch::action()))
        {
            input type="search" class="input w-full sm:w-64" autocomplete="off"
                name=(IssueSearch::FIELD.q) value=(search.q)
                placeholder="Search issues" aria-label="Search issues";
            fieldset class="flex flex-wrap gap-2" {
                legend class="sr-only" { "Status" }
                label class=(chip) data-variant="outline" data-size="sm" {
                    input type="radio" class="sr-only" name=(IssueSearch::FIELD.status)
                        value="" checked[search.status.is_none()];
                    "All"
                }
                @for status in Status::iter() {
                    label class=(chip) data-variant="outline" data-size="sm" {
                        input type="radio" class="sr-only" name=(IssueSearch::FIELD.status)
                            value=(status.as_ref()) checked[search.status == Some(status)];
                        (status.label())
                    }
                }
            }
        }
    }
}

#[fragment]
pub fn add_issue_row() -> Markup {
    html! {
        tr id=(Self) class="border-b" {
            td {}
            td class="text-muted-foreground font-mono text-xs" { "New" }
            td colspan="6" {
                form hx-action=(IssueAdd::action()) {
                    input.cell type="text" id="new-issue" required autocomplete="off"
                        name=(IssueAdd::FIELD.title)
                        placeholder="Add an issue and press Enter" aria-label="New issue";
                }
            }
        }
    }
}

/// Shown while rows are checked; the checkboxes belong to this form.
pub fn issue_bulk_bar() -> Markup {
    html! {
        form id=(IssueBulkId) class="bulk bg-background fixed inset-x-0 bottom-6 z-10 mx-auto w-fit items-center gap-2 rounded-lg border px-3 py-2 shadow-lg" {
            span class="bulk-count mr-2 text-sm font-medium" {}
            @for status in Status::iter() {
                button.btn type="button" data-variant="outline" data-size="sm"
                    hx-action=(IssuesMove::action().status(status))
                {
                    (status.label())
                }
            }
            button.btn type="button" data-variant="destructive" data-size="sm"
                hx-action=(IssuesTrash::action())
            {
                "Delete"
            }
        }
    }
}
