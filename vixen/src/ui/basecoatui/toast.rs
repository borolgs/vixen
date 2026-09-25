use axum_htmx::SwapOption;
use maud::{Markup, PreEscaped, html};
use serde::{Deserialize, Serialize};

use crate::{Part, Selector};

/// A Basecoat toast container.
///
/// Basecoat requires the fixed id `toaster`, so render one [`shell`](Self::shell)
/// per page. The struct is `const`-constructible: keep one in a `const` and
/// call it from handlers.
///
/// <https://basecoatui.com/components/toast/>
pub struct Toaster {
    align: Align,
}

/// A toast notification. [`Toast::new`] fills in the defaults.
pub struct Toast {
    /// Icon, color and ARIA role.
    pub category: Category,
    /// The heading.
    pub title: String,
    /// Text under the heading.
    pub description: Option<String>,
    /// How long the toast stays visible.
    pub duration: Duration,
    /// A footer button or link; clicking it dismisses the toast.
    pub action: Option<Action>,
}

/// The toast's icon, color and ARIA role: `alert` for [`Error`](Self::Error),
/// `status` otherwise.
///
/// Serializes as its lowercase name, so it can be an `#[action]` field.
#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    /// An operation completed.
    Success,
    /// A neutral note.
    Info,
    /// Something to check.
    Warning,
    /// An operation failed. Announced as an alert and shown longer by default.
    Error,
}

/// How long the toast remains visible.
#[derive(Clone, Copy, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Duration {
    /// Uses Basecoat's default: 3 seconds, or 5 seconds for errors.
    #[default]
    Default,
    /// Displays for the given number of milliseconds.
    Millis(u32),
    /// Remains until dismissed.
    Sticky,
}

/// A button or link in the toast footer. Clicking any of them dismisses the
/// toast.
pub enum Action {
    /// A button with this label that only dismisses.
    Dismiss(String),
    /// A link: dismisses, then follows `href`.
    Link {
        /// The link text.
        label: String,
        /// The `href`.
        href: String,
    },
    /// Your own footer markup: one button or several. Start from the template
    /// and add what you need. Keep `type`, `class` and
    /// `data-toast-action`; the last is what dismisses the toast on click.
    ///
    /// ```ignore
    /// button type="button" class="btn" data-toast-action { (label) }
    /// ```
    Custom(Markup),
}

/// The toaster's horizontal position. [`End`](Self::End) is Basecoat's default.
#[derive(Clone, Copy)]
pub enum Align {
    /// The inline start edge.
    Start,
    /// Centered.
    Center,
    /// The inline end edge.
    End,
}

const ID: &str = "toaster";
const SEL: &str = "#toaster";

impl Toaster {
    /// A toaster at the inline end.
    pub const fn new() -> Self {
        Self { align: Align::End }
    }

    /// Sets the horizontal position.
    pub const fn align(mut self, align: Align) -> Self {
        self.align = align;
        self
    }

    /// Renders the toast container. Place it once, anywhere in `<body>`.
    ///
    /// It is a manual popover so it stays above open dialogs; [`HEAD`](super::HEAD)
    /// carries the styles and scripts that make that work.
    pub fn shell(&self) -> Markup {
        html! {
            // Fix toaster (popover="manual")
            // https://github.com/hunvreus/basecoat/issues/133 (3/3)
            div popover="manual" id=(ID) class="toaster" data-align=[self.align.attr()] {}
        }
    }

    /// A [`Category::Success`] toast with the defaults of [`Toast::new`].
    pub fn success(&self, title: impl Into<String>, description: impl Into<String>) -> Part {
        self.toast(Toast::new(Category::Success, title, description))
    }

    /// A [`Category::Info`] toast with the defaults of [`Toast::new`].
    pub fn info(&self, title: impl Into<String>, description: impl Into<String>) -> Part {
        self.toast(Toast::new(Category::Info, title, description))
    }

    /// A [`Category::Warning`] toast with the defaults of [`Toast::new`].
    pub fn warning(&self, title: impl Into<String>, description: impl Into<String>) -> Part {
        self.toast(Toast::new(Category::Warning, title, description))
    }

    /// A [`Category::Error`] toast with the defaults of [`Toast::new`].
    pub fn error(&self, title: impl Into<String>, description: impl Into<String>) -> Part {
        self.toast(Toast::new(Category::Error, title, description))
    }

    /// Renders `toast` as a [`Part`] appended to the toaster:
    /// `hx-target="#toaster"`, `hx-swap="beforeend"`.
    pub fn toast(&self, toast: Toast) -> Part {
        let markup = html! {
            div class="toast" role=(toast.category.role()) aria-atomic="true" aria-hidden="false"
                data-category=(toast.category.as_str())
                data-duration=[toast.duration.attr()] {
                div class="toast-content" {
                    (PreEscaped(toast.category.icon()))
                    section {
                        h2 { (toast.title) }
                        @if let Some(description) = toast.description {
                            p { (description) }
                        }
                    }
                    @if let Some(action) = toast.action {
                        footer {
                            @match action {
                                Action::Dismiss(label) => {
                                    button type="button" class="btn" data-toast-action { (label) }
                                }
                                Action::Link { label, href } => {
                                    a href=(href) class="btn" data-toast-action { (label) }
                                }
                                Action::Custom(markup) => {
                                    (markup)
                                }
                            }
                        }
                    }
                }
            }
        };

        Part::new(Selector(SEL.to_owned()), markup).swap(SwapOption::BeforeEnd)
    }
}

impl Default for Toaster {
    fn default() -> Self {
        Self::new()
    }
}

impl Toast {
    /// A toast with Basecoat's default duration and a "Dismiss" button.
    pub fn new(
        category: Category,
        title: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            category,
            title: title.into(),
            description: Some(description.into()),
            duration: Duration::Default,
            action: Some(Action::Dismiss("Dismiss".to_owned())),
        }
    }
}

impl Category {
    const fn as_str(self) -> &'static str {
        match self {
            Category::Success => "success",
            Category::Info => "info",
            Category::Warning => "warning",
            Category::Error => "error",
        }
    }

    const fn role(self) -> &'static str {
        match self {
            Category::Error => "alert",
            _ => "status",
        }
    }

    const fn icon(self) -> &'static str {
        match self {
            Category::Success => {
                r#"<svg aria-hidden="true" xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><path d="m9 12 2 2 4-4"/></svg>"#
            }
            Category::Info => {
                r#"<svg aria-hidden="true" xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><path d="M12 16v-4"/><path d="M12 8h.01"/></svg>"#
            }
            Category::Warning => {
                r#"<svg aria-hidden="true" xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m21.73 18-8-14a2 2 0 0 0-3.48 0l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.73-3"/><path d="M12 9v4"/><path d="M12 17h.01"/></svg>"#
            }
            Category::Error => {
                r#"<svg aria-hidden="true" xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><path d="m15 9-6 6"/><path d="m9 9 6 6"/></svg>"#
            }
        }
    }
}

impl Duration {
    fn attr(self) -> Option<String> {
        match self {
            Duration::Default => None,
            Duration::Millis(ms) => Some(ms.to_string()),
            Duration::Sticky => Some("-1".to_owned()),
        }
    }
}

impl Align {
    const fn attr(self) -> Option<&'static str> {
        match self {
            Align::Start => Some("start"),
            Align::Center => Some("center"),
            Align::End => None,
        }
    }
}
