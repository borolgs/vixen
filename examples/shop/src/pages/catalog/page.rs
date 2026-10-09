use axum_extra::extract::Query;
use vixen::maud::{Markup, html};

use crate::pages::{
    catalog::{
        list::catalog_index,
        routes::{CatalogPath, SearchCatalog},
    },
    shared::layout,
};

pub async fn catalog(_: CatalogPath, Query(search): Query<SearchCatalog>) -> Markup {
    layout(
        "Shop",
        html! { (vixen::assets!()) },
        html! {
            h1 class="text-3xl font-semibold tracking-tight" { "The shelf" }
            p class="text-muted-foreground mt-3" {
                "Things for the kitchen and the table, made to be used for a long time."
            }

            (catalog_index(search).await)
        },
    )
}
