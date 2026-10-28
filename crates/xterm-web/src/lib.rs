//! Browser host components and WASM bindings for the Rust terminal engine.
//!
//! This crate is an incomplete port. Its DOM host is not a complete terminal;
//! unsupported terminal APIs are not exported as successful placeholders.

/// Upstream stylesheet required by the browser renderer. Include it once per document.
pub const XTERM_CSS: &str = include_str!("../assets/xterm.css");

pub mod api;
pub mod char_size;
#[cfg(target_arch = "wasm32")]
pub mod char_size_browser;
pub mod character_joiner;
pub mod contrast_cache;
pub mod input;
pub mod links;
pub mod renderer;
pub mod selection;
pub mod theme;
pub mod webgl;
pub mod search_host;
pub mod scroll;
pub mod viewport;
pub mod accessibility;

#[cfg(target_arch = "wasm32")]
pub mod canvas_css;
#[cfg(target_arch = "wasm32")]
pub mod dom;
#[cfg(target_arch = "wasm32")]
pub mod core_browser;
#[cfg(target_arch = "wasm32")]
pub mod unicode_addons;
#[cfg(target_arch = "wasm32")]
pub mod dom_listener;
#[cfg(target_arch = "wasm32")]
pub mod font_addons;
#[cfg(target_arch = "wasm32")]
pub mod buffer_cleanup;
#[cfg(target_arch = "wasm32")]
pub mod font_set;
#[cfg(target_arch = "wasm32")]
pub mod local_fonts;
#[cfg(target_arch = "wasm32")]
pub mod task_queue;
#[cfg(target_arch = "wasm32")]
pub mod js_error;
#[cfg(target_arch = "wasm32")]
pub mod logger;
#[cfg(target_arch = "wasm32")]
pub mod write_scheduler;
#[cfg(target_arch = "wasm32")]
pub mod byte_view;
#[cfg(target_arch = "wasm32")]
pub mod terminal;
#[cfg(target_arch = "wasm32")]
pub mod status_addons;
#[cfg(target_arch = "wasm32")]
pub mod search_addon;
#[cfg(target_arch = "wasm32")]
pub use terminal::{Terminal, runtime::BrowserRuntime};
#[cfg(target_arch = "wasm32")]
pub mod utf16;
#[cfg(target_arch = "wasm32")]
pub mod utility_addons;
#[cfg(target_arch = "wasm32")]
pub use font_addons::{load_web_fonts, WebFontsAddon};
#[cfg(target_arch = "wasm32")]
mod host;

#[cfg(target_arch = "wasm32")]
pub use host::BrowserHost;
