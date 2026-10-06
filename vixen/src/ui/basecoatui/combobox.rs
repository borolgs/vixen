use maud::{Markup, Render, html};

use crate::HxAction;

const SEARCH_TRIGGER: &str = "input changed delay:300ms";

/// A [Basecoat combobox](https://basecoatui.com/components/combobox/).
///
/// The hidden input named `field` contains the selected option's `data-value`,
/// or a JSON array in [`multiple`](Self::multiple) mode. With
/// [`search`](Self::search), selections are `{value, label}` objects instead.
///
/// ```
/// use vixen::{maud::html, ui::basecoatui::Combobox};
///
/// let category = Combobox::new("category", "category")
///     .value("kitchen")
///     .options(html! {
///         div role="option" data-value="kitchen" { "Kitchen" }
///         div role="option" data-value="textiles" { "Textiles" }
///     });
///
/// let form = html! { form { (category) } };
/// # assert!(form.into_string().contains(r#"name="category" value="kitchen""#));
/// ```
///
/// For paginated options, pass [`Paged::search`](crate::Paged::search) to
/// [`search`](Self::search) and [`Paged::render`](crate::Paged::render) to
/// [`options`](Self::options).
pub struct Combobox {
    id: Markup,
    field: &'static str,
    value: Option<String>,
    multiple: bool,
    search_field: Option<&'static str>,
    search: Option<HxAction>,
    placeholder: Option<&'static str>,
    empty: Option<&'static str>,
    class: Option<&'static str>,
    options: Markup,
}

impl Combobox {
    /// Creates a combobox with text input `id` and hidden input `field`.
    pub fn new(id: impl Render, field: &'static str) -> Self {
        Self {
            id: id.render(),
            field,
            value: None,
            multiple: false,
            search_field: None,
            search: None,
            placeholder: None,
            empty: None,
            class: None,
            options: html! {},
        }
    }

    /// Enables multiple selection.
    pub fn multiple(mut self) -> Self {
        self.multiple = true;
        self
    }

    /// Sets the initial submitted value.
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// Enables server-side search. Typing posts the search text to `action` as
    /// `field` after 300 ms.
    ///
    /// Without this, Basecoat filters the existing options.
    pub fn search(mut self, field: &'static str, action: impl Into<HxAction>) -> Self {
        self.search_field = Some(field);
        self.search = Some(action.into().trigger(SEARCH_TRIGGER));
        self
    }

    /// Sets the text input's placeholder.
    pub fn placeholder(mut self, text: &'static str) -> Self {
        self.placeholder = Some(text);
        self
    }

    /// Sets the empty-state text.
    pub fn empty(mut self, text: &'static str) -> Self {
        self.empty = Some(text);
        self
    }

    /// Adds CSS classes to the root element.
    pub fn class(mut self, class: &'static str) -> Self {
        self.class = Some(class);
        self
    }

    /// Sets the option markup. Selectable options need `role="option"` and a
    /// `data-value`.
    pub fn options(mut self, options: impl Into<Markup>) -> Self {
        self.options = options.into();
        self
    }
}

impl Render for Combobox {
    fn render(&self) -> Markup {
        let value = match &self.value {
            Some(value) => value,
            None if self.multiple => "[]",
            None => "",
        };
        let searched = self.search.is_some();

        html! {
            div class={ "combobox" @if let Some(class) = self.class { " " (class) } }
                data-filter=[searched.then_some("manual")]
                data-format=[searched.then_some("object")]
                data-auto-highlight="true"
            {
                input id=(self.id) type="text" role="combobox" placeholder=[self.placeholder]
                    autocomplete="off" autocorrect="off" spellcheck="false"
                    aria-autocomplete="list" aria-expanded="false"
                    aria-controls={ (self.id) "-listbox" }
                    name=[self.search_field] hx-action=[&self.search];
                div data-popover aria-hidden="true" {
                    div id={ (self.id) "-listbox" } role="listbox" aria-orientation="vertical"
                        aria-multiselectable=[self.multiple.then_some("true")]
                        data-empty=[self.empty]
                    {
                        (self.options)
                    }
                }
                input type="hidden" name=(self.field) value=(value);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_a_text_input_a_listbox_and_the_hidden_value() {
        let html = Combobox::new("category", "category")
            .value("kitchen")
            .placeholder("Pick one")
            .empty("No match.")
            .class("w-full")
            .options(html! { div role="option" data-value="kitchen" { "Kitchen" } })
            .render()
            .into_string();
        assert_eq!(
            html,
            concat!(
                r#"<div class="combobox w-full" data-auto-highlight="true">"#,
                r#"<input id="category" type="text" role="combobox" placeholder="Pick one" "#,
                r#"autocomplete="off" autocorrect="off" spellcheck="false" "#,
                r#"aria-autocomplete="list" aria-expanded="false" aria-controls="category-listbox">"#,
                r#"<div data-popover aria-hidden="true">"#,
                r#"<div id="category-listbox" role="listbox" aria-orientation="vertical" "#,
                r#"data-empty="No match.">"#,
                r#"<div role="option" data-value="kitchen">Kitchen</div>"#,
                "</div></div>",
                r#"<input type="hidden" name="category" value="kitchen">"#,
                "</div>",
            )
        );
    }

    #[test]
    fn multiple_starts_from_an_empty_array() {
        let html = Combobox::new("tags", "tags")
            .multiple()
            .render()
            .into_string();
        assert!(html.contains(r#"aria-multiselectable="true""#));
        assert!(html.contains(r#"<input type="hidden" name="tags" value="[]">"#));
    }

    #[test]
    fn search_posts_the_typed_text_and_turns_filtering_off() {
        let html = Combobox::new("tags", "tags")
            .search("q", "/tags/options")
            .render()
            .into_string();
        assert!(html.contains(r#"data-filter="manual" data-format="object""#));
        assert!(html.contains(concat!(
            r#"name="q" hx-action="/tags/options" "#,
            r#"hx-trigger="input changed delay:300ms" hx-method="post">"#,
        )));
    }
}
