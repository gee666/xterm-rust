//! Rust image protocols, decoded pixels and shared image storage.
//! Live browser addon integration is tracked separately from helper parity.
pub mod base64;
pub mod encoded;
pub mod header;
mod header_codec;
pub mod iip;
pub mod kitty;
pub mod metrics;
pub mod numeric;
pub mod number_map;
pub mod options;
pub mod policy;
pub mod sixel;
pub mod storage;
pub mod types;
pub use header_codec::NameDecodePolicy;
