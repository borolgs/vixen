/// A typed element id: what [`#[id]`](macro@crate::id) and
/// [`#[fragment]`](macro@crate::fragment) give a type.
///
/// The trait is for generic code that takes either kind of type.
pub trait Id {
    /// The bare id, `"todo-list"`.
    const ID: &'static str;
    /// The id as a CSS selector, `"#todo-list"`.
    const SEL: &'static str;
}
