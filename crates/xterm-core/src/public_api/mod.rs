//! Contracts shared by native terminal APIs and browser bindings.
pub mod addon_manager;
pub mod buffer;
pub mod parser;
pub mod validation;

pub use addon_manager::{AddonError, AddonHandle, AddonManager, TerminalAddon, WeakAddonManager};
pub use buffer::{BufferApiView, BufferNamespaceApi, BufferType, InvalidActiveBuffer};
pub use parser::ParserApi;
