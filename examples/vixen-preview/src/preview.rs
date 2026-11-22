use axum::extract::State;
use vixen::{
    maud::{Markup, html},
    route,
};

use super::{hello, todos};

fn code_panel(title: &str, source: &str) -> Markup {
    let columns = source
        .lines()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(1);

    html! {
        section class="preview-code-panel" aria-label=(title)
            style=(format!("--preview-code-columns: {columns}"))
        {
            header class="preview-code-header" {
                span { (title) }
            }
            pre {
                code data-preview-code="rust" { (source) ("\n".repeat(20)) }
            }
        }
    }
}

#[route("/")]
pub struct PreviewPath;

pub async fn preview(
    _: PreviewPath,
    State(state): State<todos::Todos>,
) -> Markup {
    let state = state.lock().expect("todo state poisoned");

    html! {
        div class="preview-tabs" {
            div class="preview-tab-list" role="tablist" aria-label="Examples" {
                button type="button" role="tab" id="preview-tab-hello"
                    aria-controls="preview-hello" aria-selected="true"
                    data-preview-tab="hello" { "hello" }
                button type="button" role="tab" id="preview-tab-todos"
                    aria-controls="preview-todos" aria-selected="false"
                    tabindex="-1"
                    data-preview-tab="todos" { "todos" }
            }
        }

        section id="preview-hello" role="tabpanel"
            aria-labelledby="preview-tab-hello"
            data-preview-example="hello" class="preview-layout"
        {
            div class="preview-demo" {
                (hello::view())
            }
            div class="preview-source" {
                (code_panel("hello.rs", include_str!("hello.rs")))
            }
            footer class="preview-footer" {
                span data-preview-status role="status" { "Ready" }
                span { "Rust / WASM" }
            }
            p class="preview-error" data-preview-error role="alert" hidden {}
        }

        section id="preview-todos" role="tabpanel"
            aria-labelledby="preview-tab-todos"
            data-preview-example="todos" class="preview-layout" hidden
        {
            div class="preview-demo" {
                (todos::view(&state))
            }
            div class="preview-source" {
                (code_panel("todos.rs", include_str!("todos.rs")))
            }
            footer class="preview-footer" {
                span data-preview-status role="status" { "Ready" }
                span { "Rust / WASM · state resets on reload" }
            }
            p class="preview-error" data-preview-error role="alert" hidden {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::code_panel;

    #[test]
    fn todos_start_with_active_and_completed_samples() {
        let state = super::todos::TodoState::default();
        let markup = super::todos::view(&state).into_string();

        for (id, title) in [
            (0, "Buy milk"),
            (1, "Water the plants"),
            (2, "Read the htmx 4 docs"),
        ] {
            assert!(markup.contains(&format!("id=\"todo-item-{id}\"")));
            assert!(markup.contains(title));
        }
        assert_eq!(markup.matches("data-done=\"true\"").count(), 1);
        assert_eq!(markup.matches("data-done=\"false\"").count(), 2);
        assert!(markup.contains(">2 left</p>"));
    }

    #[test]
    fn code_width_accounts_for_the_longest_line_including_imports() {
        let source = "use some::long::import;\n\nfn main() {}\n";
        let markup = code_panel("Example", source).into_string();
        assert!(markup.contains("--preview-code-columns: 23"));
    }

    #[test]
    fn code_panel_shows_the_whole_file_without_legacy_controls() {
        let source = "use vixen::maud::html;\n\nfn example() {}\n";
        let markup = code_panel("hello.rs", source).into_string();
        assert!(markup.contains(source));
        assert!(markup.contains("data-preview-code=\"rust\""));
        assert!(!markup.contains("data-preview-imports"));
        assert!(!markup.contains("data-preview-full"));
    }

    #[test]
    fn hello_has_a_form_static_text_and_a_response_slot() {
        let markup = super::hello::view().into_string();
        let form = markup.find("id=\"form\"").expect("greeting form");
        let note = markup.find("Submit a name").expect("static explanation");
        let response = markup.find("id=\"greeting\"").expect("response slot");
        assert!(form < note && note < response);
        assert!(markup.contains("aria-label=\"Your name\""));
        assert!(markup.contains("aria-live=\"polite\""));
    }
}
