//! Browser font addons backed by Rust shaping and platform APIs.
//! The font parser and cache stay portable in xterm-addons. These bindings have
//! no dependency on the terminal renderer, so component tests run independently.
#[cfg(target_arch = "wasm32")]
pub use xterm_wasm_support::utf16;
#[cfg(target_arch = "wasm32")]
pub mod font_addons;
#[cfg(target_arch = "wasm32")]
pub mod font_set;
#[cfg(target_arch = "wasm32")]
mod ligature_host;
#[cfg(target_arch = "wasm32")]
mod ligature_state;
#[cfg(target_arch = "wasm32")]
pub mod ligatures_addon;
#[cfg(target_arch = "wasm32")]
pub mod local_fonts;
#[cfg(target_arch = "wasm32")]
pub use font_addons::{load_web_fonts, WebFontsAddon};
#[cfg(target_arch = "wasm32")]
pub use ligatures_addon::LigaturesAddon;
