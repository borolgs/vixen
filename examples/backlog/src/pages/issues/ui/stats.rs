use vixen::{
    fragment,
    maud::{Markup, html},
};

use crate::pages::issues::model::Stats;

#[fragment]
pub fn issue_stats(stats: &Stats) -> Markup {
    html! {
        dl id=(Self) class="mt-8 grid grid-cols-2 gap-4 sm:grid-cols-4" {
            (stat("Open", stats.open))
            (stat("In progress", stats.in_progress))
            (stat("Points left", stats.points))
            (stat("Overdue", stats.overdue))
        }
    }
}

fn stat(label: &str, value: i64) -> Markup {
    html! {
        div class="rounded-lg border px-4 py-3" {
            dt class="text-muted-foreground text-sm" { (label) }
            dd class="mt-1 text-2xl font-semibold tabular-nums" { (value) }
        }
    }
}
