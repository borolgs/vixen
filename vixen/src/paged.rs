//! Helpers for cursor-paginated search results.
//!
//! [`Paged`] renders initial results, appends subsequent pages, and renders
//! failures in place. The request type implements [`PagedAction`] to expose its
//! cursor and build actions for subsequent pages.

use std::{fmt::Display, str::FromStr};

use axum_htmx::{HxEvent, HxReplaceUrl, SwapOption};
use maud::{Markup, html};
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error};
use serde_json::json;

use crate::{Href, HxAction, Selector, SyncStrategy};

// Radio inputs emit both `input` and `change`; text inputs emit `change` on blur.
const SEARCH_TRIGGER: &str =
    "submit, change[target.type!='search'], input[target.type=='search'] delay:300ms";
const RETRY_TRIGGER: &str = "click from:'find button'";

/// A page of cursor-paginated results.
pub struct Page<Item, Cursor> {
    /// Results in this page.
    pub items: Vec<Item>,
    /// Cursor for the following page, or `None` if this is the last page.
    pub next: Option<Cursor>,
}

impl<Item, Cursor> Page<Item, Cursor> {
    /// Builds a page from rows fetched with `LIMIT limit + 1`.
    ///
    /// If the extra row is present, it is removed and `cursor` builds
    /// [`next`](Self::next) from the last retained row.
    ///
    /// # Panics
    ///
    /// Panics if `limit` is zero.
    pub fn from_rows(
        mut rows: Vec<Item>,
        limit: usize,
        cursor: impl FnOnce(&Item) -> Cursor,
    ) -> Self {
        assert!(limit > 0, "a page limit must not be zero");

        let next = if rows.len() > limit {
            rows.truncate(limit);
            rows.last().map(cursor)
        } else {
            None
        };

        Self { items: rows, next }
    }

    /// Maps the items without changing the cursor.
    pub fn map<T>(self, f: impl FnMut(Item) -> T) -> Page<T, Cursor> {
        Page {
            items: self.items.into_iter().map(f).collect(),
            next: self.next,
        }
    }
}

/// A keyset cursor containing one sort key and a row ID.
///
/// Serialized as `{id}:{key}`. The ID must not contain `:`. Use a custom
/// [`PagedAction::Cursor`] for more than one sort key.
///
/// Example query for [`Page::from_rows`]:
///
/// ```sql
/// select id, title from todos
///     where :after_id is null or (title, id) > (:after_key, :after_id)
///     order by title, id
///     limit :limit + 1
/// ```
///
/// ```
/// use vixen::{After, Page};
///
/// # struct Todo { id: i64, title: String }
/// # let rows: Vec<Todo> = Vec::new();
/// let page = Page::from_rows(rows, 20, |todo| After {
///     id: todo.id,
///     key: todo.title.clone(),
/// });
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct After<I> {
    /// The row ID, used to break ties between equal keys.
    pub id: I,
    /// The value of the sort column.
    pub key: String,
}

impl<I: Display> Serialize for After<I> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(&format_args!("{}:{}", self.id, self.key))
    }
}

impl<'de, I: FromStr> Deserialize<'de> for After<I> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let cursor = String::deserialize(deserializer)?;

        cursor
            .split_once(':')
            .and_then(|(id, key)| {
                Some(After {
                    id: id.parse().ok()?,
                    key: key.to_owned(),
                })
            })
            .ok_or_else(|| D::Error::custom("not a cursor"))
    }
}

/// An action that fetches pages for a [`Paged`] list.
///
/// An [`#[action]`](macro@crate::action) request implements this when one
/// `Option<_>` field is marked `#[cursor]`.
pub trait PagedAction {
    /// The cursor passed between consecutive requests.
    type Cursor;

    /// Returns `None` for the initial page.
    fn cursor(&self) -> Option<Self::Cursor>;

    /// Builds an action that continues the request from `cursor`.
    fn next(&self, cursor: Self::Cursor) -> HxAction;
}

/// Search results that load the next page when scrolled into view.
///
/// Declare a `Paged` as a `const`. [`search`](Self::search) configures its form
/// action, and [`view`](Self::view) renders a loaded or failed page, both on
/// the index and in the search handler.
///
/// ```
/// use vixen::{
///     Page, Paged, action,
///     maud::{Markup, html},
/// };
///
/// #[action("/todos/search")]
/// struct SearchTodos {
///     q: String,
///     #[cursor]
///     after: Option<u32>,
/// }
///
/// const TODOS: Paged<SearchTodos, String> =
///     Paged::new("todos", |title| html! { div { (title) } });
///
/// async fn index(search: SearchTodos) -> Markup {
///     html! {
///         form hx-action=(TODOS.search(SearchTodos::action())) {
///             input type="search" name=(SearchTodos::FIELD.q);
///         }
///         (TODOS.view(&search, load(&search).await))
///     }
/// }
///
/// async fn search(search: SearchTodos) -> Markup {
///     TODOS.view(&search, load(&search).await)
/// }
/// # async fn load(_: &SearchTodos) -> std::io::Result<Page<String, u32>> {
/// #     Ok(Page { items: Vec::new(), next: None })
/// # }
/// ```
///
/// The first page includes the list container; following pages render only new
/// items and, when present, another loading sentinel. The defaults use `div`s.
/// Use [`list`](Self::list), [`empty`](Self::empty), [`failed`](Self::failed),
/// [`loading`](Self::loading) and [`retry`](Self::retry) to customize them.
/// With the `basecoatui` feature, `Paged::table` configures a `<tbody>`.
/// [`replace_url`](Self::replace_url) adds the search parameters to the browser URL.
/// Use [`After`] and [`Page::from_rows`] for single-column keyset pagination.
pub struct Paged<PAction, Item> {
    id: &'static str,
    list: fn(&'static str, &PAction, Markup) -> Markup,
    item: fn(&Item) -> Markup,
    empty: fn(&PAction) -> Markup,
    failed: fn(&PAction) -> Markup,
    loading: fn(HxAction) -> Markup,
    retry: fn(HxAction) -> Markup,
    search_trigger: &'static str,
    retry_trigger: &'static str,
}

impl<PAction: PagedAction, Item> Paged<PAction, Item> {
    /// Creates a paged list with a container ID and item renderer.
    pub const fn new(id: &'static str, item: fn(&Item) -> Markup) -> Self {
        Self {
            id,
            list: |id, _, rows| html! { div id=(id) { (rows) } },
            item,
            empty: |_| html! { div { "No results." } },
            failed: |_| html! { div { "The results did not load." } },
            loading: |next| html! { div hx-action=(next) { "Loading…" } },
            retry: |again| {
                html! {
                    div hx-action=(again) {
                        "The rest did not load. "
                        button type="button" { "Try again" }
                    }
                }
            },
            search_trigger: SEARCH_TRIGGER,
            retry_trigger: RETRY_TRIGGER,
        }
    }

    /// Sets the list container renderer.
    ///
    /// The root element must use the provided ID and contain the rows. The current
    /// search is available when rendering the initial page.
    pub const fn list(mut self, list: fn(&'static str, &PAction, Markup) -> Markup) -> Self {
        self.list = list;
        self
    }

    /// Sets the renderer for an initial page with no items.
    pub const fn empty(mut self, empty: fn(&PAction) -> Markup) -> Self {
        self.empty = empty;
        self
    }

    /// Sets the renderer for an initial page that failed to load.
    pub const fn failed(mut self, failed: fn(&PAction) -> Markup) -> Self {
        self.failed = failed;
        self
    }

    /// Sets the loading sentinel renderer.
    ///
    /// The returned markup must use the provided action as its `hx-action`.
    pub const fn loading(mut self, loading: fn(HxAction) -> Markup) -> Self {
        self.loading = loading;
        self
    }

    /// Sets the retry sentinel renderer.
    ///
    /// The returned markup must contain a button and use the provided action as its
    /// `hx-action`.
    pub const fn retry(mut self, retry: fn(HxAction) -> Markup) -> Self {
        self.retry = retry;
        self
    }

    /// Overrides the trigger used by search forms.
    pub const fn search_trigger(mut self, trigger: &'static str) -> Self {
        self.search_trigger = trigger;
        self
    }

    /// Overrides the trigger used by retry sentinels.
    pub const fn retry_trigger(mut self, trigger: &'static str) -> Self {
        self.retry_trigger = trigger;
        self
    }

    /// Configures an action for a search form.
    ///
    /// With the default trigger, text queries must use `input type="search"`.
    /// The form also searches again on [`refresh`](Self::refresh).
    pub fn search(&self, action: impl Into<HxAction>) -> HxAction {
        action
            .into()
            .trigger(format!(
                "{}, {} from:document",
                self.search_trigger,
                self.refresh_event()
            ))
            .sync(SyncStrategy::Replace)
            .target(self.sel())
            .swap(SwapOption::OuterHtml)
    }

    /// Returns an htmx event that repeats the form's search from the first page.
    pub fn refresh(&self) -> HxEvent {
        HxEvent {
            name: self.refresh_event(),
            data: Some(json!({})),
        }
    }

    /// Renders the complete list for an initial page or rows for a subsequent page.
    pub fn render(&self, search: &PAction, page: Page<Item, PAction::Cursor>) -> Markup {
        let rows = html! {
            @for item in &page.items { ((self.item)(item)) }
            @if let Some(next) = page.next { ((self.loading)(sentinel(search.next(next)))) }
        };

        if search.cursor().is_some() {
            return rows;
        }

        self.shell(
            search,
            html! {
                @if page.items.is_empty() { ((self.empty)(search)) }
                (rows)
            },
        )
    }

    /// Renders a page or its failure, on the index and in the search handler.
    ///
    /// An initial-page error renders [`failed`](Self::failed) inside the list. A
    /// subsequent-page error replaces the loading sentinel with retry markup.
    pub fn view<E>(
        &self,
        search: &PAction,
        page: Result<Page<Item, PAction::Cursor>, E>,
    ) -> Markup {
        match (page, search.cursor()) {
            (Ok(page), _) => self.render(search, page),
            (Err(_), Some(cursor)) => (self.retry)(retry(search.next(cursor), self.retry_trigger)),
            (Err(_), None) => self.shell(search, (self.failed)(search)),
        }
    }

    /// Creates an `HX-Replace-Url` response header for the initial page.
    ///
    /// The URL is `href` with the search fields encoded as query parameters.
    /// Returns `None` for subsequent pages or when serialization fails.
    pub fn replace_url(&self, search: &PAction, href: Href<impl Display>) -> Option<HxReplaceUrl>
    where
        PAction: Serialize,
    {
        if search.cursor().is_some() {
            return None;
        }
        let query = serde_html_form::to_string(search).ok()?;
        Some(HxReplaceUrl(if query.is_empty() {
            href.to_string()
        } else {
            format!("{href}?{query}")
        }))
    }

    fn shell(&self, search: &PAction, rows: Markup) -> Markup {
        (self.list)(self.id, search, rows)
    }

    fn sel(&self) -> Selector {
        Selector(format!("#{}", self.id))
    }

    fn refresh_event(&self) -> String {
        format!("{}:refresh", self.id)
    }
}

impl<PAction: PagedAction, Item> From<Paged<PAction, Item>> for Selector {
    fn from(paged: Paged<PAction, Item>) -> Self {
        paged.sel()
    }
}

fn sentinel(next: HxAction) -> HxAction {
    next.trigger("intersect once").swap(SwapOption::OuterHtml)
}

fn retry(next: HxAction, trigger: &str) -> HxAction {
    next.trigger(trigger).swap(SwapOption::OuterHtml)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action;

    #[action("/todos/search")]
    struct SearchTodos {
        q: String,
        #[cursor]
        after: Option<u32>,
    }

    const TODOS: Paged<SearchTodos, &str> = Paged::new("todos", |title| html! { p { (title) } });

    fn search(after: Option<u32>) -> SearchTodos {
        SearchTodos {
            q: "m".into(),
            after,
        }
    }

    fn page(items: &[&'static str], next: Option<u32>) -> Page<&'static str, u32> {
        Page {
            items: items.to_vec(),
            next,
        }
    }

    #[test]
    fn a_first_page_renders_the_list_with_a_sentinel() {
        let html = TODOS
            .render(&search(None), page(&["milk", "mint"], Some(2)))
            .into_string();
        assert_eq!(
            html,
            concat!(
                r#"<div id="todos"><p>milk</p><p>mint</p>"#,
                r#"<div hx-action="/todos/search" "#,
                r#"hx-vals="{&quot;after&quot;:2,&quot;q&quot;:&quot;m&quot;}" "#,
                r#"hx-trigger="intersect once" hx-swap="outerHTML" hx-method="post">Loading…</div>"#,
                "</div>",
            )
        );
    }

    #[test]
    fn a_following_page_renders_rows_only() {
        let html = TODOS
            .render(&search(Some(2)), page(&["plum"], None))
            .into_string();
        assert_eq!(html, "<p>plum</p>");
    }

    #[test]
    fn an_empty_first_page_renders_the_empty_slot() {
        let html = TODOS.render(&search(None), page(&[], None)).into_string();
        assert_eq!(html, r#"<div id="todos"><div>No results.</div></div>"#);
    }

    #[test]
    fn the_list_reads_the_search() {
        let todos =
            TODOS.list(|id, search, rows| html! { ul id=(id) data-q=(search.q) { (rows) } });
        let html = todos
            .render(&search(None), page(&["milk"], None))
            .into_string();
        assert_eq!(html, r#"<ul id="todos" data-q="m"><p>milk</p></ul>"#);
    }

    #[test]
    fn search_replaces_the_list_and_listens_for_refresh() {
        let todos = TODOS.search_trigger("submit");
        let html = html! { form hx-action=(todos.search(SearchTodos::action())) {} }.into_string();
        assert_eq!(
            html,
            concat!(
                r#"<form hx-action="/todos/search" "#,
                r#"hx-trigger="submit, todos:refresh from:document" "#,
                r##"hx-target="#todos" hx-swap="outerHTML" hx-sync="replace" "##,
                r#"hx-method="post"></form>"#,
            )
        );
        assert_eq!(todos.refresh().name, "todos:refresh");
    }

    #[test]
    fn only_a_first_page_replaces_the_url() {
        let href = Href::new("/app", "/todos");

        let first = TODOS.replace_url(&search(None), href).unwrap();
        assert_eq!(first.0, "/app/todos?q=m");
        assert!(TODOS.replace_url(&search(Some(2)), href).is_none());
    }

    #[test]
    fn an_empty_search_replaces_the_url_without_a_query() {
        #[action("/todos/all")]
        struct AllTodos {
            #[cursor]
            after: Option<u32>,
        }
        let todos: Paged<AllTodos, &str> = Paged::new("todos", |_| html! {});

        let url = todos.replace_url(&AllTodos { after: None }, Href::new("", "/todos"));
        assert_eq!(url.unwrap().0, "/todos");
    }

    #[test]
    fn a_failed_following_page_becomes_a_retry_row() {
        let html = TODOS
            .retry_trigger("click")
            .view(&search(Some(2)), Err(()))
            .into_string();
        assert_eq!(
            html,
            concat!(
                r#"<div hx-action="/todos/search" "#,
                r#"hx-vals="{&quot;after&quot;:2,&quot;q&quot;:&quot;m&quot;}" "#,
                r#"hx-trigger="click" hx-swap="outerHTML" hx-method="post">"#,
                r#"The rest did not load. <button type="button">Try again</button></div>"#,
            )
        );
    }

    #[test]
    fn a_failed_first_page_renders_the_failed_slot() {
        let html = TODOS.view(&search(None), Err(())).into_string();
        assert_eq!(
            html,
            r#"<div id="todos"><div>The results did not load.</div></div>"#
        );
    }

    #[test]
    fn an_extra_row_is_dropped_and_sets_the_next_cursor() {
        let full = Page::from_rows(vec![3, 5, 8], 2, |last| *last);
        assert_eq!((full.items, full.next), (vec![3, 5], Some(5)));

        let last = Page::from_rows(vec![3, 5], 2, |last| *last);
        assert_eq!((last.items, last.next), (vec![3, 5], None));
    }

    #[test]
    #[should_panic(expected = "a page limit must not be zero")]
    fn a_zero_limit_panics() {
        Page::from_rows(vec![3], 0, |last| *last);
    }

    #[test]
    fn map_keeps_the_cursor() {
        let page = Page::from_rows(vec![(3, "milk"), (5, "mint")], 1, |(id, _)| *id)
            .map(|(_, title)| title);
        assert_eq!((page.items, page.next), (vec!["milk"], Some(3)));
    }

    #[action("/todos/after")]
    struct TodosAfter {
        #[cursor]
        after: Option<After<u32>>,
    }

    #[test]
    fn after_is_an_id_and_a_key_in_a_form() {
        let after = After {
            id: 7,
            key: "elm:x".to_owned(),
        };
        let search = TodosAfter {
            after: Some(after.clone()),
        };

        let body = serde_html_form::to_string(&search).unwrap();
        assert_eq!(body, "after=7%3Aelm%3Ax");

        let parsed: TodosAfter = serde_html_form::from_str(&body).unwrap();
        assert_eq!(parsed.cursor(), Some(after));
    }

    #[test]
    fn a_malformed_after_is_rejected() {
        for body in ["after=7", "after=x%3Aelm"] {
            assert!(serde_html_form::from_str::<TodosAfter>(body).is_err());
        }
    }
}
