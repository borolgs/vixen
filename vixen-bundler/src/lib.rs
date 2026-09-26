//! Per-page asset bundling for build scripts.
//!
//! Call [`build`] from `build.rs`. It runs the bundled Bun script and exports
//! `VIXEN_MANIFEST` and `VIXEN_BASE_PATH` to rustc. Defaults may be overridden
//! with `VIXEN_<FIELD>` environment variables.
//!
//! Apps reach this crate as `vixen::{build, Config}`, with `vixen` listed
//! under `[build-dependencies]`:
//!
//! ```no_run
//! # mod vixen { pub use vixen_bundler::{build, Config}; }
//! // build.rs
//! vixen::build(vixen::Config::default());
//! ```

#![warn(missing_docs)]

use std::{
    env,
    io::{self, Write},
    path::{Path, PathBuf},
    process::Command,
};

/// Bundler script copied to `OUT_DIR` before execution.
const BUILD_TS: &str = include_str!("./build.ts");

/// Environment variables that affect the build configuration.
const CONFIG_ENV_VARS: [&str; 7] = [
    "VIXEN_BASE_PATH",
    "VIXEN_BUN_CMD",
    "VIXEN_ROOT",
    "VIXEN_ASSETS_PREFIX",
    "VIXEN_ENTRY_GLOB",
    "VIXEN_STATIC_GLOB",
    "VIXEN_CONFIG",
];

// TODO: Move `Config` and `build` into vixen, or separate `base_path` from
// the bundler settings.
/// Build settings. Paths are relative to `CARGO_MANIFEST_DIR`.
///
/// [`Default::default`] uses nonempty `VIXEN_<FIELD>` environment variables.
#[derive(Debug, Clone)]
pub struct Config {
    /// URL prefix the app is served under, e.g. `/app`. Empty by default.
    pub base_path: String,
    /// Bun executable (`bun` by default).
    pub bun_cmd: String,
    /// Base path stripped from entry paths (`src` by default).
    pub root: PathBuf,
    /// Output directory (`assets` by default).
    pub assets_prefix: PathBuf,
    /// Entry glob (`src/pages/**/{page,index}.ts` by default).
    pub entry_glob: String,
    /// Static asset glob for `asset!`
    /// (`src/**/assets/**/*.{svg,png,jpg,jpeg,gif,webp,avif,ico}` by default).
    pub static_glob: String,
    /// Bun config merged into `Bun.build` (`build.ts` by default).
    /// Leave `publicPath` unset; `assets!` adds the URL prefix.
    pub config: Option<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            base_path: env_or("VIXEN_BASE_PATH", ""),
            bun_cmd: env_or("VIXEN_BUN_CMD", "bun"),
            root: env_or("VIXEN_ROOT", "src").into(),
            assets_prefix: env_or("VIXEN_ASSETS_PREFIX", "assets").into(),
            entry_glob: env_or("VIXEN_ENTRY_GLOB", "src/pages/**/{page,index}.ts"),
            static_glob: env_or(
                "VIXEN_STATIC_GLOB",
                "src/**/assets/**/*.{svg,png,jpg,jpeg,gif,webp,avif,ico}",
            ),
            config: Some(env_or("VIXEN_CONFIG", "build.ts").into()),
        }
    }
}

/// Returns the literal path prefix of a glob.
fn glob_base(glob: &str) -> PathBuf {
    glob.split('/')
        .take_while(|s| !s.contains(['*', '?', '[', '{', '!']))
        .collect()
}

fn env_or(name: &str, default: &str) -> String {
    env::var(name)
        .ok()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| default.into())
}

/// Bundle the configured entries and set `VIXEN_MANIFEST` for rustc.
///
/// # Panics
///
/// Panics if Bun cannot start or exits unsuccessfully.
pub fn build(cfg: Config) {
    let manifest_dir =
        env::var("CARGO_MANIFEST_DIR").expect("vixen-bundler: CARGO_MANIFEST_DIR unset");

    let script = {
        let out_dir = env::var("OUT_DIR").expect("vixen-bundler: OUT_DIR unset");
        let script_path = PathBuf::from(out_dir).join("vixen-bundler.ts");

        std::fs::write(&script_path, BUILD_TS)
            .unwrap_or_else(|e| panic!("vixen-bundler: cannot write vixen-bundler.ts: {e}"));
        script_path
    };

    let mut bun = Command::new(&cfg.bun_cmd);
    bun.arg("run")
        .arg(&script)
        .arg("--root")
        .arg(&cfg.root)
        .arg("--assetsPrefix")
        .arg(&cfg.assets_prefix)
        .arg("--entryGlob")
        .arg(&cfg.entry_glob)
        .arg("--staticGlob")
        .arg(&cfg.static_glob)
        .current_dir(&manifest_dir);

    // `rerun-if`

    for name in CONFIG_ENV_VARS {
        println!("cargo::rerun-if-env-changed={name}");
    }
    let dir = Path::new(&manifest_dir);
    // Watch this separately because `static_glob` may point outside `root`.
    let static_base = glob_base(&cfg.static_glob);
    for path in [
        cfg.root.as_path(),
        static_base.as_path(),
        Path::new("package.json"),
        Path::new("tsconfig.json"),
    ] {
        let path = dir.join(path);
        if path != dir && path.exists() {
            println!("cargo::rerun-if-changed={}", path.display());
        }
    }

    if let Some(lockfile) = dir
        .ancestors()
        .map(|d| d.join("bun.lock"))
        .find(|p| p.exists())
    {
        println!("cargo::rerun-if-changed={}", lockfile.display());
    }

    if let Some(user_config) = &cfg.config
        && let user_config_path = dir.join(user_config)
        && user_config_path.is_file()
    {
        println!("cargo::rerun-if-changed={}", user_config_path.display());
        bun.arg("--userConfig").arg(user_config_path);
    }

    let out = bun.output().unwrap_or_else(|e| {
        panic!("vixen-bundler: cannot run `{}`: {e}", cfg.bun_cmd);
    });

    // Forward Cargo directives from `build.ts`.
    io::stdout()
        .write_all(&out.stdout)
        .expect("vixen-bundler: cannot forward bun's stdout to cargo");
    io::stderr()
        .write_all(&out.stderr)
        .expect("vixen-bundler: cannot forward bun's stderr");

    if !out.status.success() {
        panic!(
            "vixen-bundler: bun failed ({})\n{}",
            out.status,
            String::from_utf8_lossy(&out.stderr)
        );
    }

    println!("cargo::rustc-env=VIXEN_BASE_PATH={}", cfg.base_path);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glob_base_stops_at_the_first_pattern() {
        let cases = [
            ("src/**/assets/**/*.png", "src"),
            ("src/pages/*/assets/*.svg", "src/pages"),
            ("static/logo.svg", "static/logo.svg"),
            ("**/assets/*.png", ""),
            ("{src,static}/**/*.png", ""),
        ];
        for (glob, want) in cases {
            assert_eq!(glob_base(glob), Path::new(want), "{glob}");
        }
    }
}
