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
        let start = start.map(i128::from);
        let end = end.map(i128::from);
        let (x, y) = (x as i128, y as i128);
        if self.column {
            let (left, right) = if start[0] <= end[0] {
                (start[0], end[0])
            } else {
                (end[0], start[0])
            };
            return x >= left && x < right && y >= start[1] && y <= end[1];
        }
        (y > start[1] && y < end[1])
            || (start[1] == end[1] && y == start[1] && x >= start[0] && x < end[0])
            || (start[1] < end[1] && y == end[1] && x < end[0])
            || (start[1] < end[1] && y == start[1] && x >= start[0])
    }
}
#[derive(Debug)]
pub enum RowError<E> {
    Width(WidthError<E>),
    Palette(i32),
    JoinRange([usize; 2]),
    Decoration(String),
}
impl<E> From<WidthError<E>> for RowError<E> {
    fn from(error: WidthError<E>) -> Self {
        Self::Width(error)
    }
}
pub(super) struct RenderCell {
    pub attrs: AttributeData,
    pub chars: Vec<u16>,
    pub width: i32,
    pub code: Option<u32>,
}
impl From<CellData> for RenderCell {
    fn from(cell: CellData) -> Self {
        Self {
            chars: cell.get_chars(),
            width: cell.get_width(),
            code: cell.get_code(),
            attrs: cell.attributes,
        }
    }
}
