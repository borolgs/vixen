use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Error, parse2};

pub fn expand(item: TokenStream) -> TokenStream {
    let input = match parse2::<DeriveInput>(item) {
        Ok(input) => input,
        Err(err) => return err.to_compile_error(),
    };

    if !input.generics.params.is_empty() {
        return Error::new_spanned(&input.generics, "`ReqCtx` does not support generic types")
            .to_compile_error();
    }

    let ident = &input.ident;
    let vis = &input.vis;
    let outside =
        format!("`{ident}::current()` called outside `{ident}::middleware` and `{ident}::scope`");

    quote! {
        #[allow(dead_code)]
        impl #ident {
            fn __req_ctx() -> &'static ::vixen::__private::tokio::task::LocalKey<#ident> {
                ::vixen::__private::tokio::task_local! {
                    static REQ_CTX: #ident;
                }
                &REQ_CTX
            }

            /// Clones the current request context.
            ///
            /// # Panics
            ///
            /// Panics outside [`Self::middleware`] and [`Self::scope`].
            #[track_caller]
            #vis fn current() -> Self {
                match Self::__req_ctx().try_get() {
                    ::std::result::Result::Ok(ctx) => ctx,
                    ::std::result::Result::Err(_) => ::std::panic!(#outside),
                }
            }

            /// Runs `f` with `self` as the current request context.
            #vis async fn scope<F: ::std::future::Future>(self, f: F) -> F::Output {
                Self::__req_ctx().scope(self, f).await
            }

            /// Sets `self` as the request context, then calls `next`.
            #vis async fn middleware(
                self,
                req: ::axum::extract::Request,
                next: ::axum::middleware::Next,
            ) -> ::axum::response::Response {
                Self::__req_ctx().scope(self, next.run(req)).await
            }
        }
    }
}
