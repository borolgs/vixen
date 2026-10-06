use vixen::{
    maud::{DOCTYPE, Markup, html},
    ui::basecoatui::{HEAD, Toaster},
};

use crate::pages::{admin_products::ProductsPath, catalog::CatalogPath};

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
                    a.btn data-variant="ghost" data-size="sm" href=(ProductsPath) { "Admin" }
                }
                main class="mx-auto max-w-6xl px-6 py-12" { (body) }
                (TOASTER.shell())
            }
        }
    }
}

/// `4200` → `"$42.00"`.
pub fn price(cents: i64) -> String {
    format!("${}", dollars(cents))
}

/// `4200` → `"42.00"`.
pub fn dollars(cents: i64) -> String {
    format!("{}.{:02}", cents / 100, cents % 100)
}

/// `"42"`, `"42.5"`, `"42.00"` → `Some(4200)`.
pub fn parse_dollars(input: &str) -> Option<i64> {
    let input = input.trim();
    let (whole, frac) = input.split_once('.').unwrap_or((input, ""));

    let digits = |part: &str| part.bytes().all(|b| b.is_ascii_digit());
    if whole.is_empty() && frac.is_empty() || frac.len() > 2 || !digits(whole) || !digits(frac) {
        return None;
    }

    let whole: i64 = if whole.is_empty() {
        0
    } else {
        whole.parse().ok()?
    };
    let frac: i64 = if frac.is_empty() {
        0
    } else {
        format!("{frac:0<2}").parse().ok()?
    };

    whole.checked_mul(100)?.checked_add(frac)
}
