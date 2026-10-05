mod page;
mod product_art;
mod queries;
mod quick_view;
mod search;

use axum::Router;
use vixen::{RouterExt, ui::basecoatui::Drawer};

pub use page::CatalogPath;

use crate::state::AppState;

const DETAIL: Drawer = Drawer::new("catalog-detail").content_class("px-4 pb-4");

pub fn router() -> Router<AppState> {
    Router::new()
        .view(page::catalog)
        .action(search::catalog_search)
        .view(quick_view::quick_view)
}
