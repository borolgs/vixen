use vixen::{
    maud::{Markup, html},
    route,
};

use crate::{
    pages::{
        catalog::{
            DETAIL,
            queries::search_products,
            search::{SearchCatalog, catalog_grid, catalog_search},
        },
        shared::layout,
    },
    state::ctx,
};

#[route("/")]
pub struct CatalogPath;

pub async fn catalog(_: CatalogPath) -> Markup {
    let search = SearchCatalog::default();
    let page = ctx()
        .db
        .call({
            let search = search.clone();
            move |conn| search_products(conn, &search)
        })
        .await
        .inspect_err(|err| tracing::error!("list catalog: {err:#}"));

    layout(
        "Shop",
        html! { (vixen::assets!()) },
        html! {
            h1 class="text-3xl font-semibold tracking-tight" { "The shelf" }
            p class="text-muted-foreground mt-3" {
                "Things for the kitchen and the table, made to be used for a long time."
            }

            @match page {
                Ok(page) => {
                    (catalog_search(&search))
                    (catalog_grid(&page, &search))
                    (DETAIL.shell())
                }
                Err(_) => {
                    div class="alert mt-8" data-variant="destructive" {
                        h3 { "The shelf is empty" }
                        section { p { "The database did not answer." } }
                    }
                }
            }
        },
    )
}
