use vixen::{
    HxPartialResponse, Part, SyncStrategy, fragment,
    hx::HxResponseTrigger,
    maud::{Markup, html},
    partial,
    ui::basecoatui::{Dialog, Drawer, Side, Toaster},
};

const DRAWER: Drawer = Drawer::new("profile").side(Side::Left).content_class("px-4");
const CONFIRM: Dialog = Dialog::new("confirm");
const TOASTER: Toaster = Toaster::new();

#[fragment]
fn name_field() -> Markup {
    html! { input id=(Self); }
}

// A chain is a concrete handler return type; a fragment fills a slot as is.
fn open() -> HxPartialResponse {
    DRAWER.header(html! { "Profile" }).content(name_field()).into()
}

fn main() {
    let _ = open();
    let _ = partial!(CONFIRM.footer(html! { "Delete" }), TOASTER.success("Saved", "ok"));
    let _ = (
        HxResponseTrigger::normal([DRAWER.close(), CONFIRM.close()]),
        TOASTER.success("Saved", "ok"),
    );
    let _ = Part::new(DRAWER, html! {});
    let _ = SyncStrategy::QueueAll.on(CONFIRM);
}
