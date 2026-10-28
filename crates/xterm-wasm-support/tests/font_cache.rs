#![cfg(target_arch = "wasm32")]
//! Component tests compile the actual renderer modules, not copies or stubs.
//! Full Terminal rendering remains a separate integration suite.
use wasm_bindgen_test::*;
pub use xterm_wasm_support::utf16;
wasm_bindgen_test_configure!(run_in_browser);
#[path = "../../xterm-web/src/renderer/canvas_width.rs"]
mod canvas_width;
#[path = "../../xterm-web/src/renderer/raw_font.rs"]
mod raw_font;
#[path = "../../xterm-web/src/renderer/width_cache.rs"]
mod width_cache;
