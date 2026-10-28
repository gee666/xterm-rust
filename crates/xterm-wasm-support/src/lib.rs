//! Shared DOM-free WASM adapters used by the browser terminal.
//! Terminal parsing and state remain in xterm-core; actual DOM integration and
//! full Terminal tests remain in xterm-web. Component checks do not prove either.

pub mod api {
    pub use xterm_core::public_api::validation::verify_mutable_option;
}
#[cfg(target_arch = "wasm32")]
pub mod events;
#[cfg(target_arch = "wasm32")]
pub mod public_api;
#[cfg(target_arch = "wasm32")]
pub mod js_error;
#[cfg(target_arch = "wasm32")]
pub mod logger;
#[cfg(target_arch = "wasm32")]
pub mod options;
#[cfg(target_arch = "wasm32")]
pub mod utf16;
