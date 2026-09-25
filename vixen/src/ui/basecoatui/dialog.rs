use maud::{Markup, html};

use super::{Slots, modal::Modal};
use crate::{Selector, hx::HxEvent};

/// A server-driven [Basecoat dialog](https://basecoatui.com/components/dialog/).
///
/// Render its [`shell`](Self::shell) once on the page. Returning updates for
/// any of its slots opens the dialog.
#[derive(Clone, Copy)]
pub struct Dialog {
    modal: Modal,
}

impl Dialog {
    /// Creates a dialog whose element uses `id`.
    ///
    /// # Panics
    ///
    /// Panics if `id` is empty or contains ASCII whitespace or `#`.
    pub const fn new(id: &'static str) -> Self {
        Self {
            modal: Modal::new(id),
        }
    }

    /// Sets classes on the content slot.
    pub const fn content_class(mut self, class: &'static str) -> Self {
        self.modal = self.modal.content_class(class);
        self
    }

    /// Renders the empty dialog shell.
    pub fn shell(&self) -> Markup {
        html! {
            dialog id=(self.modal.id()) class="dialog"
                aria-labelledby=(self.modal.header_id())
                onclick="if (event.target === this) this.close()" {
                (self.modal.panel())
            }
        }
    }

    /// Starts a response with an update for the header slot.
    pub fn header(&self, header: impl Into<Markup>) -> Slots {
        self.modal.slots().header(header)
    }

    /// Starts a response with an update for the content slot.
    pub fn content(&self, content: impl Into<Markup>) -> Slots {
        self.modal.slots().content(content)
    }

    /// Starts a response with an update for the footer slot.
    pub fn footer(&self, footer: impl Into<Markup>) -> Slots {
        self.modal.slots().footer(footer)
    }

    /// Returns an htmx event that closes the dialog.
    pub fn close(&self) -> HxEvent {
        self.modal.close()
    }
}

impl From<Dialog> for Selector {
    fn from(dialog: Dialog) -> Self {
        dialog.modal.into()
    }
}
