use axum::Router;
use vixen::{After, RouterExt, action, route};

use crate::{
    models::{Category, ProductSort},
    pages::catalog::{list, page, quick_view},
    state::AppState,
};

pub fn router() -> Router<AppState> {
    Router::new()
        .view(page::catalog)
        .action(list::catalog_search)
        .view(quick_view::quick_view)
}

/// The page's actions for the root skill in `main.rs`.
pub fn actions() -> String {
    vixen::skill![SearchCatalog]
}

#[route("/")]
pub struct CatalogPath;

/// The matching product cards; a card's Quick view is GET /catalog/{slug}.
#[derive(Default)]
#[action("/catalog/search")]
pub struct SearchCatalog {
    pub category: Option<Category>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub q: String,
    #[serde(default)]
    pub sort: ProductSort,
    #[cursor]
    pub after: Option<After<i64>>,
}

#[route("/catalog/{slug}")]
pub struct QuickViewPath {
    pub slug: String,
}
