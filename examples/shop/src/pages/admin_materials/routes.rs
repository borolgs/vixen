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

#[route("/admin/materials")]
pub struct MaterialsPath;

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

#[action("/admin/materials/create")]
pub struct CreateMaterial {
    pub name: String,
    pub care: String,
}

#[action("/admin/materials/update")]
pub struct UpdateMaterial {
    pub id: i64,
    pub name: String,
    pub care: String,
}

#[action("/admin/materials/delete/confirm")]
pub struct ConfirmDeleteMaterial {
    pub id: i64,
}

#[action("/admin/materials/delete")]
pub struct DeleteMaterial {
    pub id: i64,
}
