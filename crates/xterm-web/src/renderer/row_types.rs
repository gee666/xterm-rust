// Copyright (c) 2018, 2023 The xterm.js authors. All rights reserved. MIT license.
use super::width_cache::WidthError;
use xterm_core::{
    buffer::{AttributeData, CellData},
    color::Color,
};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RowSpan {
    pub text: Vec<u16>,
    pub classes: Vec<String>,
    pub styles: Vec<(String, String)>,
}
impl RowSpan {
    pub fn style(&mut self, property: &str, value: String) {
        if let Some((_, old)) = self.styles.iter_mut().find(|(key, _)| key == property) {
            *old = value;
        } else {
            self.styles.push((property.into(), value));
        }
    }
}
#[derive(Clone, Debug, Default)]
pub struct RowDecoration {
    pub foreground: Option<Color>,
    pub background: Option<Color>,
    pub top: bool,
}
/// Preserve a host exception through row construction without stringifying it.
pub type RowDecorationResult = Result<Vec<RowDecoration>, std::rc::Rc<dyn std::error::Error>>;
#[derive(Clone, Debug)]
pub struct RowOptions {
    pub row: usize,
    pub cursor_row: bool,
    pub cursor_x: usize,
    pub cursor_style: String,
    pub cursor_inactive_style: String,
    pub cursor_blink: bool,
    pub cursor_hidden: bool,
    pub cursor_initialized: bool,
    pub focused: bool,
    pub blink_on: bool,
    pub cell_width: f64,
    pub default_spacing: f64,
    pub link_range: Option<[i64; 2]>,
    pub bold_bright: bool,
    pub minimum_contrast_ratio: f64,
}
impl Default for RowOptions {
    fn default() -> Self {
        Self {
            row: 0,
            cursor_row: false,
            cursor_x: 0,
            cursor_style: "block".into(),
            cursor_inactive_style: "outline".into(),
            cursor_blink: false,
            cursor_hidden: false,
            cursor_initialized: true,
            focused: true,
            blink_on: true,
            cell_width: 10.0,
            default_spacing: 0.0,
            link_range: None,
            bold_bright: true,
            minimum_contrast_ratio: 1.0,
        }
    }
}
#[derive(Clone, Copy, Debug, Default)]
pub struct Selection {
    pub start: Option<[i64; 2]>,
    pub end: Option<[i64; 2]>,
    pub column: bool,
}
impl Selection {
    pub fn contains(&self, x: usize, y: usize) -> bool {
        let (Some(start), Some(end)) = (self.start, self.end) else {
            return false;
        };
        contains_in_range(
            x as i128,
            y as i128,
            [i128::from(start[0]), i128::from(end[0])],
            [i128::from(start[1]), i128::from(end[1])],
            self.column,
        )
    }
    /// Fractional browser columns must not be rounded into a native cell index.
    pub fn contains_number(&self, x: f64, y: usize) -> bool {
        let (Some(start), Some(end)) = (self.start, self.end) else {
            return false;
        };
        contains_in_range(
            x,
            y as i128,
            [start[0] as f64, end[0] as f64],
            [i128::from(start[1]), i128::from(end[1])],
            self.column,
        )
    }
}
fn contains_in_range<T: Copy + PartialOrd>(
    x: T,
    y: i128,
    horizontal: [T; 2],
    vertical: [i128; 2],
    column: bool,
) -> bool {
    let [sx, ex] = horizontal;
    let [sy, ey] = vertical;
    if column {
        let (left, right) = if sx <= ex { (sx, ex) } else { (ex, sx) };
        return x >= left && x < right && y >= sy && y <= ey;
    }
    (y > sy && y < ey)
        || (sy == ey && y == sy && x >= sx && x < ex)
        || (sy < ey && y == ey && x < ex)
        || (sy < ey && y == sy && x >= sx)
}
#[derive(Debug)]
pub enum RowError<E> {
    Width(WidthError<E>),
    Palette(i32),
    JoinRange([usize; 2]),
    NonProgressingColumn(f64),
    Decoration(std::rc::Rc<dyn std::error::Error>),
    #[cfg(target_arch = "wasm32")]
    Browser(wasm_bindgen::JsValue),
}
impl<E> From<WidthError<E>> for RowError<E> {
    fn from(error: WidthError<E>) -> Self {
        Self::Width(error)
    }
}
pub(super) struct RenderCell {
    pub attrs: AttributeData,
    pub chars: Vec<u16>,
    pub width: f64,
    #[cfg(target_arch = "wasm32")]
    pub raw_width: Option<wasm_bindgen::JsValue>,
    pub code: Option<u32>,
}
impl RenderCell {
    pub fn pixel_width<E>(&self, cell_width: f64) -> Result<f64, RowError<E>> {
        #[cfg(target_arch = "wasm32")]
        if let Some(width) = &self.raw_width {
            return super::row_operators::multiply(width, cell_width).map_err(RowError::Browser);
        }
        Ok(self.width * cell_width)
    }
    pub fn join(&mut self, chars: Vec<u16>, width: f64) {
        self.chars = chars;
        self.width = width;
        self.code = Some(0x1fffff);
        self.attrs = AttributeData {
            fg: self.attrs.fg,
            bg: self.attrs.bg,
            ..AttributeData::default()
        };
    }
}
impl From<CellData> for RenderCell {
    fn from(cell: CellData) -> Self {
        Self {
            chars: cell.get_chars(),
            width: f64::from(cell.get_width()),
            #[cfg(target_arch = "wasm32")]
            raw_width: None,
            code: cell.get_code(),
            attrs: cell.attributes,
        }
    }
}
