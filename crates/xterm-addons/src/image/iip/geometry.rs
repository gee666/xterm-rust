// Copyright (c) 2023 The xterm.js authors. MIT license.
//! IIP sizing uses binary64 geometry until a checked raster allocation boundary.
use crate::image::numeric::parse_decimal_prefix;
pub use crate::image::storage::CellSize;
pub const DEFAULT_CELL_SIZE: CellSize = crate::image::storage::CELL_SIZE_DEFAULT;
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderMetrics {
    pub cell: CellSize,
    pub canvas: CellSize,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct IipGeometry {
    pub columns: f64,
    pub rows: f64,
    pub render: Option<RenderMetrics>,
    pub device_pixel_ratio: Option<f64>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct SizeRequest {
    pub width: Vec<u16>,
    pub height: Vec<u16>,
    pub preserve_aspect_ratio: f64,
}
impl Default for SizeRequest {
    fn default() -> Self {
        Self {
            width: "auto".encode_utf16().collect(),
            height: "auto".encode_utf16().collect(),
            preserve_aspect_ratio: 1.0,
        }
    }
}
fn truthy(value: f64) -> bool {
    value != 0.0 && !value.is_nan()
}
fn or(value: Option<f64>, fallback: f64) -> f64 {
    value.filter(|v| truthy(*v)).unwrap_or(fallback)
}

impl IipGeometry {
    pub fn resize(&self, width: f64, height: f64, request: &SizeRequest) -> CellSize {
        let cw = or(self.render.map(|m| m.cell.width), DEFAULT_CELL_SIZE.width);
        let ch = or(self.render.map(|m| m.cell.height), DEFAULT_CELL_SIZE.height);
        let canvas_width = or(self.render.map(|m| m.canvas.width), cw * self.columns);
        let canvas_height = or(self.render.map(|m| m.canvas.height), ch * self.rows);
        let requested_width = dimension(&request.width, canvas_width, cw);
        let requested_height = dimension(&request.height, canvas_height, ch);
        if !truthy(requested_width) && !truthy(requested_height) {
            let width_factor = canvas_width / width;
            let height_factor = (canvas_height - ch) / height;
            // Rust f64::min ignores NaN; Math.min propagates it.
            let factor = if width_factor.is_nan() || height_factor.is_nan() {
                f64::NAN
            } else if width_factor == 0.0
                && height_factor == 0.0
                && (width_factor.is_sign_negative() || height_factor.is_sign_negative())
            {
                -0.0
            } else {
                width_factor.min(height_factor)
            };
            return if factor < 1.0 {
                CellSize {
                    width: width * factor,
                    height: height * factor,
                }
            } else {
                CellSize { width, height }
            };
        }
        if !truthy(requested_width) {
            CellSize {
                width: width * requested_height / height,
                height: requested_height,
            }
        } else if truthy(request.preserve_aspect_ratio) || !truthy(requested_height) {
            CellSize {
                width: requested_width,
                height: height * requested_width / width,
            }
        } else {
            CellSize {
                width: requested_width,
                height: requested_height,
            }
        }
    }
    pub fn report_cell_size(&self) -> CellSize {
        self.render.map_or(DEFAULT_CELL_SIZE, |metrics| CellSize {
            width: metrics.canvas.width / self.columns,
            height: metrics.canvas.height / self.rows,
        })
    }
}

pub fn dimension(value: &[u16], total: f64, cell: f64) -> f64 {
    if value == [97, 117, 116, 111] {
        return 0.0;
    }
    if value.ends_with(&[37]) {
        parse_decimal_prefix(&value[..value.len() - 1]) * total / 100.0
    } else if value.ends_with(&[112, 120]) {
        parse_decimal_prefix(&value[..value.len() - 2])
    } else {
        parse_decimal_prefix(value) * cell
    }
}
