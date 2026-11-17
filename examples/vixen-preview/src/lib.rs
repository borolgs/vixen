mod app;
mod preview;
mod todos;
mod ui;

#[cfg(target_arch = "wasm32")]
mod browser;

pub use app::router;
