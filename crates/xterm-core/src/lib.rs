//! Platform-independent terminal state, parsing, and shared utilities.
//!
//! Browser and operating-system integration belong in separate adapter crates.
//! This crate currently contains partial ports; full xterm.js equivalence is not proven.

pub mod buffer;
pub mod charset;
pub mod circular_list;
pub mod color;
pub mod event;
pub mod lifecycle;
pub mod multi_key_map;
pub mod options;
pub mod sorted_list;
pub mod string_builder;
pub mod unicode;
pub mod unicode_v6;

pub mod input {
    pub mod keyboard;
    pub mod text_decoder;
    pub mod win32_input_mode;
    pub mod xparse_color;
}
// Keep the first integrations' root names as aliases of the same modules.
pub use input::{keyboard, text_decoder, win32_input_mode};

pub mod parser {
    pub mod apc;
    pub mod dcs;
    pub mod handler;
    pub mod osc;
    pub mod params;
}
