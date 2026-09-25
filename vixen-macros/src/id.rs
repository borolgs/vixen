use heck::ToKebabCase;
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{Error, Item, LitStr, parse_quote};

pub fn check(id: &LitStr) -> syn::Result<()> {
    let value = id.value();
    if value.is_empty() || value.contains(char::is_whitespace) || value.contains('#') {
        return Err(Error::new(
            id.span(),
            "expected a bare id: not empty, no whitespace, no `#`",
        ));
    }
    Ok(())
}

/// The default id for `ident`: kebab-case, with a trailing `-id` removed.
pub fn html_id(ident: &Ident) -> String {
    let kebab = ident.to_string().to_kebab_case();
    match kebab.strip_suffix("-id") {
        Some(stripped) => stripped.to_owned(),
        None => kebab,
    }
}

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

    let html_id = html_id(&id_struct_ident);

    let id = syn::parse2::<LitStr>(attr).unwrap_or(LitStr::new(&html_id, Span::call_site()));

    if let Err(err) = check(&id) {
        let err = err.to_compile_error();
        return quote! { #err #id_struct };
    }
    let value = id.value();

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
