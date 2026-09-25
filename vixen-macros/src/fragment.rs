use heck::ToPascalCase;
use proc_macro2::{Group, Ident, Span, TokenStream, TokenTree};
use quote::{ToTokens, format_ident, quote};
use syn::{ItemFn, LitStr, parse_quote, parse2, spanned::Spanned, visit_mut::VisitMut};

pub fn expand(attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut fragment_fn = match parse2::<ItemFn>(item) {
        Ok(v) => v,
        Err(err) => {
            return err.to_compile_error();
        }
    };

    let fragment_fn_ident = &fragment_fn.sig.ident;

    let vis = &fragment_fn.vis;
    let fragment_fn_name = &fragment_fn.sig.ident;
    let asyncness = &fragment_fn.sig.asyncness;

    // <FnName>Id;
    let id_struct_ident = format_ident!("{}Id", fragment_fn_ident.to_string().to_pascal_case());

    // The closure keeps the user's `-> Markup`, so their import stays used.
    let markup = std::mem::replace(
        &mut fragment_fn.sig.output,
        parse_quote! { -> ::vixen::Fragment<#id_struct_ident> },
    );

    // id=(Self) -> id(<FnName>Id)
    let mut replacer = Replacer::new("Self", id_struct_ident.clone());
    syn::visit_mut::visit_block_mut(&mut replacer, fragment_fn.block.as_mut());

    // Wrap the body's Markup in a Fragment.
    let body = fragment_fn.block.to_token_stream();
    let await_ = asyncness.map(|_| quote! { .await });
    fragment_fn.block = parse_quote! {{
        ::vixen::Fragment::new((#asyncness || #markup #body)() #await_)
    }};

    // TODO: match the `id` `=` `(Self)` token sequence; any `Self` passes for now.
    if replacer.count < 1 {
        return syn::Error::new(
            attr.span(),
            "`#[fragment]` expects `id=(Self)` on the root element, e.g. `ul id=(Self) { .. }`",
        )
        .to_compile_error();
    }

    // `#[fragment("custom")]`
    let id = if attr.is_empty() {
        LitStr::new(&crate::id::html_id(&id_struct_ident), Span::call_site())
    } else {
        match parse2::<LitStr>(attr).and_then(|id| crate::id::check(&id).map(|_| id)) {
            Ok(id) => id,
            Err(err) => return err.to_compile_error(),
        }
    };

    let doc = format!(
        "Fragment with id [`{id_struct_ident}`]: a page call renders the element, a `partial!` call \
         replaces it (`outerHTML`). [`{fragment_fn_name}::slot()`] renders a hidden `<div>` with the \
         same id for a later `partial!` to replace."
    );
    if fragment_fn
        .attrs
        .iter()
        .any(|attr| attr.path().is_ident("doc"))
    {
        fragment_fn.attrs.push(parse_quote!(#[doc = ""]));
    }
    fragment_fn.attrs.push(parse_quote!(#[doc = #doc]));

    quote! {
        #[::vixen::id(#id)]
        #vis struct #id_struct_ident;

        #vis mod #fragment_fn_name {
            /// A hidden `<div>` with the fragment's id, for a later `partial!` to replace.
            pub fn slot() -> ::vixen::maud::Markup {
                ::vixen::maud::html! {
                    div id=#id style="display: none;" {}
                }
            }
        }

        #fragment_fn
    }
}

struct Replacer {
    count: u32,
    from: String,
    to: Ident,
}

impl VisitMut for Replacer {
    fn visit_ident_mut(&mut self, ident: &mut Ident) {
        if ident == "Self" {
            self.count += 1;
            let mut to = self.to.clone();
            to.set_span(ident.span());
            *ident = to;
        }
    }

    fn visit_macro_mut(&mut self, mac: &mut syn::Macro) {
        mac.tokens = self.replace_ident(&mac.tokens);
    }
}

impl Replacer {
    fn new(from: impl Into<String>, to: Ident) -> Self {
        Self {
            count: 0,
            from: from.into(),
            to,
        }
    }

    fn replace_ident(&mut self, tokens: &TokenStream) -> TokenStream {
        tokens
            .to_token_stream()
            .into_iter()
            .map(|tt| match tt {
                TokenTree::Ident(i) if i == self.from => {
                    self.count += 1;
                    let mut to = self.to.clone();
                    to.set_span(i.span());
                    to.into()
                }
                TokenTree::Group(g) => {
                    let mut out = Group::new(g.delimiter(), self.replace_ident(&g.stream()));
                    out.set_span(g.span());
                    out.into()
                }
                tt => tt,
            })
            .collect()
    }
}
