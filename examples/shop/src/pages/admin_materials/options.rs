use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use vixen::partial;

use crate::{
    pages::{
        admin_materials::{queries, routes::MaterialOptions, ui::MATERIAL_OPTIONS},
        shared::TOASTER,
    },
    state::ctx,
};

pub async fn material_options(search: MaterialOptions) -> Response {
    let q = search.q.clone();
    let after = search.after.clone();

    let page = ctx()
        .db
        .call(move |conn| queries::material_options(conn, &q, after.as_ref()))
        .await
        .inspect_err(|err| tracing::error!("material options: {err:#}"));

    let toast = page
        .is_err()
        .then(|| TOASTER.error("That didn't go through", "Try again in a moment."));
    let status = page.is_err().then_some(StatusCode::INTERNAL_SERVER_ERROR);

    partial! {
        _ => MATERIAL_OPTIONS.view(&search, page),
        status,
        toast,
    }
    .into_response()
}
