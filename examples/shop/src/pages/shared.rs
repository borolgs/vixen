use vixen::{
    maud::{DOCTYPE, Markup, html},
    ui::basecoatui::{HEAD, Toaster},
};

use crate::pages::catalog::CatalogPath;

pub const TOASTER: Toaster = Toaster::new();

pub fn layout(title: &str, head: Markup, body: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { (title) " · brolly" }
                (HEAD)
                (head)
            }
            body class="bg-background text-foreground min-h-svh antialiased" {
                header class="bg-background border-border sticky top-0 z-10 flex items-center gap-2 border-b px-6 py-3" {
                    a class="mr-auto font-medium tracking-tight" href=(CatalogPath) { "☂ brolly" }
                }
                main class="mx-auto max-w-6xl px-6 py-12" { (body) }
                (TOASTER.shell())
            }
        }
    }
}

/// `4200` → `"$42.00"`.
pub fn price(cents: i64) -> String {
    format!("${}.{:02}", cents / 100, cents % 100)
}
