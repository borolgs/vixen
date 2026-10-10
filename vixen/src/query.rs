use axum::{
    extract::FromRequestParts,
    http::{StatusCode, request::Parts},
    response::{IntoResponse, Response},
};
use axum_htmx::HxCurrentUrl;
use serde::de::DeserializeOwned;

/// Deserializes query parameters from the URL in `HX-Current-URL`.
///
/// Use this when an action needs parameters written to the browser URL, for
/// example by [`Paged::replace_url`](crate::Paged::replace_url).
///
/// ```
/// use vixen::{HxCurrentQuery, action};
///
/// #[action("/todos/search")]
/// struct SearchTodos {
///     done: Option<bool>,
/// }
///
/// #[action("/todos/toggle")]
/// struct ToggleTodo {
///     id: u32,
/// }
///
/// // `HX-Current-URL: /todos?done=false`
/// async fn toggle_todo(
///     HxCurrentQuery(search): HxCurrentQuery<SearchTodos>,
///     ToggleTodo { id }: ToggleTodo,
/// ) -> String {
///     format!("{id}: {:?}", search.done)
/// }
/// # use vixen::RouterExt;
/// # let _: axum::Router = axum::Router::new().action(toggle_todo);
/// ```
///
/// A missing header or query string is treated as an empty query. Invalid
/// parameters return `400 Bad Request`. Use `Result<HxCurrentQuery<T>, Response>`
/// to handle the error in the action.
pub struct HxCurrentQuery<T>(pub T);

impl<T, S> FromRequestParts<S> for HxCurrentQuery<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Ok(HxCurrentUrl(url)) = HxCurrentUrl::from_request_parts(parts, state).await;
        let query = url.as_ref().and_then(|u| u.query()).unwrap_or_default();

        serde_html_form::from_str(query).map(Self).map_err(|err| {
            let body = format!("invalid `HX-Current-URL` query: {err}");
            (StatusCode::BAD_REQUEST, body).into_response()
        })
    }
}

#[cfg(test)]
mod tests {
    use axum::{body::to_bytes, http::Request};
    use axum_htmx::HX_CURRENT_URL;
    use serde::Deserialize;

    use super::*;

    #[derive(Debug, PartialEq, Deserialize)]
    struct Search {
        #[serde(default)]
        q: String,
        page: Option<u32>,
    }

    async fn extract(current_url: Option<&str>) -> Result<Search, (StatusCode, String)> {
        let mut req = Request::post("/todos/toggle");
        if let Some(url) = current_url {
            req = req.header(HX_CURRENT_URL, url);
        }
        let (mut parts, ()) = req.body(()).unwrap().into_parts();

        match HxCurrentQuery::from_request_parts(&mut parts, &()).await {
            Ok(HxCurrentQuery(search)) => Ok(search),
            Err(res) => {
                let status = res.status();
                let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
                Err((status, String::from_utf8(body.to_vec()).unwrap()))
            }
        }
    }

    #[tokio::test]
    async fn reads_the_query_of_the_current_url() {
        let search = extract(Some("http://localhost/todos?q=oat%20milk&page=2#top")).await;
        assert_eq!(
            search.unwrap(),
            Search {
                q: "oat milk".into(),
                page: Some(2),
            }
        );
    }

    #[tokio::test]
    async fn a_missing_header_or_query_is_an_empty_query() {
        for current_url in [None, Some("http://localhost/todos")] {
            let search = extract(current_url).await;
            assert_eq!(
                search.unwrap(),
                Search {
                    q: String::new(),
                    page: None,
                }
            );
        }
    }

    #[tokio::test]
    async fn a_query_that_does_not_fit_is_rejected() {
        let (status, body) = extract(Some("http://localhost/todos?page=two"))
            .await
            .unwrap_err();
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(body.starts_with("invalid `HX-Current-URL` query: "));
    }
}
