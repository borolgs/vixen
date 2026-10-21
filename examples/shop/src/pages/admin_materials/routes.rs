use axum::Router;
use vixen::{After, RouterExt, action, route};

use crate::{
    models::MaterialSort,
    pages::admin_materials::{delete, edit, list, options, page},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .view(page::materials)
        .action(list::materials_search)
        .action(options::material_options)
        .view(edit::new_material)
        .view(edit::edit_material)
        .action(edit::create_material)
        .action(edit::update_material)
        .action(delete::confirm_delete_material)
        .action(delete::delete_material)
}

/// The page's actions for the root skill in `main.rs`.
pub fn actions() -> String {
    vixen::skill![
        SearchMaterials,
        MaterialOptions,
        CreateMaterial,
        UpdateMaterial,
        ConfirmDeleteMaterial,
        DeleteMaterial,
    ]
}

#[route("/admin/materials")]
pub struct MaterialsPath;

/// The matching material rows; a row's Edit is GET /admin/materials/{id}/edit.
#[derive(Default)]
#[action("/admin/materials/search")]
pub struct SearchMaterials {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub q: String,
    #[serde(default)]
    pub sort: MaterialSort,
    #[cursor]
    pub after: Option<After<i64>>,
}

/// The matching materials as options with their ids; the product form's picker.
#[derive(Default)]
#[action("/admin/materials/options")]
pub struct MaterialOptions {
    #[serde(default)]
    pub q: String,
    #[cursor]
    pub after: Option<After<i64>>,
}

#[route("/admin/materials/new")]
pub struct NewMaterialPath;

#[route("/admin/materials/{id}/edit")]
pub struct EditMaterialPath {
    pub id: i64,
}

/// A toast; the rows reload.
#[action("/admin/materials/create")]
pub struct CreateMaterial {
    pub name: String,
    pub care: String,
}

/// The material's row and a toast.
#[action("/admin/materials/update")]
pub struct UpdateMaterial {
    pub id: i64,
    pub name: String,
    pub care: String,
}

/// A dialog saying how many products are made of it; its Delete button posts /admin/materials/delete.
#[action("/admin/materials/delete/confirm")]
pub struct ConfirmDeleteMaterial {
    pub id: i64,
}

/// Removes the row; a toast confirms.
#[action("/admin/materials/delete")]
pub struct DeleteMaterial {
    pub id: i64,
}
