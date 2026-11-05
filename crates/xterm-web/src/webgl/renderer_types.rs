// Copyright (c) 2018-2026 The xterm.js authors. All rights reserved. MIT license.
//! Typed dependencies of the incremental WebglRenderer algorithm.
use super::{
    cell_colors::CellColorResolver, glyphs::GlyphCell, model::CursorShape, model::RenderModel,
};
use std::{error::Error, fmt};
use xterm_core::buffer::{AttributeData, CellData};

/// Native service values for one update. Raw JavaScript accessors must be read
/// at their original phases by the browser adapter, not converted to this snapshot.
#[derive(Clone, Copy, Debug)]
pub struct UpdateContext {
    pub cols: usize,
    pub rows: usize,
    pub viewport_y: usize,
    pub cursor_x: usize,
    pub cursor_y: usize,
    pub focused: bool,
    pub cursor_initialized: bool,
    pub cursor_hidden: bool,
    pub cursor_blink_visible: bool,
    pub cursor_style: CursorShape,
    pub inactive_style: CursorShape,
    pub cursor_width: f64,
    pub device_pixel_ratio: f64,
    pub text_blink_enabled: bool,
    pub text_blink_on: bool,
}

/// Joined cells intentionally inherit only fg/bg. Extended attributes are not
/// copied by the source JoinedCellData constructor.
pub struct ModelCell {
    pub attributes: AttributeData,
    pub chars: Vec<u16>,
    pub code: Option<u32>,
    pub width: usize,
}
impl ModelCell {
    pub(super) fn load(cell: &CellData) -> Option<Self> {
        Some(Self {
            attributes: cell.attributes.clone(),
            chars: cell.get_chars(),
            code: cell.get_code(),
            width: usize::try_from(cell.get_width()).ok()?,
        })
    }
}

/// Exact JavaScript color arguments alongside GlyphCell's packed words. A GPU
/// adapter must compare these numbers before clipping overlapping backgrounds;
/// comparing just the u32 bits loses signed-versus-unsigned inequality.
#[derive(Clone, Copy, Debug, Default)]
pub struct GlyphColorNumbers {
    pub current: [i64; 3],
    pub previous_background: i64,
}

/// Each operation runs at its source call site. Errors stop immediately and
/// preserve already-written model/glyph state. The engine holds no RefCell
/// borrow across a host call. Raw browser reentry still needs its own adapter.
pub trait UpdateHost {
    type Error;
    fn has_line(&mut self, row: usize) -> Result<bool, Self::Error>;
    fn joined_characters(&mut self, row: usize) -> Result<Vec<[usize; 2]>, Self::Error>;
    fn load_cell(&mut self, row: usize, x: usize, cell: &mut CellData) -> Result<(), Self::Error>;
    fn joined_text(&mut self, row: usize, range: [usize; 2]) -> Result<Vec<u16>, Self::Error>;
    fn resolve_colors(
        &mut self,
        cell: &ModelCell,
        x: usize,
        row: usize,
        selected: bool,
        resolver: &mut CellColorResolver,
    ) -> Result<(), Self::Error>;
    /// RGBA cursor accent and cursor background, read only for a block cell.
    fn cursor_colors(&mut self) -> Result<[u32; 2], Self::Error>;
    fn update_glyph(
        &mut self,
        cell: GlyphCell<'_>,
        numbers: GlyphColorNumbers,
    ) -> Result<(), Self::Error>;
    fn update_backgrounds(&mut self, model: &RenderModel) -> Result<(), Self::Error>;
    fn update_cursor(&mut self, model: &RenderModel) -> Result<(), Self::Error>;
    fn needs_text_blink(&mut self, needed: bool) -> Result<(), Self::Error>;
}

#[derive(Debug)]
pub enum UpdateError<E> {
    Host(E),
    InvalidGrid,
    InvalidJoin([usize; 2]),
}
impl<E: fmt::Display> fmt::Display for UpdateError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Host(e) => e.fmt(f),
            Self::InvalidGrid => f.write_str(
                "WebGL update requires a resized, nonempty grid and nonnegative cell widths",
            ),
            Self::InvalidJoin(range) => write!(
                f,
                "WebGL join range {range:?} is outside the grid or does not progress"
            ),
        }
    }
}
impl<E: Error + 'static> Error for UpdateError<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Host(e) => Some(e),
            _ => None,
        }
    }
}
