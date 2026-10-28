// Copyright (c) 2026 The xterm.js authors. All rights reserved. MIT license.
// Pure policy stages from KittyGraphicsHandler._decodeAndDisplay. Platform
// bitmap/canvas operations happen between stages and supply actual dimensions.
use super::command::KittyCommand;
use crate::image::storage::{
    AddImageOptions, CellSize, CursorPosition, ImageLayer, CELL_SIZE_DEFAULT,
};
use std::fmt;

#[derive(Clone, Copy, Debug)]
pub struct BitmapSize {
    pub width: u32,
    pub height: u32,
}
#[derive(Clone, Copy, Debug)]
pub struct CropPlan {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub required: bool,
}
#[derive(Clone, Copy, Debug)]
pub struct Layout {
    pub cell: CellSize,
    pub columns: f64,
    pub rows: f64,
    pub width: f64,
    pub height: f64,
    pub layer: ImageLayer,
}
#[derive(Clone, Copy, Debug)]
pub struct OffsetPlan {
    pub x: f64,
    pub y: f64,
    pub canvas_width: f64,
    pub canvas_height: f64,
}
#[derive(Clone, Copy, Debug)]
pub struct CursorChange {
    pub x: f64,
    pub y: Option<f64>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlacementError {
    InvalidSourceRectangle,
    PixelLimit,
}
impl fmt::Display for PlacementError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidSourceRectangle => "invalid source rectangle",
            Self::PixelLimit => "image exceeds pixel limit",
        })
    }
}
impl std::error::Error for PlacementError {}

fn truthy(value: f64) -> bool {
    value != 0.0 && !value.is_nan()
}
fn min(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else if a == 0.0 && b == 0.0 {
        if a.is_sign_negative() || b.is_sign_negative() {
            -0.0
        } else {
            0.0
        }
    } else {
        a.min(b)
    }
}
fn max(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else if a == 0.0 && b == 0.0 {
        if a.is_sign_negative() && b.is_sign_negative() {
            -0.0
        } else {
            0.0
        }
    } else {
        a.max(b)
    }
}
fn round(value: f64) -> f64 {
    if !value.is_finite() || value == 0.0 {
        return value;
    }
    let floor = value.floor();
    let rounded = if value - floor < 0.5 {
        floor
    } else {
        floor + 1.0
    };
    if rounded == 0.0 && value.is_sign_negative() {
        -0.0
    } else {
        rounded
    }
}
fn check_pixels(width: f64, height: f64, limit: f64) -> Result<(), PlacementError> {
    if width * height > limit {
        Err(PlacementError::PixelLimit)
    } else {
        Ok(())
    }
}

pub fn crop(size: BitmapSize, command: &KittyCommand) -> Result<CropPlan, PlacementError> {
    let width = f64::from(size.width);
    let height = f64::from(size.height);
    let x = max(0.0, command.x.unwrap_or(0.0));
    let y = max(0.0, command.y.unwrap_or(0.0));
    let requested_width = command
        .source_width
        .filter(|value| truthy(*value))
        .unwrap_or(width - x);
    let requested_height = command
        .source_height
        .filter(|value| truthy(*value))
        .unwrap_or(height - y);
    let final_width = max(0.0, min(requested_width, max(0.0, width - x)));
    let final_height = max(0.0, min(requested_height, max(0.0, height - y)));
    if final_width == 0.0 || final_height == 0.0 {
        return Err(PlacementError::InvalidSourceRectangle);
    }
    Ok(CropPlan {
        x,
        y,
        width: final_width,
        height: final_height,
        required: x != 0.0 || y != 0.0 || final_width != width || final_height != height,
    })
}

/// Call after any actual crop operation, using its returned bitmap dimensions.
/// A zero/NaN cell metric uses source defaults; negative/infinite metrics remain
/// source values until the real platform operation accepts or rejects them.
pub fn layout(
    size: BitmapSize,
    measured: CellSize,
    command: &KittyCommand,
    pixel_limit: f64,
) -> Result<Layout, PlacementError> {
    let cell = CellSize {
        width: if truthy(measured.width) {
            measured.width
        } else {
            CELL_SIZE_DEFAULT.width
        },
        height: if truthy(measured.height) {
            measured.height
        } else {
            CELL_SIZE_DEFAULT.height
        },
    };
    let width = f64::from(size.width);
    let height = f64::from(size.height);
    let (columns, rows) = match (command.columns, command.rows) {
        (Some(columns), Some(rows)) => (columns, rows),
        (Some(columns), None) => (
            columns,
            max(
                1.0,
                ((height / width) * (columns * cell.width) / cell.height).ceil(),
            ),
        ),
        (None, Some(rows)) => (
            max(
                1.0,
                ((width / height) * (rows * cell.height) / cell.width).ceil(),
            ),
            rows,
        ),
        (None, None) => ((width / cell.width).ceil(), (height / cell.height).ceil()),
    };
    let (target_width, target_height) = if command.columns.is_some() || command.rows.is_some() {
        (round(columns * cell.width), round(rows * cell.height))
    } else {
        (width, height)
    };
    check_pixels(target_width, target_height, pixel_limit)?;
    Ok(Layout {
        cell,
        columns,
        rows,
        width: target_width,
        height: target_height,
        layer: if command.z_index.is_some_and(|z| z < 0.0) {
            ImageLayer::Bottom
        } else {
            ImageLayer::Top
        },
    })
}

/// Compute after the platform's actual resize, not from unconverted float sizes.
pub fn offset(size: BitmapSize, layout: Layout, command: &KittyCommand) -> Option<OffsetPlan> {
    let x = min(
        max(0.0, command.x_offset.unwrap_or(0.0)),
        layout.cell.width - 1.0,
    );
    let y = min(
        max(0.0, command.y_offset.unwrap_or(0.0)),
        layout.cell.height - 1.0,
    );
    if x == 0.0 && y == 0.0 {
        return None;
    }
    Some(OffsetPlan {
        x,
        y,
        canvas_width: if command.columns.is_some() {
            round(layout.columns * layout.cell.width)
        } else {
            f64::from(size.width) + x
        },
        canvas_height: if command.rows.is_some() {
            round(layout.rows * layout.cell.height)
        } else {
            f64::from(size.height) + y
        },
    })
}

/// Use current command presence after the offset bitmap promise resolves. The
/// source permits retained pending command objects to change while awaiting.
pub fn complete_offset(
    size: BitmapSize,
    mut layout: Layout,
    command: &KittyCommand,
    pixel_limit: f64,
) -> Result<Layout, PlacementError> {
    layout.width = f64::from(size.width);
    layout.height = f64::from(size.height);
    check_pixels(layout.width, layout.height, pixel_limit)?;
    if command.columns.is_none() {
        layout.columns = (layout.width / layout.cell.width).ceil();
    }
    if command.rows.is_none() {
        layout.rows = (layout.height / layout.cell.height).ceil();
    }
    Ok(layout)
}

/// Layer was chosen before resize/offset awaits, but the source reads z-index
/// again immediately before storage insertion. A retained command may change.
pub fn placement_options(layout: Layout, command: &KittyCommand) -> AddImageOptions {
    AddImageOptions {
        scrolling: true,
        layer: layout.layer,
        z_index: command.z_index.unwrap_or(0.0),
        cursor_position: CursorPosition::Iip,
    }
}

pub fn cursor(
    command: &KittyCommand,
    saved_x: f64,
    saved_y: f64,
    saved_ybase: f64,
    current_ybase: f64,
    columns: f64,
    image_columns: f64,
) -> CursorChange {
    if command.cursor_movement == Some(1.0) {
        CursorChange {
            x: saved_x,
            y: Some(max(saved_y - (current_ybase - saved_ybase), 0.0)),
        }
    } else {
        CursorChange {
            x: min(saved_x + image_columns, columns),
            y: None,
        }
    }
}
