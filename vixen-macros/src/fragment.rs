use heck::ToPascalCase;
use proc_macro2::{Group, Ident, Span, TokenStream, TokenTree};
use quote::{ToTokens, format_ident, quote};
use syn::{
    Expr, ItemFn, LitStr, parenthesized, parse::Parse, parse_quote, parse2, spanned::Spanned,
    token, visit_mut::VisitMut,
};

use crate::id::{check_id, html_id};

enum FragmentAttr {
    Default,
    Name(LitStr),
    Id { ty: Ident, arg: Option<Expr> },
}

impl Parse for FragmentAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(Self::Default);
        }

        if input.peek(LitStr) {
            let name: LitStr = input.parse()?;
            check_id(&name)?;
            if !input.is_empty() {
                return Err(input.error(
                    "a dynamic id needs its own struct: \
                     `#[id] struct TodoId(u64);` and `#[fragment(TodoId(todo.id))]`",
                ));
            }
            return Ok(Self::Name(name));
        }

        let ty: Ident = input.parse()?;

        let arg: Option<Expr> = if input.peek(token::Paren) {
            let content;
            parenthesized!(content in input);
            Some(content.parse()?)
        } else {
            None
        };

        Ok(Self::Id { ty, arg })
    }
}

pub fn expand(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr_span = attr.span();
    let attr: FragmentAttr = match parse2(attr) {
        Ok(v) => v,
        Err(err) => return err.to_compile_error(),
    };

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
    let (id_struct_ident, html_id, id_arg) = match attr {
        FragmentAttr::Default => {
            let html_id = LitStr::new(&html_id(&id_struct_ident), Span::call_site());
            (id_struct_ident, Some(html_id), None)
        }
        FragmentAttr::Name(lit_str) => (id_struct_ident, Some(lit_str), None),
        FragmentAttr::Id { ty, arg } => (ty, None, arg),
    };

    // __vixen_id | <FnName>Id
    // TODO: make the dynamic id binding hygienic so user code cannot shadow it.
    let id_val = id_arg
        .as_ref()
        .map(|_| Ident::new("__vixen_id", Span::call_site()))
        .unwrap_or(id_struct_ident.clone());

    // The closure keeps the user's `-> Markup`, so their import stays used.
    let markup = std::mem::replace(
        &mut fragment_fn.sig.output,
        parse_quote! { -> ::vixen::Fragment<#id_struct_ident> },
    );

    // id=(Self) -> id(<FnName>Id) | id=(__vixen_id)
    let mut replacer = Replacer::new("Self", id_val.clone());
    syn::visit_mut::visit_block_mut(&mut replacer, fragment_fn.block.as_mut());

    // TODO: match the `id` `=` `(Self)` token sequence; any `Self` passes for now.
    if replacer.count < 1 {
        return syn::Error::new(
            attr_span,
            "`#[fragment]` expects `id=(Self)` on the root element, e.g. `ul id=(Self) { .. }`",
        )
        .to_compile_error();
    }

    let id_stmt = id_arg.map(|arg| {
        quote! { let #id_val = #id_struct_ident(#arg); }
    });

    // Wrap the body's Markup in a Fragment.
    let body = fragment_fn.block.to_token_stream();
    let await_ = asyncness.map(|_| quote! { .await });
    fragment_fn.block = parse_quote! {{
        #id_stmt
        ::vixen::Fragment::new(&#id_val, (#asyncness || #markup #body)() #await_)
    }};

    let mut doc = format!(
        "Fragment with id [`{id_struct_ident}`]: a page call renders the element, a `partial!` call \
         replaces it (`outerHTML`)."
    );
    if html_id.is_some() {
        doc.push_str(&format!(
            " [`{fragment_fn_name}::slot()`] renders a hidden `<div>` with the same id for a later \
             `partial!` to replace."
        ));
    }
    if fragment_fn
        .attrs
        .iter()
        .any(|attr| attr.path().is_ident("doc"))
    {
        fragment_fn.attrs.push(parse_quote!(#[doc = ""]));
    }
    fragment_fn.attrs.push(parse_quote!(#[doc = #doc]));

    let id_decl = html_id.map(|html_id| {
        quote! {
            #[::vixen::id(#html_id)]
            #vis struct #id_struct_ident;

            // Static ids only; a dynamic id has `SomeId(id).slot()` instead.
            #vis mod #fragment_fn_name {
                /// A hidden `<div>` with the fragment's id, for a later `partial!` to replace.
                pub fn slot() -> ::vixen::maud::Markup {
                    ::vixen::maud::html! {
                        div id=#html_id style="display: none;" {}
                    }
                }
            }
        }
    });

    quote! {
        #id_decl
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
