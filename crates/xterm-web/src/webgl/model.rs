// Copyright (c) 2018 The xterm.js authors. All rights reserved.
// Licensed under MIT. Port of addon-webgl/src/RenderModel.ts.

use crate::renderer::selection_model::SelectionRenderModel;
use std::{collections::TryReserveError, error::Error, fmt};

pub const INDICES_PER_CELL: usize = 4;
pub const BACKGROUND_OFFSET: usize = 1;
pub const FOREGROUND_OFFSET: usize = 2;
pub const EXTENDED_OFFSET: usize = 3;
pub const COMBINED_CHAR_BIT_MASK: u32 = 0x8000_0000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CursorShape {
    Block,
    Bar,
    Underline,
    Outline,
    None,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CursorRenderModel {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub style: CursorShape,
    pub cursor_width: f64,
    pub device_pixel_ratio: f64,
}

#[derive(Debug)]
pub enum ModelError {
    DimensionsOverflow,
    Allocation(TryReserveError),
}
impl fmt::Display for ModelError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DimensionsOverflow => {
                formatter.write_str("WebGL render model dimensions exceed native capacity")
            }
            Self::Allocation(error) => error.fmt(formatter),
        }
    }
}
impl Error for ModelError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Allocation(error) => Some(error),
            Self::DimensionsOverflow => None,
        }
    }
}
impl From<TryReserveError> for ModelError {
    fn from(error: TryReserveError) -> Self {
        Self::Allocation(error)
    }
}

/// Packed cell attributes and line draw lengths. The selection/cursor survive
/// both clear and resize, exactly as upstream's shared render model does.
#[derive(Debug, Default)]
pub struct RenderModel {
    pub cells: Vec<u32>,
    pub line_lengths: Vec<u32>,
    pub selection: SelectionRenderModel,
    pub cursor: Option<CursorRenderModel>,
}
impl RenderModel {
    pub fn new() -> Self {
        Self::default()
    }

    /// Equal total cell counts preserve both arrays, even if row/column factors
    /// changed. Upstream keys reallocation on packed length, not dimension pairs.
    pub fn resize(&mut self, cols: usize, rows: usize) -> Result<(), ModelError> {
        let count = cols
            .checked_mul(rows)
            .and_then(|cells| cells.checked_mul(INDICES_PER_CELL))
            .ok_or(ModelError::DimensionsOverflow)?;
        if count != self.cells.len() {
            self.cells = zeroes(count)?;
            // Do not turn a second allocation failure into an atomic rollback:
            // the source has already replaced cells when lineLengths fails.
            self.line_lengths = zeroes(rows)?;
        }
        Ok(())
    }

    pub fn clear(&mut self) {
        self.cells.fill(0);
        self.line_lengths.fill(0);
    }
}
fn zeroes(length: usize) -> Result<Vec<u32>, TryReserveError> {
    let mut values = Vec::new();
    values.try_reserve_exact(length)?;
    values.resize(length, 0);
    Ok(values)
}
