//! Browser adapters for Rust addon logic. The core never imports DOM types.

pub mod attach_terminal;
pub mod bindings;
pub mod clipboard;
pub mod clipboard_bindings;
pub mod fit;
pub mod fit_bindings;
pub mod js_property;
pub mod link_bindings;
pub mod matcher;
pub mod open_link;
pub mod provider;
pub mod socket;
