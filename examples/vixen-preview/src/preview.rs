use axum::extract::State;
use vixen::{
    maud::{Markup, html},
    route,
};

use super::{counter, hello, todos};

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
    State(count): State<counter::Counter>,
) -> Markup {
    html! {
        div class="preview-tabs" {
            div class="preview-tab-list" role="tablist" aria-label="Examples" {
                button type="button" role="tab" id="preview-tab-hello"
                    aria-controls="preview-hello" aria-selected="true"
                    data-preview-tab="hello" { "hello" }
                button type="button" role="tab" id="preview-tab-counter"
                    aria-controls="preview-counter" aria-selected="false"
                    tabindex="-1"
                    data-preview-tab="counter" { "counter" }
                button type="button" role="tab" id="preview-tab-todos"
                    aria-controls="preview-todos" aria-selected="false"
                    tabindex="-1"
                    data-preview-tab="todos" { "todos" }
                button type="button" role="tab" id="preview-tab-more"
                    aria-controls="preview-more" aria-selected="false"
                    tabindex="-1"
                    data-preview-tab="more" { "more" }
            }
        }

        section id="preview-hello" role="tabpanel"
            aria-labelledby="preview-tab-hello"
            data-preview-example="hello" class="preview-layout"
        {
            div class="preview-demo" {
                (hello::view(hello::Index).await)
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

        section id="preview-counter" role="tabpanel"
            aria-labelledby="preview-tab-counter"
            data-preview-example="counter" class="preview-layout" hidden
        {
            div class="preview-demo" {
                (counter::view(counter::Index, State(count)).await)
            }
            div class="preview-source" {
                (code_panel("counter.rs", include_str!("counter.rs")))
            }
            footer class="preview-footer" {
                span data-preview-status role="status" { "Ready" }
                span { "Rust / WASM · state resets on reload" }
            }
            p class="preview-error" data-preview-error role="alert" hidden {}
        }

        section id="preview-todos" role="tabpanel"
            aria-labelledby="preview-tab-todos"
            data-preview-example="todos" class="preview-layout" hidden
        {
            div class="preview-demo" {
                (todos::view(todos::Index).await)
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

        section id="preview-more" role="tabpanel"
            aria-labelledby="preview-tab-more"
            data-preview-example="more" class="preview-more" hidden
        {
            h2 { "More on localhost" }
            p {
                "Run the other examples locally to try real axum servers "
                "and see the asset build flow in action."
            }
            pre tabindex="0" aria-label="Run the shop example" {
                code class="language-bash" {
                    (concat!(
                        "git clone https://github.com/borolgs/vixen.git\n",
                        "cd vixen\n",
                        "bun install\n",
                        "cargo run -p shop",
                    ))
                }
            }
            a href="https://github.com/borolgs/vixen/tree/main/examples"
                target="_blank" rel="noopener noreferrer"
            {
                "Browse examples on GitHub "
                span aria-hidden="true" { "↗" }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{code_panel, todos::Ctx};

    #[tokio::test]
    async fn more_is_the_last_tab_and_links_to_local_examples() {
        let markup = Ctx {
            todos: Default::default(),
        }
        .scope(super::preview(
            super::PreviewPath,
            super::State(Default::default()),
        ))
        .await
        .into_string();
        assert!(
            markup
                .rsplit_once("data-preview-tab=\"")
                .expect("preview tabs")
                .1
                .starts_with("more\"")
        );
        assert!(markup.contains(
            "data-preview-example=\"more\" class=\"preview-more\" hidden"
        ));
        assert!(markup.contains("real axum servers"));
        assert!(markup.contains("asset build flow"));
        assert!(markup.contains(concat!(
            "git clone https://github.com/borolgs/vixen.git\n",
            "cd vixen\n",
            "bun install\n",
            "cargo run -p shop",
        )));
        assert!(markup.contains(
            "href=\"https://github.com/borolgs/vixen/tree/main/examples\""
        ));
    }

    #[tokio::test]
    async fn counter_view_reads_its_state_and_binds_both_buttons() {
        let count = super::counter::Counter::default();
        *count.lock().await = -2;
        let markup =
            super::counter::view(super::counter::Index, super::State(count))
                .await
                .into_string();

        assert!(markup.contains("id=\"counter-kind\">Even</h2>"));
        assert!(markup.contains("aria-label=\"Count\">-2</output>"));
        assert!(markup.contains("aria-label=\"Decrease\""));
        assert!(markup.contains("aria-label=\"Increase\""));
        assert!(markup.contains("/counter/add"));
        assert!(markup.contains(r#"hx-vals="{&quot;by&quot;:-1}""#));
        assert!(markup.contains(r#"hx-vals="{&quot;by&quot;:1}""#));
    }

    #[tokio::test]
    async fn todos_start_with_active_and_completed_samples() {
        let markup = Ctx {
            todos: Default::default(),
        }
        .scope(super::todos::view(super::todos::Index))
        .await
        .into_string();

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

    #[tokio::test]
    async fn hello_has_a_form_static_text_and_a_response_slot() {
        let markup =
            super::hello::view(super::hello::Index).await.into_string();
        let form = markup.find("id=\"form\"").expect("greeting form");
        let note = markup.find("Submit a name").expect("static explanation");
        let response = markup.find("id=\"greeting\"").expect("response slot");
        assert!(form < note && note < response);
        assert!(markup.contains("aria-label=\"Your name\""));
        assert!(markup.contains("aria-live=\"polite\""));
    }
}
