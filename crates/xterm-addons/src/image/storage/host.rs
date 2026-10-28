// Copyright (c) 2020 The xterm.js authors. All rights reserved. MIT license.
use super::{CellSize, ImageBufferType, ImageLayer, ImageResource, ImageSpec, StorageError};
use std::rc::Rc;
use xterm_core::buffer::{marker::Marker, BufferService};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StorageDiagnostic {
    Error,
    Warning,
}

/// Actual terminal effects. Implementations must release terminal-state borrows
/// before dispatching events. Direct linefeed is not parser input reentrancy.
pub trait StorageTerminal {
    fn buffer_service(&self) -> Rc<BufferService>;
    fn active_buffer_type(&self) -> ImageBufferType;
    fn mark_dirty(&self, row: usize) -> Result<(), StorageError>;
    fn line_feed(&self) -> Result<(), StorageError>;
    fn register_marker(&self, offset: i64) -> Result<Option<Marker>, StorageError>;
    fn diagnostic(&self, level: StorageDiagnostic, message: &str) -> Result<(), StorageError>;
}

/// Platform drawing/resource operations. Storage calls these with no borrowed
/// map or Buffer, so synchronous host callbacks can observe partial state.
pub trait StorageRenderer {
    fn cell_size(&self) -> Result<CellSize, StorageError>;
    fn has_layer(&self, layer: ImageLayer) -> bool;
    fn insert_layer(&self, layer: ImageLayer) -> Result<(), StorageError>;
    fn remove_layer(&self, layer: ImageLayer) -> Result<(), StorageError>;
    fn rescale_canvas(&self) -> Result<(), StorageError>;
    fn clear_all(&self, layer: Option<ImageLayer>) -> Result<(), StorageError>;
    fn clear_lines(&self, start: usize, end: usize) -> Result<(), StorageError>;
    fn draw_placeholder(&self, col: usize, row: usize, count: usize) -> Result<(), StorageError>;
    fn draw(
        &self,
        spec: &ImageSpec,
        tile_id: f64,
        col: usize,
        row: usize,
        count: usize,
    ) -> Result<(), StorageError>;
    fn extract_tile(
        &self,
        spec: &ImageSpec,
        tile_id: f64,
    ) -> Result<Option<ImageResource>, StorageError>;
    fn bitmap_to_canvas(&self, image: &ImageResource) -> Result<ImageResource, StorageError>;
}
