use axum::response::Response;
use strum::IntoEnumIterator;
use vixen::{
    Page, SyncStrategy,
    maud::{Markup, html},
};

use crate::{
    models::{After, Category, ProductSort},
    pages::{
        admin_products::{
            queries::{Product, ProductQuery, search_products},
            routes::{NewProductPath, SearchProducts},
            ui::{CONFIRM, DRAWER, PRODUCTS},
        },
        shared::TOASTER,
    },
    state::ctx,
};

pub async fn products_index() -> Markup {
    let search = SearchProducts::default();
    let page = load(&search).await;

    html! {
        div class="mt-8 flex flex-wrap items-center gap-3" {
            (products_search_form(&search))
            button.btn hx-get=(NewProductPath) hx-sync=(SyncStrategy::QueueLast.on(DRAWER)) {
                "New product"
            }
        }
        div class="mt-4 overflow-x-auto" {
            table.table {
                thead {
                    tr {
                        th { "Name" }
                        th { "Slug" }
                        th { "Category" }
                        th { "Materials" }
                        th { "Price" }
                        th { "Stock" }
                        th {}
                    }
                }
                @match page {
                    Ok(page) => { (PRODUCTS.render(&search, page)) }
                    Err(_) => {
                        (PRODUCTS.shell(&search, html! {
                            tr {
                                td colspan="7" {
                                    div class="alert" data-variant="destructive" {
                                        h3 { "The shelf is empty" }
                                        section { p { "The database did not answer." } }
                                    }
                                }
                            }
                        }))
                    }
                }
            }
        }
        (DRAWER.shell())
        (CONFIRM.shell())
    }
}

pub async fn products_search(search: SearchProducts) -> Response {
    let page = load(&search)
        .await
        .map_err(|_| TOASTER.error("That didn't go through", "Try again in a moment."));

    PRODUCTS.respond(&search, page)
}

async fn load(search: &SearchProducts) -> anyhow::Result<Page<Product, After>> {
    let query = ProductQuery {
        category: search.category,
        q: search.q.clone(),
        sort: search.sort,
        after: search.after.clone(),
    };

    ctx()
        .db
        .call(move |conn| search_products(conn, &query))
        .await
        .inspect_err(|err| tracing::error!("search products: {err:#}"))
}

fn products_search_form(search: &SearchProducts) -> Markup {
    let chip = "btn has-checked:bg-primary has-checked:text-primary-foreground \
        has-focus-visible:ring-ring/50 has-focus-visible:ring-[3px]";

    html! {
        form class="flex grow flex-wrap items-center gap-3"
            hx-action=(PRODUCTS.search(SearchProducts::action()))
        {
            fieldset class="flex flex-wrap gap-2" {
                legend class="sr-only" { "Category" }
                label class=(chip) data-variant="outline" data-size="sm" {
                    input type="radio" class="sr-only" name=(SearchProducts::FIELD.category)
                        value="" checked[search.category.is_none()];
                    "All"
                }
                @for category in Category::iter() {
                    label class=(chip) data-variant="outline" data-size="sm" {
                        input type="radio" class="sr-only" name=(SearchProducts::FIELD.category)
                            value=(category.as_ref()) checked[search.category == Some(category)];
                        (category.label())
                    }
                }
            }
            input type="search" class="input w-full sm:ml-auto sm:w-64"
                name=(SearchProducts::FIELD.q) value=(search.q)
                placeholder="Search products" aria-label="Search products";
            select class="select w-48" name=(SearchProducts::FIELD.sort) aria-label="Sort" {
                @for sort in ProductSort::iter() {
                    option value=(sort.as_ref()) selected[sort == search.sort] { (sort.label()) }
                }
            }
        }
    }
}
