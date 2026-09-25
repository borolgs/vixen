use axum::response::{IntoResponse, Response};
use maud::{Markup, html};
use serde_json::json;

use crate::{HxPartial, Part, Parts, Selector, hx::HxEvent, markers::Filled};

#[derive(Clone, Copy)]
pub(super) struct Modal {
    id: &'static str,
    content_class: Option<&'static str>,
}

/// Updates for a [`Drawer`](super::Drawer) or [`Dialog`](super::Dialog).
///
/// Build them from a widget's `header`, `content`, or `footer` method, then
/// chain updates for the other slots. `Slots` can be returned directly from a
/// handler or included in [`partial!`](crate::partial!).
pub struct Slots {
    id: &'static str,
    parts: Vec<Part>,
}

const HEADER: &str = "header";
const CONTENT: &str = "content";
const FOOTER: &str = "footer";

impl Modal {
    pub(super) const fn new(id: &'static str) -> Self {
        let bytes = id.as_bytes();

        assert!(!bytes.is_empty(), "a modal id must not be empty");

        let mut index = 0;
        while index < bytes.len() {
            assert!(
                !bytes[index].is_ascii_whitespace() && bytes[index] != b'#',
                "a modal id must not contain ASCII whitespace or `#`"
            );
            index += 1;
        }

        Self {
            id,
            content_class: None,
        }
    }

    pub(super) const fn content_class(mut self, class: &'static str) -> Self {
        self.content_class = Some(class);
        self
    }

    pub(super) const fn id(&self) -> &'static str {
        self.id
    }

    pub(super) fn header_id(&self) -> String {
        slot_id(self.id, HEADER)
    }

    pub(super) fn panel(&self) -> Markup {
        html! {
            div {
                header id=(slot_id(self.id, HEADER)) {}
                section id=(slot_id(self.id, CONTENT)) class=[self.content_class] {}
                footer id=(slot_id(self.id, FOOTER)) {}
            }
        }
    }

    pub(super) fn slots(&self) -> Slots {
        Slots {
            id: self.id,
            parts: Vec::new(),
        }
    }

    pub(super) fn close(&self) -> HxEvent {
        HxEvent {
            name: "dialog:close".to_owned(),
            data: Some(json!({ "id": self.id })),
        }
    }
}

impl From<Modal> for Selector {
    fn from(modal: Modal) -> Self {
        Self(format!("#{}", modal.id))
    }
}

impl Slots {
    /// Adds an update for the header slot.
    pub fn header(self, header: impl Into<Markup>) -> Self {
        self.fill(HEADER, header.into())
    }

    /// Adds an update for the content slot.
    pub fn content(self, content: impl Into<Markup>) -> Self {
        self.fill(CONTENT, content.into())
    }

    /// Adds an update for the footer slot.
    pub fn footer(self, footer: impl Into<Markup>) -> Self {
        self.fill(FOOTER, footer.into())
    }

    fn fill(mut self, slot: &str, content: Markup) -> Self {
        self.parts
            .push(Part::new(format!("#{}", slot_id(self.id, slot)), content));
        self
    }
}

impl From<Slots> for Parts {
    fn from(slots: Slots) -> Self {
        slots.parts.into()
    }
}

impl From<Slots> for HxPartial<Filled> {
    fn from(slots: Slots) -> Self {
        HxPartial::new().parts(slots)
    }
}

impl IntoResponse for Slots {
    fn into_response(self) -> Response {
        HxPartial::from(self).into_response()
    }
}

fn slot_id(id: &str, slot: &str) -> String {
    format!("{id}-{slot}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slots_target_the_ids_derived_from_the_name_in_order() {
        let html = HxPartial::from(
            Modal::new("m")
                .slots()
                .footer(html! { "f" })
                .header(html! { "h" }),
        )
        .render()
        .into_string();
        assert_eq!(
            html,
            concat!(
                r##"<hx-partial hx-target="#m-footer">f</hx-partial>"##,
                r##"<hx-partial hx-target="#m-header">h</hx-partial>"##,
            )
        );
    }

    #[test]
    fn close_names_the_modal() {
        let event = Modal::new("m").close();
        assert_eq!(event.name, "dialog:close");
        assert_eq!(event.data, Some(json!({ "id": "m" })));
    }
}
