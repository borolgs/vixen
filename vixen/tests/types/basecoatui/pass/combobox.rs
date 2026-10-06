use vixen::{HxAction, Page, Paged, PagedAction, action, id, maud::html, ui::basecoatui::Combobox};

#[action("/tags/options")]
struct TagOptions {
    q: String,
    after: Option<u32>,
}

impl PagedAction for TagOptions {
    type Cursor = u32;

    fn cursor(&self) -> Option<u32> {
        self.after
    }

    fn next(&self, after: u32) -> HxAction {
        TagOptions::action().q(&self.q).after(after).hx()
    }
}

const OPTIONS: Paged<TagOptions, String> = Paged::new(
    "tag-options",
    |tag| html! { div role="option" data-value=(tag) { (tag) } },
);

#[id]
struct TagPickerId;

fn main() {
    let _ = Combobox::new("category", "category")
        .value("kitchen")
        .placeholder("Pick one")
        .empty("No match.")
        .class("w-full")
        .options(html! { div role="option" data-value="kitchen" { "Kitchen" } });

    // Server-searched and paged: `Paged` supplies both the action and the options.
    let search = TagOptions {
        q: String::new(),
        after: None,
    };
    let first = Page {
        items: vec![String::from("steel")],
        next: Some(20),
    };
    let picker = Combobox::new(TagPickerId, "tags")
        .multiple()
        .search(TagOptions::FIELD.q, OPTIONS.search(TagOptions::action()))
        .options(OPTIONS.render(&search, first));

    // The id is an `#[id]` type, shared with the label.
    let _ = html! {
        label for=(TagPickerId) { "Tags" }
        (picker)
    };
}
