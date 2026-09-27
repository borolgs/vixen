use axum::{Router, handler::Handler, routing::post};
use axum_extra::routing::{RouterExt as _, SecondElementIs, TypedPath};

/// Extension methods for registering typed views and actions.
///
/// ```
/// use axum::{Router, extract::State};
/// use vixen::{RouterExt, action, maud::{Markup, html}, route};
///
/// #[route("/")]
/// struct HomePath;
///
/// async fn home(_: HomePath) -> Markup {
///     html! { button hx-action=(Greet::action().name("vixen")) { "Greet" } }
/// }
///
/// #[action("/greet")]
/// struct Greet {
///     name: String,
/// }
///
/// async fn greet(State(greeting): State<&'static str>, Greet { name }: Greet) -> Markup {
///     html! { p { (greeting) ", " (name) } }
/// }
///
/// let router: Router = Router::new()
///     .view(home)
///     .action(greet)
///     .with_state("Hello");
/// ```
///
/// This trait shares its name with axum-extra's
/// [`routing::RouterExt`](crate::routing::RouterExt). Import one as `_` when
/// both are needed.
pub trait RouterExt<S> {
    /// Delegates to axum-extra's
    /// [`typed_get`](crate::routing::RouterExt::typed_get), which takes the
    /// `GET` route from the handler's first argument, a
    /// [`#[route]`](macro@crate::route) type.
    fn view<H, T, P>(self, handler: H) -> Self
    where
        H: Handler<T, S>,
        T: SecondElementIs<P> + 'static,
        P: TypedPath;

    /// The axum-extra [`typed_post`](crate::routing::RouterExt::typed_post)
    /// counterpart for [`#[action]`](macro@crate::action) handlers. Unlike
    /// `typed_post`, it takes the route from the handler's last argument, so
    /// the body-consuming action extractor stays last, as axum requires. Put
    /// extractors such as `State` before it. Use `Result<Action, Response>` to
    /// handle form rejections.
    fn action<H, T, P>(self, handler: H) -> Self
    where
        H: Handler<T, S>,
        T: LastElementIs<P> + 'static,
        P: TypedPath;
}

impl<S> RouterExt<S> for Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    fn view<H, T, P>(self, handler: H) -> Self
    where
        H: Handler<T, S>,
        T: SecondElementIs<P> + 'static,
        P: TypedPath,
    {
        self.typed_get(handler)
    }

    fn action<H, T, P>(self, handler: H) -> Self
    where
        H: Handler<T, S>,
        T: LastElementIs<P> + 'static,
        P: TypedPath,
    {
        self.route(P::PATH, post(handler))
    }
}

/// Identifies a handler tuple ending in `P`, `Option<P>`, or `Result<P, E>`.
/// This sealed trait supports [`RouterExt::action`].
pub trait LastElementIs<P>: sealed::Sealed {}

mod sealed {
    pub trait Sealed {}
}

macro_rules! impl_last_element_is {
    ( $($ty:ident),* ) => {
        impl<M, $($ty,)* L> sealed::Sealed for (M, $($ty,)* L) {}

        impl<M, $($ty,)* P: TypedPath> LastElementIs<P> for (M, $($ty,)* P) {}
        impl<M, $($ty,)* P: TypedPath> LastElementIs<P> for (M, $($ty,)* Option<P>) {}
        impl<M, $($ty,)* P: TypedPath, E> LastElementIs<P> for (M, $($ty,)* Result<P, E>) {}
    };
}

// Match axum's `Handler` limit of 16 extractors.
impl_last_element_is!();
impl_last_element_is!(T1);
impl_last_element_is!(T1, T2);
impl_last_element_is!(T1, T2, T3);
impl_last_element_is!(T1, T2, T3, T4);
impl_last_element_is!(T1, T2, T3, T4, T5);
impl_last_element_is!(T1, T2, T3, T4, T5, T6);
impl_last_element_is!(T1, T2, T3, T4, T5, T6, T7);
impl_last_element_is!(T1, T2, T3, T4, T5, T6, T7, T8);
impl_last_element_is!(T1, T2, T3, T4, T5, T6, T7, T8, T9);
impl_last_element_is!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
impl_last_element_is!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
impl_last_element_is!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
impl_last_element_is!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13);
impl_last_element_is!(T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14);
impl_last_element_is!(
    T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15
);

#[cfg(test)]
mod tests {
    use axum::{
        body::{Body, to_bytes},
        extract::State,
        http::{Method, Request, StatusCode, header::CONTENT_TYPE},
        response::Response,
    };
    use tower::ServiceExt;

    use super::*;
    use crate::{action, route};

    #[route("/items/{id}")]
    struct ItemPath {
        id: u32,
    }

    async fn item(ItemPath { id }: ItemPath, State(greeting): State<&'static str>) -> String {
        format!("{greeting} #{id}")
    }

    #[action("/greet")]
    struct Greet {
        name: String,
    }

    async fn greet(State(greeting): State<&'static str>, Greet { name }: Greet) -> String {
        format!("{greeting}, {name}")
    }

    #[action("/rename")]
    struct Rename {
        name: String,
    }

    async fn rename(
        State(greeting): State<&'static str>,
        form: Result<Rename, Response>,
    ) -> String {
        match form {
            Ok(Rename { name }) => format!("{greeting}, {name}"),
            Err(_) => "name is required".into(),
        }
    }

    async fn send(method: Method, uri: &str, form: &str) -> (StatusCode, String) {
        let router: Router = Router::new()
            .view(item)
            .action(greet)
            .action(rename)
            .with_state("Hi");
        let req = Request::builder()
            .method(method)
            .uri(uri)
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(Body::from(form.to_owned()))
            .unwrap();
        let res = router.oneshot(req).await.unwrap();
        let status = res.status();
        let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
        (status, String::from_utf8(body.to_vec()).unwrap())
    }

    #[tokio::test]
    async fn view_routes_get_by_the_first_argument() {
        let (status, body) = send(Method::GET, "/items/7", "").await;
        assert_eq!((status, body.as_str()), (StatusCode::OK, "Hi #7"));

        let (status, _) = send(Method::POST, "/items/7", "").await;
        assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);
    }

    #[tokio::test]
    async fn action_routes_post_by_the_last_argument() {
        let (status, body) = send(Method::POST, "/greet", "name=vixen").await;
        assert_eq!((status, body.as_str()), (StatusCode::OK, "Hi, vixen"));

        let (status, _) = send(Method::GET, "/greet", "").await;
        assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);
    }

    #[tokio::test]
    async fn action_accepts_a_result_wrapped_action() {
        let (status, body) = send(Method::POST, "/rename", "name=vixen").await;
        assert_eq!((status, body.as_str()), (StatusCode::OK, "Hi, vixen"));

        let (status, body) = send(Method::POST, "/rename", "").await;
        assert_eq!(
            (status, body.as_str()),
            (StatusCode::OK, "name is required")
        );
    }
}
