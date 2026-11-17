use vixen::{
    maud::{Markup, html},
    route,
};

use super::ui;

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

    html! {
        section class="preview-code-panel" aria-label=(title) {
            button type="button" class="preview-code-toggle" data-preview-full
                aria-pressed="false" aria-label=(format!("Full {title} source"))
            { "Full" }
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

pub async fn preview(_: PreviewPath) -> Markup {
    html! {
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
}

#[cfg(test)]
mod tests {
    use super::without_imports;

    #[test]
    fn strips_leading_single_and_multiline_imports() {
        let source = "\nuse vixen::{\n    fragment,\n    maud::Markup,\n};\n\nuse super::Greet;\n\n#[fragment]\nfn form() {}\n";
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
