use vixen::{
    maud::{DOCTYPE, Markup, PreEscaped, html},
    ui::basecoatui::{HEAD, Toaster},
};

pub const TOASTER: Toaster = Toaster::new();

/// The icons rows reference with `<use href="#icon-…">`.
const ICONS: PreEscaped<&str> = PreEscaped(
    r#"<svg xmlns="http://www.w3.org/2000/svg" class="absolute size-0 overflow-hidden" aria-hidden="true"><symbol id="icon-trash" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 6h18"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6"/><path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/><path d="M10 11v6"/><path d="M14 11v6"/></symbol></svg>"#,
);

pub fn layout(title: &str, head: Markup, body: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { (title) " · vixen" }
                (HEAD)
                (head)
                (crate::SKILL.script())
            }
            body class="bg-background text-foreground min-h-svh antialiased" {
                (ICONS)
                main class="mx-auto max-w-6xl px-6 py-12" { (body) }
                (TOASTER.shell())
            }
        }
    }
}
