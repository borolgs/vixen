use heck::ToKebabCase;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Error, Item, LitStr, parse_quote};

pub fn expand(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut id_struct = match syn::parse2::<Item>(item) {
        Ok(Item::Struct(s)) => s,
        Ok(other) => {
            let err = Error::new_spanned(
                &other,
                "`#[id]` expects a unit struct, e.g. `#[id] struct Notes;`",
            )
            .to_compile_error();

            return quote! { #err #other };
        }
        Err(err) => return err.to_compile_error(),
    };

    let id_struct_ident = id_struct.ident.clone();

    let html_id = {
        let kebab_id = id_struct_ident.to_string().to_kebab_case();

        if let Some(stripped) = kebab_id.strip_suffix("-id") {
            stripped.to_string()
        } else {
            kebab_id
        }
    };

    let id = syn::parse2::<LitStr>(attr).unwrap_or(LitStr::new(&html_id, Span::call_site()));

    let value = id.value();
    if value.is_empty() || value.contains(char::is_whitespace) || value.contains('#') {
        let err = Error::new(
            id.span(),
            "expected a bare id: not empty, no whitespace, no `#`",
        )
        .to_compile_error();
        return quote! { #err #id_struct };
    }

    let id_selector = LitStr::new(&format!("#{value}"), Span::call_site());

    let doc = format!("HTML id: `{value}`");
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
            pub const ID: &'static str = #id;
            pub const SEL: &'static str = #id_selector;

            pub fn slot() -> ::vixen::maud::Markup {
                ::vixen::maud::html! {
                    div id=#id {}
                }
            }
        }

        impl ::vixen::Id for #id_struct_ident {
            const ID: &'static str = Self::ID;
            const SEL: &'static str = Self::SEL;
        }
    }
}
