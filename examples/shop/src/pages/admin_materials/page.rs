use vixen::maud::{Markup, html};

use crate::pages::{
    admin_materials::{list::materials_index, routes::MaterialsPath},
    shared::layout,
};

pub async fn materials(_: MaterialsPath) -> Markup {
    layout(
        "Materials",
        html! { (vixen::assets!()) },
        html! {
            h1 class="text-3xl font-semibold tracking-tight" { "Materials" }
            p class="text-muted-foreground mt-3" {
                "What things are made of. A product picks from this list."
            }

            (materials_index().await)
        },
    )
}
