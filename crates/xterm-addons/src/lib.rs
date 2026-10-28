//! xterm.js addon behavior, separate from terminal engine and browser bindings.
//! Adapter contracts preserve event registration and disposal explicitly.

pub mod attach;
pub mod base64;
pub mod clipboard;
pub mod fit;
pub mod image;
pub mod ligatures;
pub mod link_snapshot;
pub mod native_status;
pub mod native_utility;
pub mod program_status;
pub mod progress;
pub mod status_terminal;
pub mod web_fonts;
pub mod web_link_provider;
pub mod web_links;
