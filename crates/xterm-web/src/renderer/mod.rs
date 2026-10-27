//! DOM renderer components. Font measurements and row planning are separate
//! from their browser adapters so exact source algorithms can be tested natively.
#[cfg(target_arch = "wasm32")]
pub mod canvas_width;
pub mod width_cache;
