use vixen::{
    Page, Paged, SyncStrategy, fragment, id,
    maud::{Markup, Render, html},
    ui::basecoatui::{Combobox, Dialog, Drawer},
};

use crate::{
    models::{After, Selection},
    pages::admin_materials::{
        queries::Material,
        routes::{ConfirmDeleteMaterial, EditMaterialPath, MaterialOptions, SearchMaterials},
    },
};

pub const DRAWER: Drawer = Drawer::new("material-drawer").content_class("px-4 pb-4");

pub const CONFIRM: Dialog = Dialog::new("material-confirm");

pub const MATERIALS: Paged<SearchMaterials, Material> =
    Paged::new("material-rows", |material| material_row(material).into())
        .list(|id, rows| html! { tbody id=(id) { (rows) } })
        .empty(|_| {
            html! {
                tr {
                    td colspan="4" class="text-muted-foreground text-center" {
                        "Nothing in the stores matches."
                    }
                }
            }
        })
        .loading(|next| {
            html! {
                tr hx-action=(next) {
                    td colspan="4" class="text-muted-foreground text-center" { "Loading…" }
                }
            }
        })
        .retry(|again| {
            html! {
                tr hx-action=(again) {
                    td colspan="4" class="text-muted-foreground text-center" {
                        "The rest did not load. "
                        button.btn type="button" data-variant="ghost" data-size="sm" { "Try again" }
                    }
                }
            }
        });

#[id]
pub struct MaterialRowId(pub i64);

#[fragment(MaterialRowId(material.id))]
pub fn material_row(material: &Material) -> Markup {
    html! {
        tr id=(Self) {
            td class="font-medium" { (material.name) }
            td class="text-muted-foreground min-w-48 text-xs whitespace-normal" { (material.care) }
            td { (material.products) }
            td {
                div class="flex justify-end gap-1" {
                    button.btn data-variant="ghost" data-size="sm"
                        hx-get=(EditMaterialPath { id: material.id })
                        hx-sync=(SyncStrategy::QueueLast.on(DRAWER))
                    {
                        "Edit"
                    }
                    button.btn data-variant="ghost" data-size="sm"
                        hx-action=(ConfirmDeleteMaterial::action().id(material.id))
                    {
                        "Delete"
                    }
                }
            }
        }
    }
}

/// The combobox's options, inside basecoat's listbox.
pub const MATERIAL_OPTIONS: Paged<MaterialOptions, Material> =
    Paged::new("material-options", |material: &Material| {
        html! { div role="option" data-value=(material.id) { (material.name) } }
    })
    // Not the listbox: basecoat keeps its listeners on it.
    .list(|id, rows| html! { div id=(id) role="presentation" { (rows) } })
    .empty(|_| html! {})
    .loading(|next| {
        html! {
            div role="option" aria-disabled="true" data-value="" hx-action=(next) { "Loading…" }
        }
    })
    // Not an option: a disabled one takes no clicks.
    .retry(|again| {
        html! {
            div class="text-muted-foreground py-1.5 ps-2 text-sm" data-value="" hx-action=(again) {
                "The rest did not load. "
                button.btn type="button" data-variant="ghost" data-size="sm" { "Try again" }
            }
        }
    });

#[id]
pub struct MaterialPickerId;

/// Posts a [`Selection`] as `field`; `options` is the first page.
pub fn material_picker(
    field: &'static str,
    selected: &Selection,
    options: Page<Material, After>,
) -> Markup {
    Combobox::new(MaterialPickerId, field)
        .multiple()
        .value(selected.clone())
        .search(
            MaterialOptions::FIELD.q,
            MATERIAL_OPTIONS.search(MaterialOptions::action()),
        )
        .placeholder("Search materials")
        .empty("No materials match.")
        .class("w-full")
        .options(MATERIAL_OPTIONS.render(&MaterialOptions::default(), options))
        .render()
}
