use vixen::maud::{Markup, html};

use crate::models::Category;

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
