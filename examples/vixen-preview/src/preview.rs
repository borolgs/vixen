use axum::extract::State;
use vixen::{
    maud::{Markup, html},
    route,
};

use super::{counter, hello, todos};

fn demo_panel(content: Markup) -> Markup {
    html! {
        div class="preview-demo-panel" {
            header class="preview-demo-header" {
                span data-preview-status role="status" {}
            }
            div class="preview-demo" { (content) }
        }
    }
}

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
            (demo_panel(hello::view(hello::Index).await))
            div class="preview-source" {
                (code_panel("hello.rs", include_str!("hello.rs")))
            }
            p class="preview-error" data-preview-error role="alert" hidden {}
        }

        section id="preview-counter" role="tabpanel"
            aria-labelledby="preview-tab-counter"
            data-preview-example="counter" class="preview-layout" hidden
        {
            (demo_panel(counter::view(counter::Index, State(count)).await))
            div class="preview-source" {
                (code_panel("counter.rs", include_str!("counter.rs")))
            }
            p class="preview-error" data-preview-error role="alert" hidden {}
        }

        section id="preview-todos" role="tabpanel"
            aria-labelledby="preview-tab-todos"
            data-preview-example="todos" class="preview-layout" hidden
        {
            (demo_panel(todos::view(todos::Index).await))
            div class="preview-source" {
                (code_panel("todos.rs", include_str!("todos.rs")))
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
