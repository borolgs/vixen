use vixen::{Fragment, HxAction, HxPartial, Id, Selector, id, maud::html, partial};

#[id]
struct TodoId(u64);

/// Already documented; the macro appends the id below.
#[id("user")]
struct UserSlug(String);

fn row(id: &UserSlug) -> Fragment<UserSlug> {
    Fragment::new(id, html! { li id=(id) {} })
}

fn main() {
    let todo = TodoId(7);
    assert_eq!(html! { (todo) }.into_string(), "todo-7");
    assert_eq!(todo.sel().0, "#todo-7");
    let _ = todo.slot();
    let _ = partial!(todo => html! { "milk" });

    // A `String` id isn't `Copy`: everything borrows it.
    let slug = UserSlug("ann".to_owned());
    let _: Selector = (&slug).into();
    let _ = HxAction::new("/save").target(&slug);
    let _ = HxPartial::new().target(&slug, html! {});
    let _ = partial!(&slug => html! {}, row(&slug));
    let _ = html! { (row(&slug)) (slug.slot()) };
}
