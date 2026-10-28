//! Headless public terminal contracts. The engine remains platform independent.
mod error;
mod events;
mod modes;
mod native_queue;
mod options_view;
mod queue_bridge;
mod queued;
mod scroll;
mod terminal;
mod views;
pub use error::HeadlessError;
pub use events::{RenderEvent, ResizeEvent};
pub use modes::{ModesSnapshot, MouseTrackingMode};
pub use native_queue::{NativeWritePump, NativeWriteScheduler};
pub use options_view::{PublicOptionError, PublicOptions};
pub use queued::QueuedTerminal;
pub use terminal::Terminal;
