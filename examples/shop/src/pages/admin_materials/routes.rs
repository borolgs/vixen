use axum::Router;
use vixen::{HxAction, PagedAction, RouterExt, action, route};

use crate::{
    models::{After, MaterialSort},
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
    #[serde(default)]
    pub q: String,
    #[serde(default)]
    pub sort: MaterialSort,
    pub after: Option<After>,
}

impl PagedAction for SearchMaterials {
    type Cursor = After;

    fn cursor(&self) -> Option<After> {
        self.after.clone()
    }

    fn next(&self, after: After) -> HxAction {
        SearchMaterials::action()
            .q(&self.q)
            .sort(self.sort)
            .after(after)
            .hx()
    }
}

#[derive(Default)]
#[action("/admin/materials/options")]
pub struct MaterialOptions {
    #[serde(default)]
    pub q: String,
    pub after: Option<After>,
}

impl PagedAction for MaterialOptions {
    type Cursor = After;

    fn cursor(&self) -> Option<After> {
        self.after.clone()
    }

    fn next(&self, after: After) -> HxAction {
        MaterialOptions::action().q(&self.q).after(after).hx()
    }
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
