use axum::response::{IntoResponse, Response};
use strum::IntoEnumIterator;
use vixen::{
    Page, fragment,
    maud::{Markup, html},
    partial,
};

use crate::{
    models::{After, Category, ProductSort},
    pages::{
        catalog::{
            queries::{Product, ProductQuery, search_products},
            routes::SearchCatalog,
            ui::{CATALOG, DETAIL},
        },
        shared::TOASTER,
    },
    state::ctx,
};

pub async fn catalog_index() -> Markup {
    let search = SearchCatalog::default();
    let page = load(&search).await;

    html! {
        (catalog_search_form(&search))
        (CATALOG.view(&search, page))
        (DETAIL.shell())
    }
}

pub async fn catalog_search(search: SearchCatalog) -> Response {
    let page = load(&search).await;
    let toast = page
        .is_err()
        .then(|| TOASTER.error("That didn't go through", "Try again in a moment."));

    partial! {
        _ => CATALOG.view(&search, page),
        toast,
    }
    .into_response()
}

async fn load(search: &SearchCatalog) -> anyhow::Result<Page<Product, After>> {
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
