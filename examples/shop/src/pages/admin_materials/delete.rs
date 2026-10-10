use axum::response::{IntoResponse, Response};
use vixen::{HxPartialResponse, hx::SwapOption, maud::html, partial, ui::basecoatui::Dialog};

use crate::{
    pages::{
        admin_materials::{
            queries,
            routes::{ConfirmDeleteMaterial, DeleteMaterial},
            ui::{CONFIRM, MaterialRowId},
        },
        shared::TOASTER,
    },
    state::ctx,
};

pub async fn confirm_delete_material(
    ConfirmDeleteMaterial { id }: ConfirmDeleteMaterial,
) -> Response {
    let material = ctx()
        .db
        .call(move |conn| queries::material_by_id(conn, id))
        .await;

    match material {
        Ok(Some(material)) => CONFIRM
            .header(html! {
                h2 { "Delete " (material.name) "?" }
                p {
                    @if material.products > 0 {
                        (material.products) " product(s) lose it. This cannot be undone."
                    } @else {
                        "Nothing is made of it. This cannot be undone."
                    }
                }
            })
            .footer(html! {
                button.btn data-variant="outline" type="button"
                    onclick=(Dialog::cancel()) { "Cancel" }
                button.btn data-variant="destructive" type="button"
                    hx-action=(DeleteMaterial::action().id(material.id)) { "Delete" }
            })
            .into_response(),
        Ok(None) => CONFIRM
            .header(html! {
                h2 { "Not found" }
                p { "Someone got here first." }
            })
            .footer(html! {
                button.btn data-variant="outline" type="button"
                    onclick=(Dialog::cancel()) { "Close" }
            })
            .into_response(),
        Err(err) => {
            tracing::error!("confirm delete material {id}: {err:#}");

            TOASTER
                .error("That didn't go through", "Try again in a moment.")
                .into_response()
        }
    }
}

pub async fn delete_material(DeleteMaterial { id }: DeleteMaterial) -> HxPartialResponse {
    let deleted = ctx()
        .db
        .call(move |conn| queries::delete_material(conn, id))
        .await;

    match deleted {
        Ok(_) => partial! {
            CONFIRM.close(),
            (MaterialRowId(id), SwapOption::Delete) => html! {},
            TOASTER.success("Deleted", "It is out of the stores."),
        },
        Err(err) => {
            tracing::error!("delete material {id}: {err:#}");

            TOASTER
                .error("That didn't go through", "Try again in a moment.")
                .into()
        }
    }
}
