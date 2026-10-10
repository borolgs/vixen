use vixen::{
    maud::{DOCTYPE, Markup, html},
    ui::basecoatui::{HEAD, Toaster},
};

pub const TOASTER: Toaster = Toaster::new();

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
            }
            body class="bg-background text-foreground min-h-svh antialiased" {
                main class="mx-auto max-w-6xl px-6 py-12" { (body) }
                (TOASTER.shell())
            }
        }
    }
}
