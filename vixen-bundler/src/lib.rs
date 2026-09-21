//! Per-page asset bundling for build scripts.
//!
//! Call [`build`] from `build.rs`. It runs the bundled Bun script and exports
//! the asset manifest to rustc as `VIXEN_MANIFEST`.
//!
//! Apps reach this crate as `vixen::bundler`, with `vixen` listed under
//! `[build-dependencies]`:
//!
//! ```no_run
//! # mod vixen { pub use vixen_bundler as bundler; }
//! // build.rs
//! vixen::bundler::build(&vixen::bundler::Config::default());
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

/// Bundler settings. Paths are relative to `CARGO_MANIFEST_DIR`.
#[derive(Debug, Clone)]
pub struct Config {
    /// Bun executable. Default: `bun`.
    pub bun_cmd: String,
    /// Base path stripped from entry paths. Default: `src`.
    pub root: PathBuf,
    /// Output directory. Default: `assets`.
    pub assets_prefix: PathBuf,
    /// Entry glob. Default: `src/pages/**/{page,index}.ts`.
    pub entry_glob: String,
    /// Consumer bun config merged into `Bun.build`.
    pub config: Option<PathBuf>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            bun_cmd: "bun".into(),
            root: "src".into(),
            assets_prefix: "assets".into(),
            entry_glob: "src/pages/**/{page,index}.ts".into(),
            config: Some("build.ts".into()),
        }
    }
}

/// Bundle the configured entries and set `VIXEN_MANIFEST` for rustc.
///
/// # Panics
///
/// Panics if Bun cannot start or exits unsuccessfully.
pub fn build(cfg: &Config) {
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
        .current_dir(&manifest_dir);

    if let Some(user_config) = &cfg.config
        && let user_config_path = Path::new(&manifest_dir).join(user_config)
        && user_config_path.is_file()
    {
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
}
