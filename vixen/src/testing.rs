//! Helpers for inspecting response bodies in tests.

use axum::{
    body::{Body, to_bytes},
    http::Response,
};
pub use scraper;
use serde::de::DeserializeOwned;

/// Returns `true` if `selector` matches any element in `html`.
pub fn has(html: &scraper::Html, selector: &str) -> bool {
    let sel = scraper::Selector::parse(selector).unwrap();
    html.select(&sel).next().is_some()
}

/// Parses the response body as an HTML document.
pub async fn html(res: Response<Body>) -> scraper::Html {
    scraper::Html::parse_document(&text(res).await)
}

/// Collects the response body as text, replacing malformed UTF-8.
pub async fn text(res: Response<Body>) -> String {
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    String::from_utf8_lossy(&body).to_string()
}

/// Deserializes the JSON response body.
pub async fn json<T>(res: Response<Body>) -> T
where
    T: DeserializeOwned,
{
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    serde_json::from_slice::<T>(&body).unwrap()
}
