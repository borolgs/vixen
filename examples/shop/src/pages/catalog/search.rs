use axum::response::Response;
use strum::IntoEnumIterator;
use vixen::{
    HxAction, Page, Paged, PagedAction, SyncStrategy, action, fragment,
    maud::{Markup, html},
};

use crate::{
    models::{Category, ProductSort},
    pages::{
        catalog::{
            DETAIL,
            product_art::{ArtSize, art},
            queries::{Product, search_products},
            quick_view::QuickViewPath,
        },
        shared::{TOASTER, price},
    },
    state::ctx,
};

pub async fn catalog_index() -> Markup {
    let search = SearchCatalog::default();
    let page = load(&search).await;

    html! {
        (catalog_search_form(&search))
        @match page {
            Ok(page) => { (CATALOG.render(&search, page)) }
            Err(_) => {
                (CATALOG.shell(html! {
                    li class="col-span-full" {
                        div class="alert" data-variant="destructive" {
                            h3 { "The shelf is empty" }
                            section { p { "The database did not answer." } }
                        }
                    }
                }))
            }
        }
        (DETAIL.shell())
    }
}

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

impl PagedAction for SearchCatalog {
    type Cursor = u32;

    fn cursor(&self) -> Option<u32> {
        self.after
    }

    fn next(&self, after: u32) -> HxAction {
        // TODO: Let `#[action]` build this from `self` via `Serialize`.
        SearchCatalog::action()
            .q(&self.q)
            .sort(self.sort)
            .maybe_category(self.category)
            .after(after)
            .hx()
    }
}

const CATALOG: Paged<SearchCatalog, Product> = Paged::new("catalog-grid", card)
    .list(|id, rows| {
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

pub async fn catalog_search(search: SearchCatalog) -> Response {
    let page = load(&search)
        .await
        .map_err(|_| TOASTER.error("That didn't go through", "Try again in a moment."));

    CATALOG.respond(&search, page)
}

async fn load(search: &SearchCatalog) -> anyhow::Result<Page<Product, u32>> {
    ctx()
        .db
        .call({
            let search = search.clone();
            move |conn| search_products(conn, &search)
        })
        .await
        .inspect_err(|err| tracing::error!("search catalog: {err:#}"))
}

#[fragment]
pub fn catalog_search_form(search: &SearchCatalog) -> Markup {
    let chip = "btn has-checked:bg-primary has-checked:text-primary-foreground \
        has-focus-visible:ring-ring/50 has-focus-visible:ring-[3px]";

    html! {
        form id=(Self) class="mt-8 flex flex-wrap items-center gap-3"
            hx-action=(CATALOG.search(SearchCatalog::action()))
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

        let response = Ctx { db }.scope(catalog_search(query)).await;

        let doc = testing::html(response).await;

        assert!(testing::has(
            &doc,
            "#catalog-grid [hx-get='/catalog/kettle']"
        ));
        assert!(!testing::has(&doc, "#catalog-grid .card + .card"));
    }
}
