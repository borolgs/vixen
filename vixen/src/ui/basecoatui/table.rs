use maud::{Markup, html};

use crate::{HxAction, Paged, PagedAction};

impl<PAction: PagedAction, Item> Paged<PAction, Item> {
    /// Renders the list as a Basecoat table body with `COLS` columns.
    pub const fn table<const COLS: usize>(self) -> Self {
        self.list(|id, _, rows| html! { tbody id=(id) { (rows) } })
            .empty(empty::<PAction, COLS>)
            .failed(failed::<PAction, COLS>)
            .loading(loading::<COLS>)
            .retry(retry::<COLS>)
    }
}

fn empty<PAction, const COLS: usize>(_: &PAction) -> Markup {
    html! {
        tr data-paged="empty" { td colspan=(COLS) { "No results." } }
    }
}

fn failed<PAction, const COLS: usize>(_: &PAction) -> Markup {
    html! {
        tr data-paged="failed" {
            td colspan=(COLS) {
                div.alert data-variant="destructive" { h3 { "The results did not load." } }
            }
        }
    }
}

fn loading<const COLS: usize>(next: HxAction) -> Markup {
    html! {
        tr data-paged="loading" hx-action=(next) { td colspan=(COLS) { "Loading…" } }
    }
}

fn retry<const COLS: usize>(again: HxAction) -> Markup {
    html! {
        tr data-paged="retry" hx-action=(again) {
            td colspan=(COLS) {
                "The rest did not load. "
                button.btn type="button" data-variant="ghost" data-size="sm" { "Try again" }
            }
        }
    }
}
