use maud::{Markup, html};

use crate::{HxAction, Paged, PagedAction};

impl<PAction: PagedAction, Item> Paged<PAction, Item> {
    /// Configures the list as a Basecoat `<tbody>` with `COLS` columns.
    ///
    /// The built-in empty, failed, loading and retry rows span all columns.
    /// Configure any of them after `table` to override its default.
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

#[cfg(test)]
mod tests {
    use maud::html;

    use crate::{Page, Paged, action};

    #[action("/todos/search")]
    struct SearchTodos {
        #[cursor]
        after: Option<u32>,
    }

    const TODOS: Paged<SearchTodos, &str> = Paged::new(
        "todos",
        |title, _| html! { tr { td { (title) } td { "Open" } } },
    )
    .table::<2>();

    fn search(after: Option<u32>) -> SearchTodos {
        SearchTodos { after }
    }

    fn page(items: &[&'static str], next: Option<u32>) -> Page<&'static str, u32> {
        Page {
            items: items.to_vec(),
            next,
        }
    }

    #[test]
    fn a_first_page_renders_a_tbody_with_a_loading_row() {
        let html = TODOS
            .render(&search(None), page(&["milk"], Some(2)))
            .into_string();
        assert_eq!(
            html,
            concat!(
                r#"<tbody id="todos"><tr><td>milk</td><td>Open</td></tr>"#,
                r#"<tr data-paged="loading" hx-action="/todos/search" "#,
                r#"hx-vals="{&quot;after&quot;:2}" "#,
                r#"hx-trigger="intersect once" hx-swap="outerHTML" hx-method="post">"#,
                r#"<td colspan="2">Loading…</td></tr></tbody>"#,
            )
        );
    }

    #[test]
    fn first_page_states_span_all_columns() {
        assert_eq!(
            TODOS.render(&search(None), page(&[], None)).into_string(),
            r#"<tbody id="todos"><tr data-paged="empty"><td colspan="2">No results.</td></tr></tbody>"#
        );
        assert_eq!(
            TODOS.view(&search(None), Err(())).into_string(),
            concat!(
                r#"<tbody id="todos"><tr data-paged="failed"><td colspan="2">"#,
                r#"<div class="alert" data-variant="destructive">"#,
                r#"<h3>The results did not load.</h3></div></td></tr></tbody>"#,
            )
        );
    }

    #[test]
    fn a_failed_following_page_renders_the_retry_row() {
        let html = TODOS.view(&search(Some(2)), Err(())).into_string();
        assert_eq!(
            html,
            concat!(
                r#"<tr data-paged="retry" hx-action="/todos/search" "#,
                r#"hx-vals="{&quot;after&quot;:2}" "#,
                r#"hx-trigger="click from:'find button'" hx-swap="outerHTML" hx-method="post">"#,
                r#"<td colspan="2">The rest did not load. "#,
                r#"<button class="btn" type="button" data-variant="ghost" data-size="sm">"#,
                r#"Try again</button></td></tr>"#,
            )
        );
    }
}
