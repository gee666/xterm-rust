//! DOM renderer components. Font measurements and row planning are separate
//! from their browser adapters so exact source algorithms can be tested natively.
#[cfg(target_arch = "wasm32")]
pub mod canvas_width;
pub mod color_zone_store;
#[cfg(target_arch = "wasm32")]
mod cursor_blink;
#[cfg(target_arch = "wasm32")]
pub mod dom;
pub mod geometry;
#[cfg(target_arch = "wasm32")]
mod options;
pub mod row;
mod row_colors;
#[cfg(target_arch = "wasm32")]
pub mod row_dom;
pub mod row_types;
pub mod selection_model;
#[cfg(target_arch = "wasm32")]
pub mod service_api;
#[cfg(target_arch = "wasm32")]
mod text_blink;
pub mod theme_css;
pub mod width_cache;
