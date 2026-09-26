use std::{
    collections::HashMap,
    env,
    path::{Component, Path, PathBuf, absolute},
};

use proc_macro2::TokenStream;
use quote::quote;
use serde::Deserialize;
use syn::{Error, LitStr, parse2};

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

pub fn expand_asset(item: TokenStream) -> TokenStream {
    let lit: LitStr = match parse2(item) {
        Ok(lit) => lit,
        Err(err) => return err.to_compile_error(),
    };
    let span = lit.span();

    // rust-analyzer has no calling file, so defer path resolution to rustc.
    let Some(dir) = call_site_dir(span) else {
        return quote! { ::vixen::Href::new(::vixen::base_path!(), "") };
    };

    let url = parse_manifest().and_then(|manifest| resolve_asset(&dir, &lit.value(), &manifest));
    match url {
        Ok(url) => {
            let url = LitStr::new(&url, span);
            quote! { ::vixen::Href::new(::vixen::base_path!(), #url) }
        }
        Err(err) => Error::new(span, err).to_compile_error(),
    }
}

fn resolve_asset(dir: &Path, rel: &str, manifest: &Manifest) -> Result<String, String> {
    let path = normalize(&dir.join(rel));
    if !path.is_file() {
        return Err(format!("cannot find `{rel}` at {}", path.display()));
    }
    path.to_str()
        .and_then(|p| manifest.statics.get(p))
        .cloned()
        .ok_or_else(|| {
            format!(
                "`{rel}` is not matched by `Config::static_glob` (`{}`)",
                manifest.static_glob
            )
        })
}

/// Collapses `.` and `..` without accessing the file system.
fn normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for c in path.components() {
        match c {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            c => out.push(c),
        }
    }
    out
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
    pub static_glob: String,
    pub dir: String,
    pub prefix: String,
    pub files: Vec<String>,
    pub entries: HashMap<String, Entry>,
    #[serde(rename = "static")]
    pub statics: HashMap<String, String>,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest(statics: &[(&Path, &str)]) -> Manifest {
        let statics: HashMap<_, _> = statics
            .iter()
            .map(|(path, url)| (path.to_str().unwrap(), url))
            .collect();
        let raw = serde_json::json!({
            "entry_glob": "src/pages/**/{page,index}.ts",
            "static_glob": "src/**/assets/**/*.png",
            "dir": "/out/assets",
            "prefix": "assets",
            "files": [],
            "entries": {},
            "static": statics,
        });
        serde_json::from_value(raw).unwrap()
    }

    #[test]
    fn normalizes_dot_segments() {
        assert_eq!(
            normalize(Path::new("/app/src/pages/a/./../b/assets/x.png")),
            Path::new("/app/src/pages/b/assets/x.png")
        );
    }

    #[test]
    fn resolves_relative_to_the_calling_dir() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let manifest = manifest(&[(&root.join("Cargo.toml"), "/assets/Cargo-1a2b3c4d.toml")]);

        let url = resolve_asset(&root.join("src"), "../Cargo.toml", &manifest);
        assert_eq!(url.as_deref(), Ok("/assets/Cargo-1a2b3c4d.toml"));
    }

    #[test]
    fn rejects_missing_and_unmatched_files() {
        let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let manifest = manifest(&[]);

        let err = resolve_asset(&src, "./nope.png", &manifest).unwrap_err();
        assert!(err.starts_with("cannot find `./nope.png` at "), "{err}");

        let err = resolve_asset(&src, "./lib.rs", &manifest).unwrap_err();
        assert_eq!(
            err,
            "`./lib.rs` is not matched by `Config::static_glob` (`src/**/assets/**/*.png`)"
        );
    }

    #[test]
    fn falls_back_to_an_empty_url_without_a_calling_file() {
        let out = expand_asset(quote!("./assets/x.png"));
        let expected = quote! { ::vixen::Href::new(::vixen::base_path!(), "") };
        assert_eq!(out.to_string(), expected.to_string());
    }
}
