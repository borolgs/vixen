use axum::response::{IntoResponse, Response};
use strum::IntoEnumIterator;
use vixen::{
    Page, SyncStrategy, href,
    maud::{Markup, html},
    partial,
};

use crate::{
    models::{After, Category, ProductSort},
    pages::{
        admin_products::{
            queries::{Product, ProductQuery, search_products},
            routes::{NewProductPath, ProductsPath, SearchProducts},
            ui::{CONFIRM, DRAWER, PRODUCTS},
        },
        shared::TOASTER,
    },
    state::ctx,
};

pub async fn products_index(search: SearchProducts) -> Markup {
    let search = SearchProducts {
        after: None,
        ..search
    };
    let page = load(&search).await;

    html! {
        div class="mt-8 flex flex-wrap items-center gap-3" {
            (products_search_form(&search))
            button.btn hx-get=(NewProductPath) hx-sync=(SyncStrategy::QueueLast.on(DRAWER)) {
                "New product"
            }
        }
        div class="mt-4 overflow-x-auto" {
            table class="table min-w-5xl table-fixed" {
                thead {
                    tr {
                        th { "Name" }
                        th class="w-32" { "Slug" }
                        th class="w-28" { "Category" }
                        th class="w-52" { "Materials" }
                        th class="w-20" { "Price" }
                        th class="w-24" { "Stock" }
                        th class="w-36" {}
                    }
                }
                (PRODUCTS.view(&search, page))
            }
        }
        (DRAWER.shell())
        (CONFIRM.shell())
    }
}

pub async fn products_search(search: SearchProducts) -> Response {
    let page = load(&search).await;
    let toast = page
        .is_err()
        .then(|| TOASTER.error("That didn't go through", "Try again in a moment."));

    (
        PRODUCTS.replace_url(&search, href!(ProductsPath)),
        partial! {
            _ => PRODUCTS.view(&search, page),
            toast
        },
    )
        .into_response()
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
