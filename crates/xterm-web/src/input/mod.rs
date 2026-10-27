//! Browser input policy. UTF-16 vectors retain JS string indices and lone surrogates.

pub mod clipboard;
pub mod composition;
pub mod composition_geometry;
pub mod keyboard;
pub mod keyboard_dispatch;
pub mod keyboard_dispatch_types;
pub mod mouse_coords;
pub mod mouse_policy;
pub mod mouse_wheel;

#[cfg(target_arch = "wasm32")]
pub mod dom;
