use axum::response::{IntoResponse, Response};
use strum::IntoEnumIterator;
use vixen::{
    Page, SyncStrategy, href,
    maud::{Markup, html},
    partial,
};

use crate::{
    models::{After, MaterialSort},
    pages::{
        admin_materials::{
            queries::{Material, MaterialQuery, search_materials},
            routes::{MaterialsPath, NewMaterialPath, SearchMaterials},
            ui::{CONFIRM, DRAWER, MATERIALS},
        },
        shared::TOASTER,
    },
    state::ctx,
};

pub async fn materials_index(search: SearchMaterials) -> Markup {
    let search = SearchMaterials {
        after: None,
        ..search
    };
    let page = load(&search).await;

    html! {
        div class="mt-8 flex flex-wrap items-center gap-3" {
            (materials_search_form(&search))
            button.btn hx-get=(NewMaterialPath) hx-sync=(SyncStrategy::QueueLast.on(DRAWER)) {
                "New material"
            }
        }
        div class="mt-4 overflow-x-auto" {
            table class="table min-w-3xl table-fixed" {
                thead {
                    tr {
                        th class="w-64" { "Name" }
                        th { "Care" }
                        th class="w-24" { "Products" }
                        th class="w-36" {}
                    }
                }
                (MATERIALS.view(&search, page))
            }
        }
        (DRAWER.shell())
        (CONFIRM.shell())
    }
}

pub async fn materials_search(search: SearchMaterials) -> Response {
    let page = load(&search).await;
    let toast = page
        .is_err()
        .then(|| TOASTER.error("That didn't go through", "Try again in a moment."));

    (
        MATERIALS.replace_url(&search, href!(MaterialsPath)),
        partial!(_ => MATERIALS.view(&search, page), toast),
    )
        .into_response()
}

async fn load(search: &SearchMaterials) -> anyhow::Result<Page<Material, After>> {
    let query = MaterialQuery {
        q: search.q.clone(),
        sort: search.sort,
        after: search.after.clone(),
    };

    ctx()
        .db
        .call(move |conn| search_materials(conn, &query))
        .await
        .inspect_err(|err| tracing::error!("search materials: {err:#}"))
}

fn materials_search_form(search: &SearchMaterials) -> Markup {
    html! {
        form class="flex grow flex-wrap items-center gap-3"
            hx-action=(MATERIALS.search(SearchMaterials::action()))
        {
            input type="search" class="input w-full sm:w-64"
                name=(SearchMaterials::FIELD.q) value=(search.q)
                placeholder="Search materials" aria-label="Search materials";
            select class="select w-48" name=(SearchMaterials::FIELD.sort) aria-label="Sort" {
                @for sort in MaterialSort::iter() {
                    option value=(sort.as_ref()) selected[sort == search.sort] { (sort.label()) }
                }
            }
        }
    }
}
