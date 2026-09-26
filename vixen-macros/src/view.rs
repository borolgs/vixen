//! `#[view_path]` — `TypedPath` + `Deserialize` in one attribute.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Error, Item, LitStr, parse_quote};

/// `#[view_path("/a/{id}")] struct P { id: i64 }` becomes
/// `#[derive(Deserialize, TypedPath)] #[typed_path("/a/{id}")]` on the same
/// struct and adds a base-path-aware `Render` implementation.
pub fn expand(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut path_struct = match syn::parse2::<Item>(item) {
        Ok(Item::Struct(s)) => s,
        Ok(other) => {
            let err = Error::new_spanned(
                &other,
                "`#[view_path]` expects a struct, e.g. `#[view_path(\"/\")] struct Index;`",
            )
            .to_compile_error();

            return quote! { #err #other };
        }
        Err(err) => return err.to_compile_error(),
    };

    let path = match syn::parse2::<LitStr>(attr) {
        Ok(path) => path,
        Err(err) => {
            let err = Error::new(
                err.span(),
                "`#[view_path]` expects a path literal, e.g. `#[view_path(\"/items/{id}\")]`",
            )
            .to_compile_error();
            return quote! { #err #path_struct };
        }
    };

    let doc = format!("Route: `{}`", path.value());
    if path_struct
        .attrs
        .iter()
        .any(|attr| attr.path().is_ident("doc"))
    {
        path_struct.attrs.push(parse_quote!(#[doc = ""]));
    }
    path_struct.attrs.push(parse_quote!(#[doc = #doc]));

    let ident = &path_struct.ident;
    let (impl_generics, ty_generics, where_clause) = path_struct.generics.split_for_impl();

    quote! {
        #[derive(::vixen::__private::serde::Deserialize, ::axum_extra::routing::TypedPath)]
        #[serde(crate = "::vixen::__private::serde")]
        #[typed_path(#path)]
        #path_struct

        impl #impl_generics ::vixen::maud::Render for #ident #ty_generics #where_clause {
            fn render_to(&self, buffer: &mut ::std::string::String) {
                ::vixen::maud::Render::render_to(&::vixen::href!(self), buffer)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derives_the_path_and_documents_the_route() {
        let out = expand(
            quote!("/"),
            quote!(
                struct Root;
            ),
        );
        let expected = quote! {
            #[derive(::vixen::__private::serde::Deserialize, ::axum_extra::routing::TypedPath)]
            #[serde(crate = "::vixen::__private::serde")]
            #[typed_path("/")]
            #[doc = "Route: `/`"]
            struct Root;

            impl ::vixen::maud::Render for Root {
                fn render_to(&self, buffer: &mut ::std::string::String) {
                    ::vixen::maud::Render::render_to(&::vixen::href!(self), buffer)
                }
            }
        };
        assert_eq!(out.to_string(), expected.to_string());
    }

    #[test]
    fn the_route_goes_under_the_docs_already_there() {
        let out = expand(
            quote!("/items/{id}"),
            quote! {
                #[doc = "An item."]
                struct Item {
                    id: u32,
                }
            },
        );
        let expected = quote! {
            #[derive(::vixen::__private::serde::Deserialize, ::axum_extra::routing::TypedPath)]
            #[serde(crate = "::vixen::__private::serde")]
            #[typed_path("/items/{id}")]
            #[doc = "An item."]
            #[doc = ""]
            #[doc = "Route: `/items/{id}`"]
            struct Item {
                id: u32,
            }

            impl ::vixen::maud::Render for Item {
                fn render_to(&self, buffer: &mut ::std::string::String) {
                    ::vixen::maud::Render::render_to(&::vixen::href!(self), buffer)
                }
            }
        };
        assert_eq!(out.to_string(), expected.to_string());
    }
}
