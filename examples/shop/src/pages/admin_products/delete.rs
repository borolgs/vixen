use axum::response::{IntoResponse, Response};
use vixen::{
    hx::{HxResponseTrigger, SwapOption},
    maud::html,
    partial,
};

use crate::{
    pages::{
        admin_products::{
            queries,
            routes::{ConfirmDeleteProduct, DeleteProduct},
            ui::{CONFIRM, ProductRowId},
        },
        shared::TOASTER,
    },
    state::ctx,
};

pub async fn confirm_delete_product(ConfirmDeleteProduct { id }: ConfirmDeleteProduct) -> Response {
    let product = ctx()
        .db
        .call(move |conn| queries::product_by_id(conn, id))
        .await;

    match product {
        Ok(Some(product)) => CONFIRM
            .header(html! {
                h2 { "Delete " (product.name) "?" }
                p { "It comes off the shelf for good. This cannot be undone." }
            })
            .footer(html! {
                button.btn data-variant="outline" type="button"
                    onclick="this.closest('dialog').close()" { "Cancel" }
                button.btn data-variant="destructive" type="button"
                    hx-action=(DeleteProduct::action().id(product.id)) { "Delete" }
            })
            .into_response(),
        Ok(None) => CONFIRM
            .header(html! {
                h2 { "Not found" }
                p { "Someone got here first." }
            })
            .footer(html! {
                button.btn data-variant="outline" type="button"
                    onclick="this.closest('dialog').close()" { "Close" }
            })
            .into_response(),
        Err(err) => {
            tracing::error!("confirm delete product {id}: {err:#}");

            TOASTER
                .error("That didn't go through", "Try again in a moment.")
                .into_response()
        }
    }
}

pub async fn delete_product(DeleteProduct { id }: DeleteProduct) -> Response {
    let deleted = ctx()
        .db
        .call(move |conn| queries::delete_product(conn, id))
        .await;

    match deleted {
        Ok(_) => (
            HxResponseTrigger::normal([CONFIRM.close()]),
            partial!(
                (ProductRowId(id), SwapOption::Delete) => html! {},
                TOASTER.success("Deleted", "It came off the shelf."),
            ),
        )
            .into_response(),
        Err(err) => {
            tracing::error!("delete product {id}: {err:#}");

            TOASTER
                .error("That didn't go through", "Try again in a moment.")
                .into_response()
        }
    }
}
