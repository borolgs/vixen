use axum::Router;
use vixen::{HxAction, PagedAction, RouterExt, action, route};

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
    #[serde(default)]
    pub q: String,
    #[serde(default)]
    pub sort: ProductSort,
    pub after: Option<After>,
}

impl PagedAction for SearchCatalog {
    type Cursor = After;

    fn cursor(&self) -> Option<After> {
        self.after.clone()
    }

    fn next(&self, after: After) -> HxAction {
        // TODO: Let `#[action]` build this from `self` via `Serialize`.
        SearchCatalog::action()
            .q(&self.q)
            .sort(self.sort)
            .maybe_category(self.category)
            .after(after)
            .hx()
    }
}

#[route("/catalog/{slug}")]
pub struct QuickViewPath {
    pub slug: String,
}
