use axum::{
    Router,
    extract::Path,
    http::{StatusCode, header},
    response::IntoResponse,
    routing::get,
};
use maud::{Markup, html};

pub type Assets = &'static [(&'static str, &'static str, &'static [u8])];

pub fn router<S: Clone + Send + Sync + 'static>(files: Assets, prefix: &str) -> Router<S> {
    Router::new().route(
        &format!("/{}/{{*path}}", prefix),
        get(async |Path(path): Path<String>| {
            let idx = files.binary_search_by(|&(a, ..)| a.cmp(path.as_str()));
            match idx {
                Ok(idx) => {
                    let (_, content_type, file) = files[idx];
                    (
                        [
                            (header::CONTENT_TYPE, content_type),
                            (header::CACHE_CONTROL, "public, max-age=31536000, immutable"),
                        ],
                        file,
                    )
                        .into_response()
                }
                Err(_) => StatusCode::NOT_FOUND.into_response(),
            }
        }),
    )
}

pub fn head(base: &str, js: &str, css: Option<&str>) -> Markup {
    html! {
        script type="module" defer src={ (base) (js) } {}
        @if let Some(css) = css {
            link rel="stylesheet" type="text/css" href={ (base) (css) };
        }
    }
}
