use axum::response::{IntoResponse, Response};
use vixen::{
    maud::{Markup, html},
    route,
};

use crate::{
    pages::{
        catalog::{
            DETAIL,
            product_art::{ArtSize, art},
            queries::{Material, Product, product_by_slug, product_materials},
        },
        shared::{TOASTER, price},
    },
    state::ctx,
};

#[route("/catalog/{slug}")]
pub struct QuickViewPath {
    pub slug: String,
}

pub async fn quick_view(QuickViewPath { slug }: QuickViewPath) -> Response {
    let found = ctx()
        .db
        .call({
            let slug = slug.clone();
            move |conn| {
                let Some(product) = product_by_slug(conn, &slug)? else {
                    return Ok(None);
                };
                let materials = product_materials(conn, product.id)?;

                Ok(Some((product, materials)))
            }
        })
        .await;

    match found {
        Ok(Some((product, materials))) => DETAIL
            .header(html! {
                h2 { (product.name) }
                p { (product.category.label()) " · " (price(product.price_cents)) }
            })
            .content(product_details(&product, &materials))
            .footer(html! {
                @if product.in_stock {
                    span.badge data-variant="secondary" { "In stock" }
                } @else {
                    span.badge data-variant="outline" { "Sold out" }
                }
                (close_button())
            })
            .into_response(),
        Ok(None) => DETAIL
            .header(html! {
                h2 { "Not found" }
                p { "It is not on the shelf any more." }
            })
            .content(html! {
                div.alert data-variant="destructive" {
                    h3 { "No such product" }
                    section { p { "It may have been taken down. Close this and refresh." } }
                }
            })
            .footer(close_button())
            .into_response(),
        Err(err) => {
            tracing::error!("quick view {slug}: {err:#}");

            TOASTER
                .error("That didn't go through", "Try again in a moment.")
                .into_response()
        }
    }
}

fn product_details(product: &Product, materials: &[Material]) -> Markup {
    html! {
        (art(product.category, ArtSize::QuickView))
        p class="mt-4 text-balance" { (product.tagline) }

        h3 class="mt-8 text-sm font-medium" { "Made of" }
        @if materials.is_empty() {
            p class="text-muted-foreground mt-2 text-sm" { "Nobody wrote it down." }
        } @else {
            dl class="mt-2 grid gap-2 text-sm" {
                @for material in materials {
                    div {
                        dt class="font-medium" { (material.name) }
                        dd class="text-muted-foreground" { (material.care) }
                    }
                }
            }
        }
    }
}

fn close_button() -> Markup {
    html! {
        button.btn data-variant="outline" type="button"
            onclick="this.closest('dialog').close()" { "Close" }
    }
}
