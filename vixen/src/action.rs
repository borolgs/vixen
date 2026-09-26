use std::fmt::Write;

use axum_htmx::SwapOption;
use maud::{Escaper, Render};
use serde_json::{Map, Value};

use crate::Selector;
use crate::partial::swap_attr;

/// Attributes for an htmx action request.
///
/// Use it as `hx-action=(...)`. Field values become `hx-vals`, chained options
/// add other htmx attributes, and the request method is always POST.
///
/// When rendered as `hx-action`, it writes that attribute plus any configured
/// `hx-*` attributes. Using it as another attribute's value produces invalid
/// markup.
///
/// [`#[action]`](macro@crate::action) builds one for each endpoint, prefixed
/// with [`base_path!`](crate::base_path). To build one directly:
///
/// ```
/// use vixen::{HxAction, SyncStrategy, maud::html};
///
/// let search = HxAction::new("/search")
///     .base(vixen::base_path!())
///     .trigger("input changed delay:300ms")
///     .target("#results")
///     .sync(SyncStrategy::Replace);
///
/// assert_eq!(
///     html! { input name="q" hx-action=(search); }.into_string(),
///     concat!(
///         r#"<input name="q" hx-action="/search" "#,
///         r#"hx-trigger="input changed delay:300ms" "#,
///         r##"hx-target="#results" "##,
///         r#"hx-sync="replace" "#,
///         r#"hx-method="post">"#,
///     )
/// );
/// ```
#[derive(Clone)]
pub struct HxAction {
    base: &'static str,
    path: &'static str,
    vals: Map<String, Value>,
    trigger: Option<String>,
    target: Option<Selector>,
    swap: Option<SwapOption>,
    sync: Option<HxSync>,
}

impl From<&'static str> for HxAction {
    fn from(path: &'static str) -> Self {
        Self::new(path)
    }
}

impl HxAction {
    /// Creates a POST action for `path` with no values or options.
    pub fn new(path: &'static str) -> Self {
        HxAction {
            base: "",
            path,
            vals: Map::new(),
            trigger: None,
            target: None,
            swap: None,
            sync: None,
        }
    }

    /// Prefixes the path, usually with [`base_path!`](crate::base_path).
    pub fn base(mut self, base: &'static str) -> Self {
        self.base = base;
        self
    }

    /// Adds an `hx-vals` entry, omitting null values.
    pub fn val(mut self, key: &'static str, value: Value) -> Self {
        if !value.is_null() {
            self.vals.insert(key.into(), value);
        }
        self
    }

    /// Sets `hx-trigger`.
    pub fn trigger(mut self, spec: impl Into<String>) -> Self {
        self.trigger = Some(spec.into());
        self
    }

    /// Sets `hx-target` from a selector or an `#[id]` type.
    pub fn target(mut self, target: impl Into<Selector>) -> Self {
        self.target = Some(target.into());
        self
    }

    /// Sets `hx-swap`.
    pub fn swap(mut self, swap: SwapOption) -> Self {
        self.swap = Some(swap);
        self
    }

    /// Sets `hx-sync` from a strategy, or one scoped with [`SyncStrategy::on`].
    pub fn sync(mut self, sync: impl Into<HxSync>) -> Self {
        self.sync = Some(sync.into());
        self
    }
}

fn escape(buffer: &mut String, s: &str) {
    let _ = Escaper::new(buffer).write_str(s);
}

impl Render for HxAction {
    fn render_to(&self, buffer: &mut String) {
        // Rendering starts inside maud's quoted `hx-action` value. Each
        // separator starts another attribute; maud supplies the final quote.
        escape(buffer, self.base);
        escape(buffer, self.path);
        if !self.vals.is_empty() {
            buffer.push_str("\" hx-vals=\"");
            let vals = serde_json::to_string(&self.vals).expect("hx-vals map serializes");
            escape(buffer, &vals);
        }
        if let Some(trigger) = &self.trigger {
            buffer.push_str("\" hx-trigger=\"");
            escape(buffer, trigger);
        }
        if let Some(target) = &self.target {
            buffer.push_str("\" hx-target=\"");
            escape(buffer, &target.0);
        }
        if let Some(swap) = self.swap {
            buffer.push_str("\" hx-swap=\"");
            escape(buffer, &swap_attr(swap));
        }
        if let Some(sync) = &self.sync {
            buffer.push_str("\" hx-sync=\"");
            escape(buffer, &sync.attr());
        }
        buffer.push_str("\" hx-method=\"post");
    }
}

/// What `hx-sync` does with a request while another one is in flight.
///
/// htmx 4 defaults to [`SyncStrategy::QueueFirst`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SyncStrategy {
    /// Drops this request.
    Drop,
    /// Drops this request, and aborts it if another one arrives while it runs.
    Abort,
    /// Aborts the in-flight request and runs this one.
    Replace,
    /// Queues this request only if the queue is empty.
    QueueFirst,
    /// Queues this request in place of whatever was queued.
    QueueLast,
    /// Queues every request.
    QueueAll,
}

impl SyncStrategy {
    /// Synchronizes on `scope` instead of the element itself, e.g.
    /// `"closest form"` or an `#[id]` type.
    pub fn on(self, scope: impl Into<Selector>) -> HxSync {
        HxSync {
            scope: Some(scope.into()),
            strategy: self,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Drop => "drop",
            Self::Abort => "abort",
            Self::Replace => "replace",
            Self::QueueFirst => "queue first",
            Self::QueueLast => "queue last",
            Self::QueueAll => "queue all",
        }
    }
}

impl Render for SyncStrategy {
    fn render_to(&self, buffer: &mut String) {
        buffer.push_str(self.as_str());
    }
}

impl Render for HxSync {
    fn render_to(&self, buffer: &mut String) {
        self.attr().render_to(buffer);
    }
}

/// An `hx-sync` value: a strategy, optionally scoped to another element.
#[derive(Clone)]
pub struct HxSync {
    scope: Option<Selector>,
    strategy: SyncStrategy,
}

impl From<SyncStrategy> for HxSync {
    fn from(strategy: SyncStrategy) -> Self {
        HxSync {
            scope: None,
            strategy,
        }
    }
}

impl HxSync {
    fn attr(&self) -> String {
        match &self.scope {
            Some(scope) => format!("{}:{}", scope.0, self.strategy.as_str()),
            None => self.strategy.as_str().to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use axum_extra::routing::TypedPath;
    use maud::html;

    use super::*;
    use crate::{action, id};

    #[action("/save")]
    struct Save {
        id: Option<i64>,
        name: String,
    }

    #[action("/search")]
    struct Search {
        q: String,
        after: Option<i64>,
    }

    #[id]
    struct ResultsId;

    #[test]
    fn the_path_is_the_route() {
        assert_eq!(Save::PATH, "/save");
        assert_eq!(Search::PATH, "/search");
    }

    #[test]
    fn setters_fill_vals_escaped_inside_the_attribute() {
        let html = html! { form hx-action=(Save::action().name("a \"quoted\" name").id(7)) {} }
            .into_string();
        assert_eq!(
            html,
            concat!(
                r#"<form hx-action="/save" "#,
                r#"hx-vals="{&quot;id&quot;:7,&quot;name&quot;:&quot;a \&quot;quoted\&quot; name&quot;}" "#,
                r#"hx-method="post"></form>"#,
            )
        );
    }

    #[test]
    fn options_render_from_the_same_value() {
        let html = html! {
            input hx-action=(Search::action().q("steel").after(40).hx()
                .trigger("input changed delay:300ms")
                .target(ResultsId)
                .swap(SwapOption::OuterHtml)
                .sync(SyncStrategy::Replace));
        }
        .into_string();
        assert_eq!(
            html,
            concat!(
                r#"<input hx-action="/search" "#,
                r#"hx-vals="{&quot;after&quot;:40,&quot;q&quot;:&quot;steel&quot;}" "#,
                r#"hx-trigger="input changed delay:300ms" "#,
                r##"hx-target="#results" "##,
                r#"hx-swap="outerHTML" "#,
                r#"hx-sync="replace" "#,
                r#"hx-method="post">"#,
            )
        );
    }

    #[test]
    fn base_prefixes_the_path() {
        let html = html! { form hx-action=(HxAction::new("/save").base("/app")) {} }.into_string();
        assert_eq!(
            html,
            r#"<form hx-action="/app/save" hx-method="post"></form>"#
        );
    }

    #[test]
    fn a_target_is_also_a_raw_selector() {
        let html = html! { div hx-action=(Search::action().hx().target("#list")) {} }.into_string();
        assert_eq!(
            html,
            r##"<div hx-action="/search" hx-target="#list" hx-method="post"></div>"##
        );
    }

    #[test]
    fn none_renders_no_attribute() {
        let html = html! { div hx-action=[None::<HxAction>] {} }.into_string();
        assert_eq!(html, "<div></div>");
    }

    #[test]
    fn renders_strategy_alone() {
        assert_eq!(HxSync::from(SyncStrategy::Replace).attr(), "replace");
        assert_eq!(HxSync::from(SyncStrategy::QueueLast).attr(), "queue last");
    }

    #[test]
    fn renders_scope_before_strategy() {
        assert_eq!(
            SyncStrategy::Abort.on("closest form").attr(),
            "closest form:abort"
        );
    }
}
