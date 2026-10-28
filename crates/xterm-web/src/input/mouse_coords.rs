// Copyright (c) 2017, 2026 The xterm.js authors. All rights reserved. MIT license.
//! Coordinate policy from browser/input/Mouse.ts and MouseCoordsService.ts.
//! DOM adapters sample geometry from the element's current owner window.

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MouseGeometry {
    pub rect_left: f64,
    pub rect_top: f64,
    pub padding_left: f64,
    pub padding_top: f64,
    pub valid_char_size: bool,
    pub cell_width: f64,
    pub cell_height: f64,
    pub canvas_width: f64,
    pub canvas_height: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReportCoords {
    pub col: f64,
    pub row: f64,
    pub x: f64,
    pub y: f64,
}

/// CSS padding is parsed with decimal parseInt, not parseFloat. Invalid text
/// remains NaN, matching a failed computed-style parse in the source.
pub fn parse_css_padding(text: &str) -> f64 {
    let text = text.trim_start_matches(|ch| {
        matches!(ch,
        '\u{0009}'..='\u{000d}' | '\u{0020}' | '\u{00a0}' | '\u{1680}' |
        '\u{2000}'..='\u{200a}' | '\u{2028}' | '\u{2029}' | '\u{202f}' |
        '\u{205f}' | '\u{3000}' | '\u{feff}')
    });
    let (negative, digits) = match text.as_bytes().first() {
        Some(b'-') => (true, &text[1..]),
        Some(b'+') => (false, &text[1..]),
        _ => (false, text),
    };
    let count = digits.bytes().take_while(u8::is_ascii_digit).count();
    if count == 0 {
        return f64::NAN;
    }
    // Parsing the complete decimal digit prefix uses correctly-rounded f64
    // conversion, including large values and overflow to infinity.
    let magnitude = digits[..count]
        .parse::<f64>()
        .expect("a nonempty ASCII decimal integer is a valid f64 literal");
    if negative {
        -magnitude
    } else {
        magnitude
    }
}

fn clamp_js(value: f64, lower: f64, upper: f64) -> f64 {
    if value.is_nan() || lower.is_nan() || upper.is_nan() {
        f64::NAN
    } else {
        value.max(lower).min(upper)
    }
}

impl MouseGeometry {
    pub fn relative_coords(&self, client_x: f64, client_y: f64) -> [f64; 2] {
        [
            client_x - self.rect_left - self.padding_left,
            client_y - self.rect_top - self.padding_top,
        ]
    }

    /// One-based viewport cells. Selection admits the extra end column and
    /// shifts the horizontal threshold by half a cell, exactly as upstream.
    pub fn coords(
        &self,
        client_x: f64,
        client_y: f64,
        cols: f64,
        rows: f64,
        selection: bool,
    ) -> Option<[f64; 2]> {
        if !self.valid_char_size {
            return None;
        }
        let [x, y] = self.relative_coords(client_x, client_y);
        let offset = if selection {
            self.cell_width / 2.0
        } else {
            0.0
        };
        Some([
            clamp_js(
                ((x + offset) / self.cell_width).ceil(),
                1.0,
                cols + f64::from(u8::from(selection)),
            ),
            clamp_js((y / self.cell_height).ceil(), 1.0, rows),
        ])
    }

    /// Zero-based report cells and integer pixels, clamped to the canvas.
    pub fn report_coords(&self, client_x: f64, client_y: f64) -> Option<ReportCoords> {
        let [x, y] = self.relative_coords(client_x, client_y);
        if !self.valid_char_size {
            return None;
        }
        let x = clamp_js(x, 0.0, self.canvas_width - 1.0);
        let y = clamp_js(y, 0.0, self.canvas_height - 1.0);
        Some(ReportCoords {
            col: (x / self.cell_width).floor(),
            row: (y / self.cell_height).floor(),
            x: x.floor(),
            y: y.floor(),
        })
    }
}
