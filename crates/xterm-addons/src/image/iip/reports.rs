// Copyright (c) 2023 The xterm.js authors. MIT license.
//! Exact ECMAScript toFixed(3) for IIP reports, including binary rounding ties.
use super::geometry::IipGeometry;

pub use crate::image::numeric::fixed_three;

pub fn cell_size_report(geometry: &IipGeometry) -> Vec<u16> {
    let cell = geometry.report_cell_size();
    let scale = geometry.device_pixel_ratio.unwrap_or(1.0);
    format!(
        "\x1b]1337;ReportCellSize={};{};{}\x1b\\",
        fixed_three(cell.height),
        fixed_three(cell.width),
        fixed_three(scale)
    )
    .encode_utf16()
    .collect()
}
