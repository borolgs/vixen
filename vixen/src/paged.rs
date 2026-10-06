#![allow(missing_docs)] // TODO: document the public pagination API

//! Helpers for cursor-paginated search results.
//!
//! [`Paged`] renders initial results, appends subsequent pages, and handles retry
//! responses. The request type implements [`PagedAction`] to expose its cursor and
//! build actions for subsequent pages.

use axum::response::{IntoResponse, Response};
use axum_htmx::{HxEvent, HxReswap, SwapOption};
use maud::{Markup, html};
use serde_json::json;

use crate::{HxAction, HxPartial, Parts, Selector, SyncStrategy};

// Radio inputs emit both `input` and `change`; text inputs emit `change` on blur.
const SEARCH_TRIGGER: &str =
    "submit, change[target.type!='search'], input[target.type=='search'] delay:300ms";
const RETRY_TRIGGER: &str = "click from:'find button'";

pub struct Page<Item, Cursor> {
    pub items: Vec<Item>,
    pub next: Option<Cursor>,
}

pub trait PagedAction {
    type Cursor;

    /// Returns `None` for the initial page.
    fn cursor(&self) -> Option<Self::Cursor>;

    /// Builds an action that continues the request from `cursor`.
    fn next(&self, cursor: Self::Cursor) -> HxAction;
}

pub struct Paged<PAction, Item> {
    id: &'static str,
    list: fn(&'static str, Markup) -> Markup,
    item: fn(&Item) -> Markup,
    empty: fn(&PAction) -> Markup,
    loading: fn(HxAction) -> Markup,
    retry: fn(HxAction) -> Markup,
    search_trigger: &'static str,
    retry_trigger: &'static str,
}

impl<PAction: PagedAction, Item> Paged<PAction, Item> {
    pub const fn new(id: &'static str, item: fn(&Item) -> Markup) -> Self {
        Self {
            id,
            list: |id, rows| html! { div id=(id) { (rows) } },
            item,
            empty: |_| html! { div { "No results." } },
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

    pub const fn list(mut self, list: fn(&'static str, Markup) -> Markup) -> Self {
        self.list = list;
        self
    }

    pub const fn empty(mut self, empty: fn(&PAction) -> Markup) -> Self {
        self.empty = empty;
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

        self.shell(html! {
            @if page.items.is_empty() { ((self.empty)(search)) }
            (rows)
        })
    }

    /// Wraps `rows` in the list container targeted by search actions.
    pub fn shell(&self, rows: Markup) -> Markup {
        (self.list)(self.id, rows)
    }

    /// Converts a page or error parts into an htmx response.
    ///
    /// An initial-page error leaves the current list unchanged. A subsequent-page
    /// error replaces the loading sentinel with retry markup.
    pub fn respond(
        &self,
        search: &PAction,
        page: Result<Page<Item, PAction::Cursor>, impl Into<Parts>>,
    ) -> Response {
        match (page, search.cursor()) {
            (Ok(page), _) => self.render(search, page).into_response(),
            (Err(parts), Some(cursor)) => HxPartial::new()
                .main((self.retry)(retry(search.next(cursor), self.retry_trigger)))
                .parts(parts)
                .into_response(),
            // Keep htmx from replacing the current list with an empty response.
            (Err(parts), None) => {
                (HxReswap(SwapOption::None), HxPartial::new().parts(parts)).into_response()
            }
        }
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
