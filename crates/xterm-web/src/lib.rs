//! Browser host components and WASM bindings for the Rust terminal engine.
//!
//! This crate is an incomplete port. Its DOM host is not a complete terminal;
//! unsupported terminal APIs are not exported as successful placeholders.

pub mod api;
pub mod input;
pub mod links;
pub mod selection;
pub mod theme;
pub mod contrast_cache;
pub mod renderer;
pub mod char_size;
pub mod character_joiner;
#[cfg(target_arch = "wasm32")]
pub mod char_size_browser;

#[cfg(target_arch = "wasm32")]
pub mod dom;
#[cfg(target_arch = "wasm32")]
pub mod canvas_css;
#[cfg(target_arch = "wasm32")]
pub mod dom_listener;
#[cfg(target_arch = "wasm32")]
pub mod utf16;
#[cfg(target_arch = "wasm32")]
pub mod js_error;
#[cfg(target_arch = "wasm32")]
pub mod logger;
#[cfg(target_arch = "wasm32")]
pub mod task_queue;
#[cfg(target_arch = "wasm32")]
pub mod font_set;
#[cfg(target_arch = "wasm32")]
pub mod font_addons;
#[cfg(target_arch = "wasm32")]
pub mod utility_addons;
#[cfg(target_arch = "wasm32")]
pub use font_addons::{load_web_fonts, WebFontsAddon};
#[cfg(target_arch = "wasm32")]
mod host;

#[cfg(target_arch = "wasm32")]
pub use host::BrowserHost;
