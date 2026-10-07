use axum::response::{IntoResponse, Response};
use maud::{Markup, html};
use serde_json::json;

use crate::{HxPartial, Part, Parts, Selector, hx::HxEvent, markers::Filled};

#[derive(Clone, Copy)]
pub(super) struct Modal {
    id: &'static str,
    content_class: Option<&'static str>,
}

/// Updates all slots in a [`Drawer`](super::Drawer) or [`Dialog`](super::Dialog).
///
/// Start with the widget's `header`, `content`, or `footer` method, then chain
/// setters for the other slots. `Slots` can be returned directly from a handler
/// or included in [`partial!`](crate::partial!). Omitted slots are emptied.
///
/// To update only part of an open modal, target a
/// [`#[fragment]`](macro@crate::fragment) inside a slot.
pub struct Slots {
    id: &'static str,
    header: Markup,
    content: Markup,
    footer: Markup,
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
            header: html! {},
            content: html! {},
            footer: html! {},
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
    /// Sets the header slot.
    pub fn header(mut self, header: impl Into<Markup>) -> Self {
        self.header = header.into();
        self
    }

    /// Sets the content slot.
    pub fn content(mut self, content: impl Into<Markup>) -> Self {
        self.content = content.into();
        self
    }

    /// Sets the footer slot.
    pub fn footer(mut self, footer: impl Into<Markup>) -> Self {
        self.footer = footer.into();
        self
    }
}

impl From<Slots> for Parts {
    fn from(slots: Slots) -> Self {
        let id = slots.id;
        let part = |slot, markup| Part::new(format!("#{}", slot_id(id, slot)), markup);
        vec![
            part(HEADER, slots.header),
            part(CONTENT, slots.content),
            part(FOOTER, slots.footer),
        ]
        .into()
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
    fn slots_replace_every_target_in_order() {
        let html = HxPartial::from(
            Modal::new("m")
                .slots()
                .footer(html! { "f" })
                .content(html! { "c" })
                .header(html! { "h" }),
        )
        .render()
        .into_string();
        assert_eq!(
            html,
            concat!(
                r##"<hx-partial hx-target="#m-header">h</hx-partial>"##,
                r##"<hx-partial hx-target="#m-content">c</hx-partial>"##,
                r##"<hx-partial hx-target="#m-footer">f</hx-partial>"##,
            )
        );
    }

    #[test]
    fn omitted_slots_are_empty() {
        let html = HxPartial::from(Modal::new("m").slots().footer(html! { "f" }))
            .render()
            .into_string();
        assert_eq!(
            html,
            concat!(
                r##"<hx-partial hx-target="#m-header"></hx-partial>"##,
                r##"<hx-partial hx-target="#m-content"></hx-partial>"##,
                r##"<hx-partial hx-target="#m-footer">f</hx-partial>"##,
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
