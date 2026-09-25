//! `#[action]` — one declaration per htmx endpoint.

use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote, quote_spanned};
use syn::ext::IdentExt;
use syn::{Error, Fields, GenericArgument, ItemStruct, PathArguments, Type};
use syn::{LitStr, parse_quote};

/// One declaration per htmx endpoint: a struct that names its route type and
/// lists the fields the request carries.
pub fn expand(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut action_struct = match syn::parse2::<ItemStruct>(item) {
        Ok(action_struct) => {
            if !action_struct.generics.params.is_empty() {
                let err = Error::new_spanned(
                    &action_struct.generics,
                    "`#[action]` does not support generic structs",
                )
                .to_compile_error();
                return quote! { #err #action_struct };
            }

            action_struct
        }
        Err(err) => {
            let err = Error::new(err.span(), "`#[action]` expects a struct with named fields")
                .to_compile_error();
            return quote! { #err };
        }
    };

    let path = match syn::parse2::<LitStr>(attr) {
        Ok(path) => path,
        Err(err) => {
            let err = Error::new(
                err.span(),
                r#"`#[action]` expects a path literal, e.g. `#[action("/feedback")]`"#,
            )
            .to_compile_error();
            return quote! { #err #action_struct };
        }
    };

    let action_ident = action_struct.ident.clone();
    let vis = &action_struct.vis;

    // structs
    let span = action_ident.span();
    let action_path_ident = format_ident!("{}Path", action_ident, span = span);
    let action_builder_ident = format_ident!("{}Action", action_ident, span = span);
    let action_fields_ident = format_ident!("{}Fields", action_ident, span = span);

    // field_name: "field_name"
    let mut action_field_idents = Vec::new();
    let mut action_field_keys = Vec::new();

    // fn field_name(val: FieldType) {}
    let mut action_setters = Vec::new();

    let Fields::Named(ref action_fields) = action_struct.fields else {
        let err = Error::new(
            action_ident.span(),
            "`#[action]` expects a struct with named fields",
        )
        .to_compile_error();
        return quote! { #err #action_struct };
    };

    for field in action_fields.named.iter() {
        let field_name = &field.ident;
        let field_type = &field.ty;

        let Some(field_name) = field_name else {
            continue;
        };

        let key = LitStr::new(&field_name.unraw().to_string(), field_name.span());

        action_field_idents.push(field_name.clone());
        action_field_keys.push(key.clone());

        // (val: #field_type, kv(key, #val_expr))
        // TODO: Recognize only unqualified or canonical standard-library `String` /
        // `Option<T>` paths (with exactly one type argument for `Option`); matching
        // only the final segment also catches qualified user-defined types.
        let (field_type, val_expr) = match field_type {
            // String -> impl Into<String>
            Type::Path(type_path)
                if type_path
                    .path
                    .segments
                    .last()
                    .is_some_and(|seg| seg.ident == "String" && seg.arguments.is_empty()) =>
            {
                (
                    quote! { impl ::std::convert::Into<::std::string::String> },
                    quote! { ::vixen::__private::serde_json::Value::String(::std::convert::Into::into(val)) },
                )
            }
            // Option<T> -> T
            Type::Path(type_path)
                if let Some(seg) = type_path.path.segments.last()
                    && seg.ident == "Option"
                    && let PathArguments::AngleBracketed(ref args) = seg.arguments
                    && let Some(GenericArgument::Type(arg_type)) = args.args.first() =>
            {
                match arg_type {
                    // Option<String> -> impl Into<String>
                    Type::Path(type_path)
                        if type_path.path.segments.last().is_some_and(|seg| {
                            seg.ident == "String" && seg.arguments.is_empty()
                        }) =>
                    {
                        (
                            quote! { impl ::std::convert::Into<::std::string::String> },
                            quote! { ::vixen::__private::serde_json::Value::String(::std::convert::Into::into(val)) },
                        )
                    }
                    _ => (
                        arg_type.to_token_stream(),
                        quote! { ::vixen::__private::serde_json::to_value(val).unwrap_or(::vixen::__private::serde_json::Value::Null) },
                    ),
                }
            }
            _ => (
                field_type.to_token_stream(),
                quote! { ::vixen::__private::serde_json::to_value(val).unwrap_or(::vixen::__private::serde_json::Value::Null) },
            ),
        };

        let setter = quote_spanned! { field_name.span() =>
            #[allow(unused)]
            #vis fn #field_name(mut self, val: #field_type) -> Self {
                self.0 = self.0.val(#key, #val_expr);
                self
            }
        };

        action_setters.push(setter);
    }

    let doc = format!("Route: `POST {}`", path.value());
    let builder_doc = format!(
        "`hx-action` attribute for [`{0}`]. Setters fill `hx-vals`; `.hx()` continues with htmx options.",
        action_ident
    );
    if action_struct.attrs.iter().any(|a| a.path().is_ident("doc")) {
        action_struct.attrs.push(parse_quote! {#[doc = ""]});
    }
    action_struct.attrs.push(parse_quote! {#[doc = #doc]});

    quote! {
        #[derive(
            ::vixen::__private::serde::Deserialize,
            ::vixen::__private::serde::Serialize,
            ::axum::extract::FromRequest,
        )]
        #[serde(crate = "::vixen::__private::serde")]
        #[from_request(via(::axum_extra::extract::Form))]
        #action_struct

        #[derive(::vixen::__private::serde::Deserialize, ::axum_extra::routing::TypedPath)]
        #[serde(crate = "::vixen::__private::serde")]
        #[typed_path(#path)]
        #vis struct #action_path_ident;

        // TypedPath's glue

        impl ::axum_extra::routing::TypedPath for #action_ident {
            const PATH: &'static str =  <#action_path_ident as ::axum_extra::routing::TypedPath>::PATH;
        }

        impl ::std::fmt::Display for #action_ident {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                ::std::fmt::Display::fmt(&#action_path_ident, f)
            }
        }

        #[doc = #builder_doc]
        #vis struct #action_builder_ident(::vixen::HxAction);

        impl #action_builder_ident {
            #(#action_setters)*

            /// Continues with htmx options: `trigger`, `target`, `swap`, `sync`.
            #vis fn hx(self) -> ::vixen::HxAction {
                self.0
            }
        }

        impl ::std::convert::From<#action_builder_ident> for ::vixen::HxAction {
            fn from(action: #action_builder_ident) -> Self {
                action.0
            }
        }

        impl ::vixen::maud::Render for #action_builder_ident {
            fn render_to(&self, buffer: &mut ::std::string::String) {
                ::vixen::maud::Render::render_to(&self.0, buffer)
            }
        }

        #vis struct #action_fields_ident {
            #(pub #action_field_idents: &'static str,)*
        }

        impl #action_ident {
            pub const FIELD: #action_fields_ident = #action_fields_ident {
                #(#action_field_idents: #action_field_keys,)*
            };

            /// Starts an `hx-action=(...)` value for this route.
            pub fn action() -> #action_builder_ident {
                #action_builder_ident(::vixen::HxAction::new(
                    <#action_path_ident as ::axum_extra::routing::TypedPath>::PATH,
                ))
            }

            pub fn path(&self) -> &'static str {
                <#action_path_ident as ::axum_extra::routing::TypedPath>::PATH
            }
        }
    }
}
