use axum::Router;
use vixen::{RouterExt, action, route};

use crate::{
    models::{After, Category, ProductSort},
    pages::catalog::{list, page, quick_view},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .view(page::catalog)
        .action(list::catalog_search)
        .view(quick_view::quick_view)
}

#[route("/")]
pub struct CatalogPath;

#[derive(Default)]
#[action("/catalog/search")]
pub struct SearchCatalog {
    pub category: Option<Category>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub q: String,
    #[serde(default)]
    pub sort: ProductSort,
    #[cursor]
    pub after: Option<After>,
}

#[route("/catalog/{slug}")]
pub struct QuickViewPath {
    pub slug: String,
}
