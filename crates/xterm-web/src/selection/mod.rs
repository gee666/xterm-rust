//! Browser selection coordinates and range operations.
#[cfg(target_arch = "wasm32")]
pub mod dom;
pub mod model;
pub mod pointer;
pub mod service;
pub mod text;
pub mod word;
