// Copyright (c) 2018 The xterm.js authors. All rights reserved.
// Licensed under MIT. Port of addon-webgl/src/RectangleRenderer.ts.

use super::model::{
    CursorShape, RenderModel, BACKGROUND_OFFSET, FOREGROUND_OFFSET, INDICES_PER_CELL,
};
use crate::{api::RenderDimensions, theme::SharedColors};
use std::{collections::TryReserveError, error::Error, fmt};
use xterm_core::buffer::constants::{attributes, fg_flags};

pub const FLOATS_PER_RECTANGLE: usize = 8;
const INITIAL_CAPACITY: usize = 20 * FLOATS_PER_RECTANGLE;

#[derive(Debug)]
pub enum RectangleError {
    DimensionsOverflow,
    ModelTooSmall,
    MissingPaletteColor(usize),
    Allocation(TryReserveError),
}
impl fmt::Display for RectangleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DimensionsOverflow => f.write_str("WebGL rectangle dimensions exceed capacity"),
            Self::ModelTooSmall => f.write_str("WebGL model is smaller than the viewport"),
            Self::MissingPaletteColor(index) => write!(f, "WebGL palette has no color {index}"),
            Self::Allocation(error) => error.fmt(f),
        }
    }
}
impl Error for RectangleError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Allocation(error) => Some(error),
            _ => None,
        }
    }
}
impl From<TryReserveError> for RectangleError {
    fn from(error: TryReserveError) -> Self {
        Self::Allocation(error)
    }
}

/// Each instance stores normalized position/size followed by RGBA. The backing
/// array retains unused entries, just as the source Float32Array does.
pub struct RectangleVertices {
    attributes: Vec<f32>,
    count: usize,
}
impl Default for RectangleVertices {
    fn default() -> Self {
        Self {
            attributes: vec![0.0; INITIAL_CAPACITY],
            count: 0,
        }
    }
}
impl RectangleVertices {
    pub fn attributes(&self) -> &[f32] {
        &self.attributes
    }
    pub fn count(&self) -> usize {
        self.count
    }
    pub fn active_attributes(&self) -> &[f32] {
        &self.attributes[..self.count * FLOATS_PER_RECTANGLE]
    }
}

/// Portable background/cursor preparation. The browser renderer forwards theme
/// events into `colors_changed`; direct users must do so after changing colors.
/// Packed model/palette lengths are validated rather than indexing unchecked JS
/// arrays. Cell/canvas geometry otherwise preserves source floating arithmetic.
pub struct RectangleGeometry {
    cols: usize,
    rows: usize,
    dimensions: RenderDimensions,
    colors: SharedColors,
    background: [f32; 4],
    cursor: [f32; 4],
    backgrounds: RectangleVertices,
    cursors: RectangleVertices,
}
impl RectangleGeometry {
    pub fn new(
        cols: usize,
        rows: usize,
        dimensions: RenderDimensions,
        colors: SharedColors,
    ) -> Self {
        let background = rgba(colors.borrow().background.borrow().rgba);
        let cursor = rgba(colors.borrow().cursor.borrow().rgba);
        Self {
            cols,
            rows,
            dimensions,
            colors,
            background,
            cursor,
            backgrounds: RectangleVertices::default(),
            cursors: RectangleVertices::default(),
        }
    }
    pub fn backgrounds(&self) -> &RectangleVertices {
        &self.backgrounds
    }
    pub fn cursors(&self) -> &RectangleVertices {
        &self.cursors
    }
    /// Like source setDimensions, this does not resize or regenerate vertices.
    pub fn set_dimensions(&mut self, dimensions: RenderDimensions) {
        self.dimensions = dimensions;
    }
    pub fn handle_resize(&mut self, cols: usize, rows: usize) {
        self.cols = cols;
        self.rows = rows;
        self.update_viewport();
    }
    pub fn colors_changed(&mut self) {
        self.background = rgba(self.colors.borrow().background.borrow().rgba);
        self.cursor = rgba(self.colors.borrow().cursor.borrow().rgba);
        self.update_viewport();
    }
    fn update_viewport(&mut self) {
        let cell = self.dimensions.device.cell;
        let rect = self.rectangle(
            0.0,
            0.0,
            self.cols as f64 * cell.width,
            self.rows as f64 * cell.height,
            self.background,
        );
        self.backgrounds.attributes[..FLOATS_PER_RECTANGLE].copy_from_slice(&rect);
    }
    pub fn update_backgrounds(&mut self, model: &RenderModel) -> Result<(), RectangleError> {
        let length = self
            .cols
            .checked_mul(self.rows)
            .and_then(|n| n.checked_mul(INDICES_PER_CELL))
            .ok_or(RectangleError::DimensionsOverflow)?;
        if model.cells.len() < length {
            return Err(RectangleError::ModelTooSmall);
        }
        let mut count = 1;
        for y in 0..self.rows {
            let (mut start, mut bg, mut fg, mut inverse) = (0, 0, 0, false);
            for x in 0..self.cols {
                let index = (y * self.cols + x) * INDICES_PER_CELL;
                let next_bg = model.cells[index + BACKGROUND_OFFSET];
                let next_fg = model.cells[index + FOREGROUND_OFFSET];
                let next_inverse = next_fg & fg_flags::INVERSE != 0;
                if next_bg != bg || (next_fg != fg && (inverse || next_inverse)) {
                    if bg != 0 || (inverse && fg != 0) {
                        self.push_background(count, fg, bg, start, x, y)?;
                        count += 1;
                    }
                    (start, bg, fg, inverse) = (x, next_bg, next_fg, next_inverse);
                }
            }
            if bg != 0 || (inverse && fg != 0) {
                self.push_background(count, fg, bg, start, self.cols, y)?;
                count += 1;
            }
        }
        self.backgrounds.count = count;
        Ok(())
    }
    fn push_background(
        &mut self,
        count: usize,
        fg: u32,
        bg: u32,
        start: usize,
        end: usize,
        y: usize,
    ) -> Result<(), RectangleError> {
        let inverse = fg & fg_flags::INVERSE != 0;
        let value = if inverse { fg } else { bg };
        let colors = self.colors.borrow();
        let color = match value & attributes::CM_MASK {
            attributes::CM_P16 | attributes::CM_P256 => {
                let index = (value & attributes::PCOLOR_MASK) as usize;
                colors
                    .ansi
                    .get(index)
                    .ok_or(RectangleError::MissingPaletteColor(index))?
                    .borrow()
                    .rgba
            }
            attributes::CM_RGB => (value & attributes::RGB_MASK) << 8,
            _ if inverse => colors.foreground.borrow().rgba,
            _ => colors.background.borrow().rgba,
        };
        drop(colors);
        let mut color = rgba(color);
        // Non-default rectangles are always opaque, even with transparent themes.
        color[3] = 1.0;
        let cell = self.dimensions.device.cell;
        let rect = self.rectangle(
            start as f64 * cell.width,
            y as f64 * cell.height,
            (end - start) as f64 * cell.width,
            cell.height,
            color,
        );
        let offset = count * FLOATS_PER_RECTANGLE;
        if self.backgrounds.attributes.len() < offset + FLOATS_PER_RECTANGLE {
            let max = self
                .cols
                .checked_mul(self.rows)
                .and_then(|n| n.checked_add(1))
                .and_then(|n| n.checked_mul(FLOATS_PER_RECTANGLE))
                .ok_or(RectangleError::DimensionsOverflow)?;
            let old = self.backgrounds.attributes.len();
            let length = old.saturating_mul(2).min(max);
            self.backgrounds
                .attributes
                .try_reserve_exact(length - old)?;
            self.backgrounds.attributes.resize(length, 0.0);
        }
        self.backgrounds.attributes[offset..offset + FLOATS_PER_RECTANGLE].copy_from_slice(&rect);
        Ok(())
    }
    pub fn update_cursor(&mut self, model: &RenderModel) {
        self.cursors.count = 0;
        let Some(cursor) = model.cursor else {
            return;
        };
        let cell = self.dimensions.device.cell;
        let (x, y) = (cursor.x as f64 * cell.width, cursor.y as f64 * cell.height);
        let width = cursor.width as f64 * cell.width;
        let dpr = cursor.device_pixel_ratio;
        if matches!(cursor.style, CursorShape::Bar | CursorShape::Outline) {
            let edge = if cursor.style == CursorShape::Bar {
                dpr * cursor.cursor_width
            } else {
                dpr
            };
            self.push_cursor(x, y, edge, cell.height);
        }
        if matches!(cursor.style, CursorShape::Underline | CursorShape::Outline) {
            self.push_cursor(x, (cursor.y as f64 + 1.0) * cell.height - dpr, width, dpr);
        }
        if cursor.style == CursorShape::Outline {
            self.push_cursor(x, y, width, dpr);
            self.push_cursor(
                (cursor.x as f64 + cursor.width as f64) * cell.width - dpr,
                y,
                dpr,
                cell.height,
            );
        }
    }
    fn push_cursor(&mut self, x: f64, y: f64, width: f64, height: f64) {
        let rect = self.rectangle(x, y, width, height, self.cursor);
        let offset = self.cursors.count * FLOATS_PER_RECTANGLE;
        self.cursors.attributes[offset..offset + FLOATS_PER_RECTANGLE].copy_from_slice(&rect);
        self.cursors.count += 1;
    }
    fn rectangle(&self, x: f64, y: f64, width: f64, height: f64, color: [f32; 4]) -> [f32; 8] {
        let canvas = self.dimensions.device.canvas;
        [
            (x / canvas.width) as f32,
            (y / canvas.height) as f32,
            (width / canvas.width) as f32,
            (height / canvas.height) as f32,
            color[0],
            color[1],
            color[2],
            color[3],
        ]
    }
}
fn rgba(value: u32) -> [f32; 4] {
    [24, 16, 8, 0].map(|shift| (f64::from((value >> shift) & 255) / 255.0) as f32)
}
