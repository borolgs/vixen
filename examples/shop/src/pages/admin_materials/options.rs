use axum::response::Response;

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
        .inspect_err(|err| tracing::error!("material options: {err:#}"))
        .map_err(|_| TOASTER.error("That didn't go through", "Try again in a moment."));

    MATERIAL_OPTIONS.respond(&search, page)
}
