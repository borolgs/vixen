use vixen::{
    Paged, SyncStrategy,
    maud::{Markup, html},
    ui::basecoatui::Drawer,
};

use crate::{
    models::Category,
    pages::{
        catalog::{
            queries::Product,
            routes::{QuickViewPath, SearchCatalog},
        },
        shared::price,
    },
};

pub const DETAIL: Drawer = Drawer::new("catalog-detail").content_class("px-4 pb-4");

pub const CATALOG: Paged<SearchCatalog, Product> = Paged::new("catalog-grid", card)
    .list(|id, _, rows| {
        html! {
            ul id=(id) class="mt-6 grid list-none gap-6 p-0 sm:grid-cols-2 lg:grid-cols-4" {
                (rows)
            }
        }
    })
    .empty(|_| {
        html! {
            li class="text-muted-foreground col-span-full py-12 text-center text-sm" {
                "Nothing on the shelf matches."
            }
        }
    })
    .failed(|_| {
        html! {
            li class="col-span-full" {
                div class="alert" data-variant="destructive" {
                    h3 { "The shelf is empty" }
                    section { p { "The database did not answer." } }
                }
            }
        }
    })
    .loading(|next| {
        html! {
            li class="text-muted-foreground col-span-full py-6 text-center text-sm"
                hx-action=(next)
            {
                "Loading…"
            }
        }
    })
    .retry(|again| {
        html! {
            li class="text-muted-foreground col-span-full py-6 text-center text-sm"
                hx-action=(again)
            {
                "The rest did not load. "
                button.btn type="button" data-variant="ghost" data-size="sm" { "Try again" }
            }
        }
    });

fn card(product: &Product) -> Markup {
    html! {
        li class="card gap-4 overflow-hidden pt-0" {
            (art(product.category, ArtSize::Card))
            header {
                h3 { (product.name) }
                p class="line-clamp-2" { (product.tagline) }
            }
            section class="flex items-center gap-2" {
                span.badge data-variant="outline" { (product.category.label()) }
                @if !product.in_stock {
                    span.badge data-variant="secondary" { "Sold out" }
                }
            }
            footer class="mt-auto items-center justify-between gap-2" {
                span class="mr-auto font-medium" { (price(product.price_cents)) }
                button.btn data-variant="outline" data-size="sm"
                    hx-get=(QuickViewPath { slug: product.slug.clone() })
                    hx-sync=(SyncStrategy::QueueLast.on(DETAIL))
                {
                    "Quick view"
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
pub enum ArtSize {
    Card,
    QuickView,
}

/// What stands in for a photo: a block tinted by category, with its glyph.
pub fn art(category: Category, size: ArtSize) -> Markup {
    // Whole literals: tailwind scans this file for class names.
    let tint = match category {
        Category::Kitchen => "bg-amber-100 text-amber-900 dark:bg-amber-950 dark:text-amber-200",
        Category::Textiles => "bg-rose-100 text-rose-900 dark:bg-rose-950 dark:text-rose-200",
        Category::Tableware => "bg-sky-100 text-sky-900 dark:bg-sky-950 dark:text-sky-200",
        Category::Woodwork => {
            "bg-emerald-100 text-emerald-900 dark:bg-emerald-950 dark:text-emerald-200"
        }
    };
    let (frame, glyph) = match size {
        ArtSize::Card => ("aspect-[4/3]", "size-16"),
        ArtSize::QuickView => ("aspect-[16/9] rounded-lg", "size-20"),
    };

    html! {
        div class={ "flex items-center justify-center " (frame) " " (tint) } aria-hidden="true" {
            // Closed with `{}`: an unclosed `<path>` swallows its siblings in SVG.
            svg class=(glyph) viewBox="0 0 24 24" fill="none" stroke="currentColor"
                stroke-width="1.25" stroke-linecap="round" stroke-linejoin="round"
            {
                @match category {
                    // a pot, steaming
                    Category::Kitchen => {
                        path d="M5 10h14v6a3 3 0 0 1-3 3H8a3 3 0 0 1-3-3z" {}
                        path d="M3 10h18" {}
                        path d="M9.5 7c0-1.2 1-1.3 1-2.5M13.5 7c0-1.2 1-1.3 1-2.5" {}
                    }
                    // an apron
                    Category::Textiles => {
                        path d="M8 3v2.5a4 4 0 0 0 8 0V3" {}
                        path d="M8 5.5 5 10.5V21h14V10.5l-3-5" {}
                        path d="M9.5 13.5h5v3.5h-5z" {}
                    }
                    // a mug
                    Category::Tableware => {
                        path d="M5 8h11v8a4 4 0 0 1-4 4H9a4 4 0 0 1-4-4z" {}
                        path d="M16 10.5h1.5a2.25 2.25 0 0 1 0 4.5H16" {}
                        path d="M8.5 5V3.5M12.5 5V3.5" {}
                    }
                    // a board, with its grain
                    Category::Woodwork => {
                        rect x="3" y="7" width="18" height="11" rx="2" {}
                        circle cx="6.5" cy="12.5" r="1" {}
                        path d="M10 11c2.5 1 5 1 8 0M10 14.5c2.5 1 5 1 8 0" {}
                    }
                }
            }
        }
    }
}
