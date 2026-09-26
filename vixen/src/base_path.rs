use axum::{Router, http::Uri, response::Redirect, routing::get};

/// Normalizes and validates `VIXEN_BASE_PATH` at compile time.
/// Missing, empty, and `/` values mean no base path; trailing slashes are removed.
pub const fn base_path(raw: Option<&'static str>) -> &'static str {
    let Some(raw) = raw else { return "" };
    let bytes = raw.as_bytes();
    let mut end = bytes.len();
    while end > 0 && bytes[end - 1] == b'/' {
        end -= 1;
    }
    if end == 0 {
        return "";
    }
    if bytes[0] != b'/' {
        panic!("base path must start with `/`, like `/app`");
    }
    raw.split_at(end).0
}

/// Mounts `router` at `base` and redirects bare `base` requests to `base/`.
#[track_caller]
pub fn mount<S: Clone + Send + Sync + 'static>(base: &str, router: Router<S>) -> Router<S> {
    if base.is_empty() {
        return router;
    }
    let root = format!("{base}/");
    let redirect = move |uri: Uri| async move {
        match uri.query() {
            Some(query) => Redirect::permanent(&format!("{root}?{query}")),
            None => Redirect::permanent(&root),
        }
    };
    Router::new()
        .nest(&format!("{base}/"), router)
        .route(base, get(redirect))
}

/// The app's base path, `""` when there is none.
///
/// Set it with [`Config::base_path`](crate::Config) in `build.rs`, or set
/// `VIXEN_BASE_PATH` when using [`Config::default`](crate::Config::default).
/// Trailing slashes are dropped; anything else must start with `/`.
/// Routes stay unprefixed: [`mount!`](macro@crate::mount) nests them, and
/// [`#[action]`](macro@crate::action), [`assets!`](crate::assets),
/// [`asset!`](crate::asset) and [`href!`](macro@crate::href) prefix the URLs
/// they emit.
///
/// ```
/// assert_eq!(vixen::base_path!(), "");
/// ```
#[macro_export]
macro_rules! base_path {
    () => {
        const { $crate::__private::base_path(::core::option_env!("VIXEN_BASE_PATH")) }
    };
}

/// Serves `router` under [`base_path!`]: its `/` becomes `{base}/`, and the
/// bare `{base}` redirects there. Without a base path, `router` is returned
/// as is.
///
/// ```
/// use axum::Router;
///
/// let router: Router = vixen::mount!(Router::new());
/// ```
#[macro_export]
macro_rules! mount {
    ($router:expr) => {
        $crate::__private::mount($crate::base_path!(), $router)
    };
}

#[cfg(test)]
mod tests {
    use axum::{
        body::Body,
        http::{Request, StatusCode, header::LOCATION},
        response::Response,
    };
    use tower::ServiceExt;

    use super::*;

    #[test]
    fn normalizes_the_raw_value() {
        let cases = [
            (None, ""),
            (Some(""), ""),
            (Some("/"), ""),
            (Some("/app"), "/app"),
            (Some("/app/"), "/app"),
            (Some("/app//"), "/app"),
        ];
        for (raw, want) in cases {
            assert_eq!(base_path(raw), want, "{raw:?}");
        }
    }

    #[test]
    #[should_panic(expected = "must start with `/`")]
    fn rejects_a_relative_base() {
        base_path(Some("app"));
    }

    async fn send(router: Router, uri: &str) -> Response {
        router
            .oneshot(Request::get(uri).body(Body::empty()).unwrap())
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn mounts_under_the_base_and_redirects_the_bare_base() {
        let app = || {
            Router::new()
                .route("/", get(|| async { "root" }))
                .route("/x", get(|| async { "x" }))
        };

        assert_eq!(send(mount("", app()), "/").await.status(), StatusCode::OK);

        let mounted = || mount("/app", app());
        assert_eq!(send(mounted(), "/app/").await.status(), StatusCode::OK);
        assert_eq!(send(mounted(), "/app/x").await.status(), StatusCode::OK);
        assert_eq!(send(mounted(), "/x").await.status(), StatusCode::NOT_FOUND);

        let res = send(mounted(), "/app?q=1").await;
        assert_eq!(res.status(), StatusCode::PERMANENT_REDIRECT);
        assert_eq!(res.headers()[LOCATION], "/app/?q=1");
    }
}
