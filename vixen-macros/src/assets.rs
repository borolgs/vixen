use std::{
    collections::HashMap,
    env,
    path::{Path, PathBuf, absolute},
};

use proc_macro2::TokenStream;
use quote::quote;
use serde::Deserialize;
use syn::{Error, LitStr};

pub fn expand_assets_router(_: TokenStream) -> TokenStream {
    let call_span = proc_macro2::Span::call_site();

    let mut manifest = match parse_manifest() {
        Ok(m) => m,
        Err(err) => {
            return Error::new(call_span, err).to_compile_error();
        }
    };

    manifest.files.sort();

    let mut files = Vec::new();
    for file in manifest.files {
        let abs_path = PathBuf::from(&manifest.dir).join(&file);
        let content_type = mime_guess::from_path(&file)
            .first_or_octet_stream()
            .to_string();

        let abs_path = match abs_path.to_str() {
            Some(path) => path,
            None => {
                return Error::new(
                    call_span,
                    format!(
                        "asset path is not valid UTF-8: {}",
                        abs_path.to_string_lossy()
                    ),
                )
                .to_compile_error();
            }
        };

        let file = LitStr::new(&file, call_span);
        let content_type = LitStr::new(&content_type, call_span);
        let abs_path = LitStr::new(abs_path, call_span);
        files.push(quote! {
            // "chunk-srqsj3n7.js", "text/javascript", b"...\n",
            (#file, #content_type, ::core::include_bytes!(#abs_path))
        });
    }

    let prefix = LitStr::new(&manifest.prefix, call_span);

    quote! {
        {
            const FILES: &[(&::core::primitive::str, &::core::primitive::str, &[::core::primitive::u8])] = &[
                #(#files,)*
            ];

            ::vixen::assets::router(FILES, #prefix)
        }
    }
}

pub fn expand_assets_head(_: TokenStream) -> TokenStream {
    let call_span = proc_macro2::Span::call_site();

    let empty = quote! { ::vixen::maud::PreEscaped(::std::string::String::new()) };

    let Some(page_dir) = call_site_dir(call_span) else {
        return empty;
    };

    let manifest = match parse_manifest() {
        Ok(m) => m,
        Err(err) => {
            return Error::new(call_span, err).to_compile_error();
        }
    };

    let Some(page_entry) = manifest.entries.iter().find_map(|(entry, e)| {
        (Path::new(entry).parent() == Some(page_dir.as_path())).then_some(e)
    }) else {
        return empty;
    };

    let js = LitStr::new(&page_entry.js, call_span);
    let css = match &page_entry.css {
        Some(css) => {
            let css = LitStr::new(css, call_span);
            quote! { ::core::option::Option::Some(#css) }
        }
        None => quote! { ::core::option::Option::None },
    };

    quote! { ::vixen::assets::head(::vixen::base_path!(), #js, #css) }
}

fn call_site_dir(span: proc_macro2::Span) -> Option<PathBuf> {
    if !proc_macro::is_available() {
        return None;
    }
    let file = span.unwrap().local_file()?;
    absolute(file.parent()?).ok()
}

#[derive(Deserialize)]
pub struct Manifest {
    #[allow(unused)]
    pub entry_glob: String,
    pub dir: String,
    pub prefix: String,
    pub files: Vec<String>,
    pub entries: HashMap<String, Entry>,
}

#[derive(Deserialize, Debug)]
pub struct Entry {
    pub js: String,
    pub css: Option<String>,
}

pub fn parse_manifest() -> Result<Manifest, String> {
    let manifest_raw = env::var("VIXEN_MANIFEST")
        .map_err(|_| "VIXEN_MANIFEST is not set: call `vixen::build` from build.rs".to_string())?;

    serde_json::from_str(&manifest_raw)
        .map_err(|e| format!("VIXEN_MANIFEST is not a valid asset manifest: {e}"))
}
