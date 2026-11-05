// Copyright (c) 2018-2026 The xterm.js authors. All rights reserved.
// Licensed under MIT. Portable GlyphRenderer.ts vertex preparation.
use super::model::ModelError;
use crate::{api::RenderDimensions, renderer::utils::allow_rescaling};
use std::{error::Error, fmt};

pub const GLYPH_STRIDE: usize = 11;

/// Device-pixel raster bounds and normalized coordinates within one atlas page.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RasterizedGlyph {
    pub offset: [f64; 2],
    pub size: [f64; 2],
    pub texture_page: usize,
    pub texture_position: [f64; 2],
    pub texture_size: [f64; 2],
}

/// A missing code denotes the right half of a wide character. Combined strings
/// use JavaScript UTF-16 length when choosing the atlas lookup method.
#[derive(Clone, Copy, Debug)]
pub struct GlyphCell<'a> {
    pub x: usize,
    pub y: usize,
    pub code: Option<i64>,
    pub bg: u32,
    pub fg: u32,
    pub ext: u32,
    pub chars: &'a str,
    pub width: f64,
    pub last_bg: u32,
}

#[derive(Debug)]
pub enum GlyphError {
    Model(ModelError),
    CellOutsideGrid,
    InvalidLineLengths,
}
impl fmt::Display for GlyphError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Model(e) => e.fmt(f),
            Self::CellOutsideGrid => f.write_str("Glyph cell is outside the renderer grid"),
            Self::InvalidLineLengths => f.write_str("Glyph line lengths exceed the renderer grid"),
        }
    }
}
impl Error for GlyphError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Model(e) => Some(e),
            _ => None,
        }
    }
}
impl From<ModelError> for GlyphError {
    fn from(value: ModelError) -> Self {
        Self::Model(value)
    }
}

/// Per-cell vertices and alternating compact upload buffers. Keeping the full
/// grid separate preserves untouched cell positions and clips trailing blanks.
pub struct GlyphGeometry {
    cols: usize,
    rows: usize,
    dimensions: RenderDimensions,
    attributes: Vec<f32>,
    buffers: [Vec<f32>; 2],
    active_buffer: usize,
}
impl GlyphGeometry {
    pub fn new(cols: usize, rows: usize, dimensions: RenderDimensions) -> Result<Self, GlyphError> {
        let mut result = Self {
            cols: 0,
            rows: 0,
            dimensions,
            attributes: Vec::new(),
            buffers: [Vec::new(), Vec::new()],
            active_buffer: 0,
        };
        result.resize(cols, rows)?;
        Ok(result)
    }
    pub fn dimensions(&self) -> RenderDimensions {
        self.dimensions
    }
    pub fn set_dimensions(&mut self, dimensions: RenderDimensions) {
        self.dimensions = dimensions;
    }
    pub fn attributes(&self) -> &[f32] {
        &self.attributes
    }
    pub fn clear(&mut self) {
        self.attributes.fill(0.0);
        for buffer in &mut self.buffers {
            buffer.fill(0.0);
        }
        for y in 0..self.rows {
            for x in 0..self.cols {
                let i = (y * self.cols + x) * GLYPH_STRIDE;
                self.attributes[i + 9] = (x as f64 / self.cols as f64) as f32;
                self.attributes[i + 10] = (y as f64 / self.rows as f64) as f32;
            }
        }
    }
    pub fn resize(&mut self, cols: usize, rows: usize) -> Result<(), GlyphError> {
        let count = cols
            .checked_mul(rows)
            .and_then(|n| n.checked_mul(GLYPH_STRIDE))
            .ok_or(ModelError::DimensionsOverflow)?;
        if count != self.attributes.len() {
            // Allocate before replacing the previous, still usable grid.
            let attributes = zeroes(count)?;
            let buffers = [zeroes(count)?, zeroes(count)?];
            self.attributes = attributes;
            self.buffers = buffers;
        }
        self.cols = cols;
        self.rows = rows;
        self.clear();
        Ok(())
    }
    /// None means no atlas is installed. Non-null cells then remain untouched,
    /// matching the source's early return. Null cells clear even without an atlas.
    pub fn update_cell(
        &mut self,
        cell: GlyphCell<'_>,
        glyph: Option<&RasterizedGlyph>,
        page_width: f64,
        rescale_overlapping: bool,
    ) -> Result<(), GlyphError> {
        if cell.x >= self.cols || cell.y >= self.rows {
            return Err(GlyphError::CellOutsideGrid);
        }
        let i = (cell.y * self.cols + cell.x) * GLYPH_STRIDE;
        if cell.code.is_none() || cell.code == Some(0) {
            // Upstream clears eight entries, not nine. The stale texture height
            // is harmless because size is zero, but remains observable in uploads.
            self.attributes[i..i + 8].fill(0.0);
            return Ok(());
        }
        let Some(glyph) = glyph else {
            return Ok(());
        };
        let d = self.dimensions.device;
        let padding = ((d.cell.width - d.character.width) / 2.0).floor();
        let mut values = [
            -glyph.offset[0] + d.character.left,
            -glyph.offset[1] + d.character.top,
            glyph.size[0] / d.canvas.width,
            glyph.size[1] / d.canvas.height,
            glyph.texture_page as f64,
            glyph.texture_position[0],
            glyph.texture_position[1],
            glyph.texture_size[0],
            glyph.texture_size[1],
        ];
        if cell.bg != cell.last_bg && glyph.offset[0] > padding {
            let clipped = glyph.offset[0] - padding;
            values[0] = -(glyph.offset[0] - clipped) + d.character.left;
            values[2] = (glyph.size[0] - clipped) / d.canvas.width;
            values[5] = glyph.texture_position[0] + clipped / page_width;
            values[7] = glyph.texture_size[0] - clipped / page_width;
        }
        for (target, value) in self.attributes[i..i + 9].iter_mut().zip(values) {
            *target = value as f32;
        }
        if rescale_overlapping
            && allow_rescaling(cell.code, cell.width, glyph.size[0], d.cell.width)
        {
            self.attributes[i + 2] = ((d.cell.width - 1.0) / d.canvas.width) as f32;
        }
        Ok(())
    }
    /// Return only live line prefixes, alternating backing storage each frame.
    pub fn compact(&mut self, line_lengths: &[u32]) -> Result<&[f32], GlyphError> {
        if line_lengths.len() > self.rows || line_lengths.iter().any(|&n| n as usize > self.cols) {
            return Err(GlyphError::InvalidLineLengths);
        }
        self.active_buffer = (self.active_buffer + 1) % 2;
        let buffer = &mut self.buffers[self.active_buffer];
        let mut length = 0;
        for (y, &cells) in line_lengths.iter().enumerate() {
            let start = y * self.cols * GLYPH_STRIDE;
            let count = cells as usize * GLYPH_STRIDE;
            buffer[length..length + count].copy_from_slice(&self.attributes[start..start + count]);
            length += count;
        }
        Ok(&buffer[..length])
    }
}
fn zeroes(count: usize) -> Result<Vec<f32>, ModelError> {
    let mut values = Vec::new();
    values.try_reserve_exact(count)?;
    values.resize(count, 0.0);
    Ok(values)
}
