use heck::ToKebabCase;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{ToTokens, quote};
use syn::{Error, Item, LitStr, parse_quote, parse2};

pub fn expand(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut id_struct = match parse2(item) {
        Ok(Item::Struct(s)) => s,
        Ok(other) => {
            let err = Error::new_spanned(
                &other,
                "`#[id]` expects a unit or one-field tuple struct, e.g. `#[id] struct TodoId(u64);`",
            )
            .to_compile_error();

            return quote! { #err #other };
        }
        Err(err) => return err.to_compile_error(),
    };

    // #[id] SomeId(suffix)
    let suffix_field = match &id_struct.fields {
        syn::Fields::Unit => None,
        // TODO: forbit generics
        syn::Fields::Unnamed(fields) if fields.unnamed.len() == 1 => fields.unnamed.first(),
        fields => {
            let err = Error::new_spanned(
                fields,
                "`#[id]` expects a unit or one-field tuple struct, e.g. `#[id] struct TodoId(u64);`",
            )
            .to_compile_error();

            return quote! { #err #id_struct };
        }
    };

    let id_struct_ident = id_struct.ident.clone();

    let id = {
        let html_id = html_id(&id_struct_ident);
        parse2(attr).unwrap_or(LitStr::new(&html_id, Span::call_site()))
    };

    if let Err(err) = check_id(&id) {
        let err = err.to_compile_error();
        return quote! { #err #id_struct };
    }

    if let Some(field) = suffix_field {
        let id_value = id.value();

        let doc = format!("HTML id: `{id_value}-{{{}}}`", field.ty.to_token_stream());
        if id_struct
            .attrs
            .iter()
            .any(|attr| attr.path().is_ident("doc"))
        {
            id_struct.attrs.push(parse_quote!(#[doc = ""]));
        }
        id_struct.attrs.push(parse_quote!(#[doc = #doc]));

        return quote! {
            #id_struct

            impl ::vixen::maud::Render for #id_struct_ident {
                fn render_to(&self, buffer: &mut ::std::string::String) {
                    use ::std::fmt::Write as _;
                    let _ = ::std::write!(
                        ::vixen::maud::Escaper::new(buffer),
                        "{}-{}",
                        #id,
                        self.0,
                    );
                }
            }

            impl ::vixen::Id for #id_struct_ident {
                fn sel(&self) -> ::vixen::Selector {
                    // TODO: add CSS escape
                    ::vixen::Selector::from(
                        ::std::format!("#{}-{}", #id, self.0)
                    )
                }
            }

            impl #id_struct_ident {
                pub fn slot(&self) -> ::vixen::maud::Markup {
                    ::vixen::maud::html! {
                        div id=(self) {}
                    }
                }
            }
        };
    }

    let id_value = id.value();
    // TODO: add CSS escape
    let id_selector = LitStr::new(&format!("#{id_value}"), Span::call_site());

    let doc = format!("HTML id: `{id_value}`");
    if id_struct
        .attrs
        .iter()
        .any(|attr| attr.path().is_ident("doc"))
    {
        id_struct.attrs.push(parse_quote!(#[doc = ""]));
    }
    id_struct.attrs.push(parse_quote!(#[doc = #doc]));

    quote! {
        #id_struct

        impl ::vixen::maud::Render for #id_struct_ident {
            fn render_to(&self, buffer: &mut ::std::string::String) {
                buffer.push_str(#id);
            }
        }

        impl #id_struct_ident {
            pub fn slot() -> ::vixen::maud::Markup {
                ::vixen::maud::html! {
                    div id=#id {}
                }
            }
        }

        impl ::vixen::Id for #id_struct_ident {
            fn sel(&self) -> ::vixen::Selector {
                ::vixen::Selector::from(#id_selector)
            }
        }
    }
}

/// The default id for `ident`: kebab-case, with a trailing `-id` removed.
pub fn html_id(ident: &Ident) -> String {
    let kebab = ident.to_string().to_kebab_case();
    match kebab.strip_suffix("-id") {
        Some(stripped) => stripped.to_owned(),
        None => kebab,
    }
}

/// is valid id
pub fn check_id(id: &LitStr) -> syn::Result<()> {
    let value = id.value();
    if value.is_empty() || value.contains(char::is_whitespace) || value.contains('#') {
        return Err(Error::new(
            id.span(),
            "expected a bare id: not empty, no whitespace, no `#`",
        ));
    }
    Ok(())
}
