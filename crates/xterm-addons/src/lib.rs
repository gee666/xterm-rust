//! xterm.js addon behavior, separate from terminal engine and browser bindings.
//! Adapter contracts preserve event registration and disposal explicitly.

pub mod attach;
pub mod base64;
pub mod clipboard;
pub mod fit;
pub mod ligatures;
pub mod web_fonts;
pub mod program_status;
pub mod progress;
pub mod status_terminal;
pub mod native_status;
pub mod web_links;
pub mod web_link_provider;
