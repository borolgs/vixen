use axum::response::{IntoResponse, Response};
use strum::IntoEnumIterator;
use vixen::{
    HxAction, SyncStrategy, action, fragment,
    hx::SwapOption,
    maud::{Markup, html},
    partial,
};

use crate::{
    models::{Category, ProductSort},
    pages::{
        catalog::{
            DETAIL,
            product_art::{ArtSize, art},
            queries::{CatalogPage, Product, search_products},
            quick_view::QuickViewPath,
        },
        shared::{TOASTER, price},
    },
    state::ctx,
};

#[derive(Clone, Default)]
#[action("/catalog/search")]
pub struct SearchCatalog {
    pub category: Option<Category>,
    #[serde(default)]
    pub q: String,
    #[serde(default)]
    pub sort: ProductSort,
    pub after: Option<u32>,
}

pub async fn search(search: SearchCatalog) -> Response {
    let page = ctx()
        .db
        .call({
            let search = search.clone();
            move |conn| search_products(conn, &search)
        })
        .await;

    match (page, search.after) {
        (Ok(page), None) => partial!(catalog_grid(&page, &search)).into_response(),
        (Ok(page), Some(_)) => {
            partial!((CatalogGridId, SwapOption::BeforeEnd) => cards(&page, &search))
                .into_response()
        }
        (Err(err), after) => {
            tracing::error!("search catalog: {err:#}");

            let toast = TOASTER.error("That didn't go through", "Try again in a moment.");

            match after {
                Some(after) => partial!(
                    toast,
                    (CatalogGridId, SwapOption::BeforeEnd) => retry(&search, after),
                )
                .into_response(),
                None => toast.into_response(),
            }
        }
    }
}

// A radio fires `input` as well as `change`, and a text input fires `change` on blur.
const SEARCH_TRIGGER: &str =
    "submit, change[target.type!='search'], input[target.type=='search'] delay:300ms";

#[fragment]
pub fn catalog_search(search: &SearchCatalog) -> Markup {
    let chip = "btn has-checked:bg-primary has-checked:text-primary-foreground \
        has-focus-visible:ring-ring/50 has-focus-visible:ring-[3px]";

    html! {
        form id=(Self) class="mt-8 flex flex-wrap items-center gap-3"
            hx-action=(SearchCatalog::action().hx().trigger(SEARCH_TRIGGER).sync(SyncStrategy::Replace))
        {
            fieldset class="flex flex-wrap gap-2" {
                legend class="sr-only" { "Category" }
                label class=(chip) data-variant="outline" data-size="sm" {
                    input type="radio" class="sr-only" name=(SearchCatalog::FIELD.category)
                        value="" checked[search.category.is_none()];
                    "All"
                }
                @for category in Category::iter() {
                    label class=(chip) data-variant="outline" data-size="sm" {
                        input type="radio" class="sr-only" name=(SearchCatalog::FIELD.category)
                            value=(category.as_ref()) checked[search.category == Some(category)];
                        (category.label())
                    }
                }
            }
            input type="search" class="input w-full sm:ml-auto sm:w-64"
                name=(SearchCatalog::FIELD.q) value=(search.q)
                placeholder="Search the shelf" aria-label="Search the shelf";
            select class="select w-48" name=(SearchCatalog::FIELD.sort) aria-label="Sort" {
                @for sort in ProductSort::iter() {
                    option value=(sort.as_ref()) selected[sort == search.sort] { (sort.label()) }
                }
            }
        }
    }
}

#[fragment]
pub fn catalog_grid(page: &CatalogPage, search: &SearchCatalog) -> Markup {
    html! {
        ul id=(Self) class="mt-6 grid list-none gap-6 p-0 sm:grid-cols-2 lg:grid-cols-4" {
            @if page.cards.is_empty() {
                li class="text-muted-foreground col-span-full py-12 text-center text-sm" {
                    "Nothing on the shelf matches."
                }
            }
            (cards(page, search))
        }
    }
}

/// One page of cards, then the sentinel that asks for the next.
fn cards(page: &CatalogPage, search: &SearchCatalog) -> Markup {
    html! {
        @for product in &page.cards { (card(product)) }
        @if let Some(after) = page.next {
            li class="text-muted-foreground col-span-full py-6 text-center text-sm"
                hx-action=(next_page(search, after).trigger("intersect once"))
            {
                "Loading…"
            }
        }
    }
}

/// Stands in for the sentinel when the page `after` failed to load.
fn retry(search: &SearchCatalog, after: u32) -> Markup {
    html! {
        li class="text-muted-foreground col-span-full py-6 text-center text-sm" {
            "The rest did not load. "
            button.btn data-variant="ghost" data-size="sm"
                hx-action=(next_page(search, after).target("closest li"))
            {
                "Try again"
            }
        }
    }
}

/// The same search, continued from `after`. The row that asks deletes itself.
fn next_page(search: &SearchCatalog, after: u32) -> HxAction {
    let action = SearchCatalog::action()
        .q(&search.q)
        .sort(search.sort)
        .after(after);
    let action = match search.category {
        Some(category) => action.category(category),
        None => action,
    };

    action
        .hx()
        .swap(SwapOption::Delete)
        .sync(SyncStrategy::Abort.on(CatalogSearchId))
}

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

#[cfg(test)]
mod tests {
    use vixen::testing;

    use super::*;
    use crate::{db::Db, schema::SCHEMA, state::Ctx};

    #[tokio::test]
    async fn search_filters_by_query() {
        let db = Db::open_in_memory().unwrap();
        db.call(|conn| Ok(conn.execute_batch(SCHEMA)?))
            .await
            .unwrap();

        let query = SearchCatalog {
            q: "kettle".into(),
            ..Default::default()
        };

        let response = Ctx { db }.scope(search(query)).await;

        let doc = testing::html(response).await;

        assert!(testing::has(
            &doc,
            "#catalog-grid [hx-get='/catalog/kettle']"
        ));
        assert!(!testing::has(&doc, "#catalog-grid .card + .card"));
    }
}
