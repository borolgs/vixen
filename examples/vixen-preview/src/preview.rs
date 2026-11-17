use axum::extract::State;
use vixen::{
    maud::{Markup, html},
    route,
};

use super::{todos, ui};

fn without_imports(mut source: &str) -> &str {
    source = source.trim_start();
    while source.starts_with("use ") {
        let Some((_, rest)) = source.split_once(';') else {
            break;
        };
        source = rest.trim_start();
    }
    source
}

fn code_panel(title: &str, language: &str, source: &str) -> Markup {
    let body = without_imports(source);
    let (imports, _) = source.split_at(source.len() - body.len());

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
                button type="button" class="preview-code-toggle"
                    data-preview-full aria-pressed="false"
                    aria-label=(format!("Full {title} source"))
                { "Full" }
            }
            pre {
                code data-preview-code=(language) {
                    span data-preview-imports hidden { (imports) }
                    span data-preview-body { (body) }
                }
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
        div class="preview-tabs" role="tablist" aria-label="Examples" {
            button type="button" role="tab" id="preview-tab-hello"
                aria-controls="preview-hello" aria-selected="true"
                data-preview-tab="hello" { "hello" }
            button type="button" role="tab" id="preview-tab-todos"
                aria-controls="preview-todos" aria-selected="false"
                tabindex="-1"
                data-preview-tab="todos" { "todos" }
        }

        section id="preview-hello" role="tabpanel"
            aria-labelledby="preview-tab-hello"
            data-preview-example="hello"
        {
            (ui::form())

            div class="preview-snippets" {
                (code_panel("UI", "maud", include_str!("ui.rs")))
                (code_panel("App", "rust", include_str!("app.rs")))
            }

            p class="preview-error" data-preview-error role="alert" hidden {}

            section class="preview-result-panel" aria-label="Result" {
                div class="preview-result" role="status" aria-live="polite" {
                    span data-preview-status { "Ready" }
                    (ui::greeting::slot())
                }
            }
        }

        section id="preview-todos" role="tabpanel"
            aria-labelledby="preview-tab-todos"
            data-preview-example="todos" class="preview-layout" hidden
        {
            div class="preview-demo" {
                (todos::view(&state))
            }
            div class="preview-source" {
                (code_panel("todos.rs", "rust", include_str!("todos.rs")))
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
    use super::{code_panel, without_imports};

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
        let markup = code_panel("Example", "rust", source).into_string();
        assert!(markup.contains("--preview-code-columns: 23"));
    }

    #[test]
    fn strips_leading_single_and_multiline_imports() {
        let source = concat!(
            "\nuse vixen::{\n    fragment,\n    maud::Markup,\n};\n",
            "\nuse super::Greet;\n\n#[fragment]\nfn form() {}\n",
        );
        assert_eq!(without_imports(source), "#[fragment]\nfn form() {}\n");
    }

    #[test]
    fn preserves_body_imports_and_formatting() {
        let source = "fn example() {\n    use vixen::maud::html;\n}\n";
        assert_eq!(without_imports(source), source);
    }

    #[test]
    fn strips_import_only_source() {
        assert_eq!(without_imports("use vixen::maud::html;\n\n"), "");
        assert_eq!(without_imports(""), "");
    }

    #[test]
    fn keeps_incomplete_import() {
        let source = "use vixen::maud::html";
        assert_eq!(without_imports(source), source);
    }
}
