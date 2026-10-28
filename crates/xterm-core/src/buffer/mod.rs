//! Packed cells and attributes shared by buffers, renderers and terminal APIs.
mod attributes;
mod cell;
pub mod cleanup;
pub mod constants;
mod extended;
mod line;
pub mod marker;
pub mod range;
pub mod reflow;
pub mod screen;
pub use cleanup::BufferCleanup;
pub mod buffer_set;
pub mod service;
pub use buffer_set::BufferSet;
pub use screen::{Buffer, BufferDimensions, BufferError, LineRef};
pub use service::BufferService;

pub use attributes::AttributeData;
pub use cell::{CellData, CharData};
pub use extended::ExtendedAttrs;
pub use line::BufferLine;
