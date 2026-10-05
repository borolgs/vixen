use vixen::{
    maud::{Markup, html},
    route,
};

use crate::pages::{catalog::search::catalog_index, shared::layout};

#[route("/")]
pub struct CatalogPath;

pub async fn catalog(_: CatalogPath) -> Markup {
    layout(
        "Shop",
        html! { (vixen::assets!()) },
        html! {
            h1 class="text-3xl font-semibold tracking-tight" { "The shelf" }
            p class="text-muted-foreground mt-3" {
                "Things for the kitchen and the table, made to be used for a long time."
            }

            (catalog_index().await)
        },
    )
}
