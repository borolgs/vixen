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

    let empty = quote! { ::vixen::maud::PreEscaped("") };

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

    let mut head = String::new();
    head.push_str(r#"<script type="module" defer src=""#);
    escape_attr(&page_entry.js, &mut head);
    head.push_str(r#""></script>"#);
    if let Some(css) = &page_entry.css {
        head.push_str(r#"<link rel="stylesheet" type="text/css" href=""#);
        escape_attr(css, &mut head);
        head.push_str(r#"">"#);
    }

    let head = LitStr::new(&head, call_span);

    quote! { ::vixen::maud::PreEscaped(#head) }
}

fn escape_attr(value: &str, out: &mut String) {
    for c in value.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            c => out.push(c),
        }
    }
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
    let manifest_raw = env::var("VIXEN_MANIFEST").map_err(|_| {
        "VIXEN_MANIFEST is not set: call `vixen::bundler::build` from build.rs".to_string()
    })?;

    serde_json::from_str(&manifest_raw)
        .map_err(|e| format!("VIXEN_MANIFEST is not a valid asset manifest: {e}"))
}
