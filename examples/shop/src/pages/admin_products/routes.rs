use axum::Router;
use vixen::{HxAction, PagedAction, RouterExt, action, route};

use crate::{
    models::{After, Category, ProductSort},
    pages::admin_products::{delete, edit, list, page},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .view(page::products)
        .action(list::products_search)
        .view(edit::new_product)
        .view(edit::edit_product)
        .action(edit::create_product)
        .action(edit::update_product)
        .action(delete::confirm_delete_product)
        .action(delete::delete_product)
}

#[route("/admin/products")]
pub struct ProductsPath;

#[derive(Default)]
#[action("/admin/products/search")]
pub struct SearchProducts {
    pub category: Option<Category>,
    #[serde(default)]
    pub q: String,
    #[serde(default)]
    pub sort: ProductSort,
    pub after: Option<After>,
}

impl PagedAction for SearchProducts {
    type Cursor = After;

    fn cursor(&self) -> Option<After> {
        self.after.clone()
    }

    fn next(&self, after: After) -> HxAction {
        SearchProducts::action()
            .q(&self.q)
            .sort(self.sort)
            .maybe_category(self.category)
            .after(after)
            .hx()
    }
}

#[route("/admin/products/new")]
pub struct NewProductPath;

#[route("/admin/products/{id}/edit")]
pub struct EditProductPath {
    pub id: i64,
}

#[action("/admin/products/create")]
pub struct CreateProduct {
    pub slug: String,
    pub name: String,
    pub tagline: String,
    pub price: String,
    pub category: Category,
    #[serde(default)]
    pub in_stock: bool,
}

#[action("/admin/products/update")]
pub struct UpdateProduct {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub tagline: String,
    pub price: String,
    pub category: Category,
    #[serde(default)]
    pub in_stock: bool,
}

#[action("/admin/products/delete/confirm")]
pub struct ConfirmDeleteProduct {
    pub id: i64,
}

#[action("/admin/products/delete")]
pub struct DeleteProduct {
    pub id: i64,
}
