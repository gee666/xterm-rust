// Copyright (c) 2020 The xterm.js authors. All rights reserved. MIT license.
use super::ImageResource;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use xterm_core::buffer::marker::Marker;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CellSize {
    pub width: f64,
    pub height: f64,
}
pub const CELL_SIZE_DEFAULT: CellSize = CellSize {
    width: 7.0,
    height: 14.0,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageLayer {
    Top,
    Bottom,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CursorPosition {
    Vt340,
    Iip,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImageBufferType {
    Normal,
    Alternate,
}

#[derive(Clone, Copy, Debug)]
pub struct AddImageOptions {
    pub scrolling: bool,
    pub layer: ImageLayer,
    pub z_index: f64,
    pub cursor_position: CursorPosition,
}
impl Default for AddImageOptions {
    fn default() -> Self {
        Self {
            scrolling: true,
            layer: ImageLayer::Top,
            z_index: 0.0,
            cursor_position: CursorPosition::Vt340,
        }
    }
}

pub struct ImageStorageOptions {
    pub storage_limit: f64,
    /// Shared with the addon setter, so render reads the current option rather
    /// than a constructor snapshot or a second mirrored policy field.
    pub show_placeholder: Rc<Cell<bool>>,
}
impl Default for ImageStorageOptions {
    fn default() -> Self {
        Self {
            storage_limit: 128.0,
            show_placeholder: Rc::new(Cell::new(true)),
        }
    }
}

/// Isolated mutable cell payload. Overwrite reuses this exact object.
#[derive(Debug)]
pub struct ImageTileInfo {
    pub image_id: Cell<i64>,
    pub tile_id: Cell<f64>,
}
impl ImageTileInfo {
    pub fn new(image_id: i64, tile_id: f64) -> Self {
        Self {
            image_id: Cell::new(image_id),
            tile_id: Cell::new(tile_id),
        }
    }
}
impl Default for ImageTileInfo {
    fn default() -> Self {
        Self::new(-1, -1.0)
    }
}

pub struct ImageSpec {
    pub id: u64,
    pub original: ImageResource,
    pub(super) actual: RefCell<Option<ImageResource>>,
    pub original_cell_size: CellSize,
    pub actual_cell_size: Cell<CellSize>,
    pub marker: Option<Marker>,
    pub tile_count: Cell<f64>,
    pub buffer_type: ImageBufferType,
    pub layer: ImageLayer,
    pub z_index: f64,
}
