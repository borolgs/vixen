use vixen::{
    Paged, SyncStrategy, fragment, id,
    maud::{Markup, html},
    ui::basecoatui::{Dialog, Drawer},
};

use crate::pages::{
    admin_products::{
        queries::Product,
        routes::{ConfirmDeleteProduct, EditProductPath, SearchProducts},
    },
    shared::price,
};

pub const DRAWER: Drawer = Drawer::new("product-drawer").content_class("px-4 pb-4");

pub const CONFIRM: Dialog = Dialog::new("product-confirm");

pub const PRODUCTS: Paged<SearchProducts, Product> =
    Paged::new("product-rows", |product| product_row(product).into())
        .list(|id, rows| html! { tbody id=(id) { (rows) } })
        .empty(|_| {
            html! {
                tr {
                    td colspan="7" class="text-muted-foreground text-center" {
                        "Nothing on the shelf matches."
                    }
                }
            }
        })
        .failed(|_| {
            html! {
                tr {
                    td colspan="7" {
                        div class="alert" data-variant="destructive" {
                            h3 { "The shelf is empty" }
                            section { p { "The database did not answer." } }
                        }
                    }
                }
            }
        })
        .loading(|next| {
            html! {
                tr hx-action=(next) {
                    td colspan="7" class="text-muted-foreground text-center" { "Loading…" }
                }
            }
        })
        .retry(|again| {
            html! {
                tr hx-action=(again) {
                    td colspan="7" class="text-muted-foreground text-center" {
                        "The rest did not load. "
                        button.btn type="button" data-variant="ghost" data-size="sm" { "Try again" }
                    }
                }
            }
        });

#[id]
pub struct ProductRowId(pub i64);

#[fragment(ProductRowId(product.id))]
pub fn product_row(product: &Product) -> Markup {
    html! {
        tr id=(Self) {
            td class="min-w-56 whitespace-normal" {
                div class="font-medium" { (product.name) }
                div class="text-muted-foreground text-xs" { (product.tagline) }
            }
            td { code { (product.slug) } }
            td { span.badge data-variant="outline" { (product.category.label()) } }
            td class="text-muted-foreground min-w-32 text-xs whitespace-normal" {
                @if product.materials.is_empty() { "—" } @else { (product.materials) }
            }
            td { (price(product.price_cents)) }
            td {
                @if product.in_stock {
                    span.badge data-variant="secondary" { "In stock" }
                } @else {
                    span.badge data-variant="outline" { "Sold out" }
                }
            }
            td {
                div class="flex justify-end gap-1" {
                    button.btn data-variant="ghost" data-size="sm"
                        hx-get=(EditProductPath { id: product.id })
                        hx-sync=(SyncStrategy::QueueLast.on(DRAWER))
                    {
                        "Edit"
                    }
                    button.btn data-variant="ghost" data-size="sm"
                        hx-action=(ConfirmDeleteProduct::action().id(product.id))
                    {
                        "Delete"
                    }
                }
            }
        }
    }
}
