use axum::{
    Router,
    body::{Body, to_bytes},
    extract::{FromRef, FromRequestParts, State},
    http::Request,
    middleware,
    routing::get,
};
use tower::ServiceExt;
use vixen::ReqCtx;

#[derive(Clone, FromRef)]
struct AppState {
    shop_name: &'static str,
}

#[derive(Clone, FromRequestParts, ReqCtx)]
#[from_request(state(AppState))]
struct Ctx {
    #[from_request(via(State))]
    shop_name: &'static str,
}

// No extractor: the handler reads the context the middleware set.
async fn shop_name() -> &'static str {
    Ctx::current().shop_name
}

#[tokio::test]
async fn middleware_extracts_the_context_for_the_handler() {
    let state = AppState {
        shop_name: "brolly",
    };
    let router: Router = Router::new()
        .route("/", get(shop_name))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            Ctx::middleware,
        ))
        .with_state(state);

    let req = Request::get("/").body(Body::empty()).unwrap();
    let res = router.oneshot(req).await.unwrap();
    let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    assert_eq!(body, "brolly");
}

#[tokio::test]
async fn scope_sets_the_context_without_a_request() {
    let ctx = Ctx {
        shop_name: "brolly",
    };
    assert_eq!(ctx.scope(shop_name()).await, "brolly");
}

#[test]
#[should_panic(expected = "`Ctx::current()` called outside `Ctx::middleware` and `Ctx::scope`")]
fn current_panics_outside_a_scope() {
    Ctx::current();
}
