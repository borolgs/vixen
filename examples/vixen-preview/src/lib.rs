use axum::{Router, routing::get};
use vixen::{
    RouterExt,
    maud::{Markup, html},
};

mod form;
mod greet;

#[cfg(target_arch = "wasm32")]
mod browser;

pub fn router() -> Router {
    Router::new()
        .route("/", get(preview))
        .action(greet::greet)
        .route(
            "/hello",
            get(|| async {
                tracing::info!(route = "/hello", "Request received");

                "Hello from axum, running in your browser!"
            }),
        )
}

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
        section class="preview-code-panel" {
            h2 {
                span { (title) }
                button type="button" class="preview-code-toggle" data-preview-full
                    aria-pressed="false" aria-label=(format!("Full {title} source"))
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

async fn preview() -> Markup {
    html! {
        (form::form())

        div class="preview-snippets" {
            (code_panel("Form", "maud", include_str!("form.rs")))
            (code_panel("Handler", "rust", include_str!("greet.rs")))
        }

        p class="preview-error" data-preview-error role="alert" hidden {}

        section class="preview-result-panel" {
            h2 { "Result" small data-preview-status { "Ready" } }
            div class="preview-result" role="status" aria-live="polite" {
                (greet::greeting::slot())
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
