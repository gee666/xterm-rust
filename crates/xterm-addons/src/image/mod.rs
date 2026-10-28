//! Rust image protocols, decoded pixels and shared image storage.
//! Live browser addon integration is tracked separately from helper parity.
pub mod header;
mod header_codec;
pub mod metrics;
pub mod types;
pub use header_codec::NameDecodePolicy;
