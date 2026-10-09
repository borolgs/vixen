use axum::response::Response;
use strum::IntoEnumIterator;
use vixen::{
    Page, SyncStrategy,
    maud::{Markup, html},
};

use crate::{
    models::{After, MaterialSort},
    pages::{
        admin_materials::{
            queries::{Material, MaterialQuery, search_materials},
            routes::{NewMaterialPath, SearchMaterials},
            ui::{CONFIRM, DRAWER, MATERIALS},
        },
        shared::TOASTER,
    },
    state::ctx,
};

pub async fn materials_index() -> Markup {
    let search = SearchMaterials::default();
    let page = load(&search).await;

    html! {
        div class="mt-8 flex flex-wrap items-center gap-3" {
            (materials_search_form(&search))
            button.btn hx-get=(NewMaterialPath) hx-sync=(SyncStrategy::QueueLast.on(DRAWER)) {
                "New material"
            }
        }
        div class="mt-4 overflow-x-auto" {
            table.table {
                thead {
                    tr {
                        th { "Name" }
                        th { "Care" }
                        th { "Products" }
                        th {}
                    }
                }
                @match page {
                    Ok(page) => { (MATERIALS.render(&search, page)) }
                    Err(_) => {
                        (MATERIALS.shell(&search, html! {
                            tr {
                                td colspan="4" {
                                    div class="alert" data-variant="destructive" {
                                        h3 { "The stores are empty" }
                                        section { p { "The database did not answer." } }
                                    }
                                }
                            }
                        }))
                    }
                }
            }
        }
        (DRAWER.shell())
        (CONFIRM.shell())
    }
}

pub async fn materials_search(search: SearchMaterials) -> Response {
    let page = load(&search)
        .await
        .map_err(|_| TOASTER.error("That didn't go through", "Try again in a moment."));

    MATERIALS.respond(&search, page)
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
