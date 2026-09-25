use maud::{Markup, html};

use super::{Slots, modal::Modal};
use crate::{Selector, hx::HxEvent};

/// A server-driven [Basecoat drawer](https://basecoatui.com/components/drawer/).
///
/// Render its [`shell`](Self::shell) once on the page. Returning updates for
/// any of its slots opens the drawer.
#[derive(Clone, Copy)]
pub struct Drawer {
    modal: Modal,
    side: Side,
}

/// The edge from which a [`Drawer`] opens.
#[derive(Clone, Copy)]
pub enum Side {
    /// The top edge.
    Top,
    /// The right edge.
    Right,
    /// The bottom edge.
    Bottom,
    /// The left edge.
    Left,
}

impl Drawer {
    /// Creates a drawer on the right whose element uses `id`.
    ///
    /// # Panics
    ///
    /// Panics if `id` is empty or contains ASCII whitespace or `#`.
    pub const fn new(id: &'static str) -> Self {
        Self {
            modal: Modal::new(id),
            side: Side::Right,
        }
    }

    /// Sets the edge from which the drawer opens.
    pub const fn side(mut self, side: Side) -> Self {
        self.side = side;
        self
    }

    /// Sets classes on the content slot.
    pub const fn content_class(mut self, class: &'static str) -> Self {
        self.modal = self.modal.content_class(class);
        self
    }

    /// Renders the empty drawer shell.
    pub fn shell(&self) -> Markup {
        html! {
            dialog id=(self.modal.id()) class="drawer" data-side=(self.side.as_str())
                aria-labelledby=(self.modal.header_id()) {
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

    /// Returns an htmx event that closes the drawer.
    pub fn close(&self) -> HxEvent {
        self.modal.close()
    }
}

impl From<Drawer> for Selector {
    fn from(drawer: Drawer) -> Self {
        drawer.modal.into()
    }
}

impl Side {
    const fn as_str(self) -> &'static str {
        match self {
            Side::Top => "top",
            Side::Right => "right",
            Side::Bottom => "bottom",
            Side::Left => "left",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_renders_empty_slots_labelled_by_the_header() {
        let html = Drawer::new("p")
            .side(Side::Left)
            .content_class("px-4")
            .shell()
            .into_string();
        assert_eq!(
            html,
            concat!(
                r#"<dialog id="p" class="drawer" data-side="left" aria-labelledby="p-header"><div>"#,
                r#"<header id="p-header"></header>"#,
                r#"<section id="p-content" class="px-4"></section>"#,
                r#"<footer id="p-footer"></footer>"#,
                "</div></dialog>",
            )
        );
    }
}
