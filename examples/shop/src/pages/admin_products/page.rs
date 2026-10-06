use vixen::maud::{Markup, html};

use crate::pages::{
    admin_products::{list::products_index, routes::ProductsPath},
    shared::layout,
};

pub async fn products(_: ProductsPath) -> Markup {
    layout(
        "Products",
        html! { (vixen::assets!()) },
        html! {
            h1 class="text-3xl font-semibold tracking-tight" { "Products" }
            p class="text-muted-foreground mt-3" {
                "Everything on the shelf. Edit in place, or add something new."
            }

            (products_index().await)
        },
    )
}
