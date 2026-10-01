use crate::Selector;

/// A typed element id: what [`#[id]`](macro@crate::id) and
/// [`#[fragment]`](macro@crate::fragment) give a type.
///
/// The trait is for generic code that takes either kind of type.
pub trait Id {
    /// The id as a CSS selector: `#todo-list`, `#todo-7`.
    fn sel(&self) -> Selector;
}

impl<T: Id + ?Sized> Id for &T {
    fn sel(&self) -> Selector {
        (**self).sel()
    }
}

#[cfg(test)]
mod tests {
    use maud::html;

    use crate::{HxAction, id, partial};

    #[id]
    struct TodoId(String);

    #[test]
    fn a_dynamic_id_renders_escaped() {
        let id = TodoId(r#"a"<b"#.to_owned());
        assert_eq!(
            html! { li id=(id) hx-action=(HxAction::new("/x").target(&id)) {} }.into_string(),
            concat!(
                r#"<li id="todo-a&quot;&lt;b" "#,
                r##"hx-action="/x" hx-target="#todo-a&quot;&lt;b" hx-method="post"></li>"##,
            )
        );
        assert_eq!(
            partial!(id => html! {}).render().into_string(),
            r##"<hx-partial hx-target="#todo-a&quot;&lt;b"></hx-partial>"##
        );
    }
}
