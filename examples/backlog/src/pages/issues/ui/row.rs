use strum::IntoEnumIterator;
use vixen::{
    fragment, id,
    maud::{Markup, PreEscaped, html},
};

use super::list::IssueBulkId;
use crate::pages::issues::{
    edit::{IssueDue, IssueEstimate, IssuePriority, IssueStatus, IssueTitle, IssuesTrash},
    model::{Issue, Priority, Status},
};

#[id]
pub struct IssueRowId(pub i64);

/// Every cell is a control that posts itself on `change`. The ids let htmx
/// keep the focus when the row is swapped.
#[fragment(IssueRowId(issue.id))]
pub fn issue_row(issue: &Issue) -> Markup {
    let id = issue.id;

    html! {
        tr id=(Self) {
            td {
                input type="checkbox" class="input" form=(IssueBulkId)
                    name=(IssuesTrash::FIELD.ids) value=(id) aria-label="Select";
            }
            td class="text-muted-foreground font-mono text-xs" { "VIX-" (id) }
            td {
                input.cell.font-medium."text-muted-foreground"[!issue.open]."line-through"[!issue.open]
                    type="text" id={ (Self) "-title" } aria-label="Title" autocomplete="off"
                    name=(IssueTitle::FIELD.title) value=(issue.title)
                    hx-action=(IssueTitle::action().id(id));
            }
            td {
                div class="flex items-center" {
                    span class={ "size-2 shrink-0 rounded-full " (issue.status.tone()) } {}
                    select.cell id={ (Self) "-status" } aria-label="Status"
                        name=(IssueStatus::FIELD.status)
                        hx-action=(IssueStatus::action().id(id))
                    {
                        @for status in Status::iter() {
                            option value=(status.as_ref()) selected[status == issue.status] {
                                (status.label())
                            }
                        }
                    }
                }
            }
            td {
                select.cell."text-destructive"[issue.priority == Priority::Urgent]
                    ."text-muted-foreground"[issue.priority == Priority::None]
                    id={ (Self) "-priority" } aria-label="Priority"
                    name=(IssuePriority::FIELD.priority)
                    hx-action=(IssuePriority::action().id(id))
                {
                    @for priority in Priority::iter() {
                        option value=(priority.as_ref()) selected[priority == issue.priority] {
                            (priority.label())
                        }
                    }
                }
            }
            td {
                input.cell.text-right.tabular-nums type="number" min="0" max="99" placeholder="—"
                    id={ (Self) "-estimate" } aria-label="Estimate"
                    name=(IssueEstimate::FIELD.estimate) value=[issue.estimate]
                    hx-action=(IssueEstimate::action().id(id));
            }
            td {
                input.cell."text-destructive"[issue.overdue] type="date"
                    id={ (Self) "-due" } aria-label="Due"
                    name=(IssueDue::FIELD.due) value=[&issue.due]
                    hx-action=(IssueDue::action().id(id));
            }
            td {
                button.btn type="button" data-variant="ghost" data-size="icon-sm" aria-label="Delete"
                    name=(IssuesTrash::FIELD.ids) value=(id) hx-action=(IssuesTrash::action())
                {
                    (TRASH)
                }
            }
        }
    }
}

const TRASH: PreEscaped<&str> = PreEscaped(
    r#"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M3 6h18"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6"/><path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/><path d="M10 11v6"/><path d="M14 11v6"/></svg>"#,
);
