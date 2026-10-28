//! Packed cells and attributes shared by buffers, renderers and terminal APIs.
mod attributes;
mod cell;
pub mod constants;
mod extended;
mod line;
pub mod reflow;
pub mod marker;
pub mod range;
pub mod screen;
pub mod cleanup;
pub use cleanup::BufferCleanup;
pub mod buffer_set;
pub mod service;
pub use screen::{Buffer, BufferDimensions, BufferError, LineRef};
pub use buffer_set::BufferSet;
pub use service::BufferService;

pub use attributes::AttributeData;
pub use cell::{CellData, CharData};
pub use extended::ExtendedAttrs;
pub use line::BufferLine;
