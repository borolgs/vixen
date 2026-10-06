use vixen::{HxAction, SyncStrategy, action, hx::SwapOption, id, maud::html, routing::TypedPath};

#[action("/save")]
struct Save {
    id: Option<i64>,
    name: String,
    note: Option<String>,
}

#[action("/search")]
struct Search {
    q: String,
    after: Option<i64>,
}

#[id]
struct ResultsId;

fn main() {
    let _: &str = Save::PATH;

    // A `String` field takes anything `Into<String>`, an `Option<T>` field takes `T`.
    let _ = Save::action()
        .name("kettle")
        .name(String::from("mug"))
        .id(7);

    // `maybe_<field>` takes the `Option` itself.
    let _ = Save::action()
        .maybe_id(Some(7))
        .maybe_note(Some("fragile"))
        .maybe_note(None::<String>);

    // `.hx()` hands over an `HxAction`; a target is an `#[id]` type or a raw selector.
    let _: HxAction = Search::action()
        .q("steel")
        .after(40)
        .hx()
        .trigger("input changed delay:300ms")
        .target(ResultsId)
        .swap(SwapOption::OuterHtml)
        .sync(SyncStrategy::Replace);
    let _ = Search::action().hx().target("#list");

    // The builder and `HxAction` are both attribute values, optional ones too.
    let _ = html! {
        form hx-action=(Save::action()) {}
        div hx-action=(Search::action().hx()) {}
        div hx-action=[None::<HxAction>] {}
    };
}
