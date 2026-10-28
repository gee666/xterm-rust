//! Platform-independent terminal state, parsing, and shared utilities.
//!
//! Browser and operating-system integration belong in separate adapter crates.
//! This crate currently contains partial ports; full xterm.js equivalence is not proven.

pub mod buffer;
pub mod charset;
pub mod circular_list;
pub mod color;
pub mod core_service;
pub mod decoration;
pub mod event;
pub mod headless;
pub mod instantiation;
pub mod lifecycle;
pub mod log_service;
pub mod mouse_state;
pub mod multi_key_map;
pub mod options;
pub mod osc_link_service;
pub mod platform;
pub mod public_api;
pub mod search;
pub mod serialize;
pub mod service_registry;
pub mod sorted_list;
pub mod string_builder;
pub mod task_queue;
pub mod terminal;
pub mod unicode;
pub mod unicode11;
pub mod unicode_addons;
pub mod unicode_graphemes;
pub mod unicode_properties;
pub mod unicode_service;
pub mod unicode_v6;
pub mod version;
pub mod windows_mode;

pub mod input {
    pub mod keyboard;
    pub mod keyboard_event_source;
    pub mod kitty_keyboard;
    pub mod move_to_cell;
    pub mod parse_source;
    pub mod text_decoder;
    pub mod win32_input_mode;
    pub mod write_buffer;
    pub mod xparse_color;
}
// Keep the first integrations' root names as aliases of the same modules.
pub use input::{keyboard, text_decoder, win32_input_mode};

pub mod data {
    pub mod escape_sequences;
}

pub mod parser {
    pub mod apc;
    pub mod dcs;
    pub mod escape;
    pub mod handler;
    pub mod osc;
    pub mod params;
    pub mod table;
}
