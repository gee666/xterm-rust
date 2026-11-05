// Copyright (c) 2021 The xterm.js authors. All rights reserved. MIT license.
//! Source customGlyphs geometry and immediate canvas operations.
use std::borrow::Cow;

#[derive(Clone, Copy, Debug)]
pub struct CustomGlyphDrawParams<'a> {
    pub x_offset: f64,
    pub y_offset: f64,
    pub device_cell_width: f64,
    pub device_cell_height: f64,
    pub device_char_width: f64,
    pub device_char_height: f64,
    pub font_size: f64,
    pub device_pixel_ratio: f64,
    pub background_color: Option<&'a str>,
    pub variant_offset: i32,
}

#[derive(Clone, Copy)]
pub(super) struct Octant {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}
#[derive(Clone, Copy)]
pub(super) enum Path {
    Static(&'static str),
    Dynamic(fn(f64, f64) -> String),
}
impl Path {
    pub fn instructions(self, xp: f64, yp: f64) -> Cow<'static, str> {
        match self {
            Self::Static(path) => Cow::Borrowed(path),
            Self::Dynamic(path) => Cow::Owned(path(xp, yp)),
        }
    }
}
#[derive(Clone, Copy)]
pub(super) struct Vector {
    pub path: &'static str,
    pub stroke: bool,
    pub left: f64,
    pub right: f64,
}
#[derive(Clone, Copy)]
pub(super) enum Definition {
    Blocks(&'static [Octant]),
    Pattern(u16, &'static [&'static [u8]]),
    PathFunction(Path),
    Path(&'static str),
    Negative(Vector),
    Vector(Vector),
    Braille(u8),
}
#[derive(Clone, Copy)]
pub(super) struct Part {
    pub data: Definition,
    pub scale_char: bool,
    pub clip: Option<&'static str>,
    pub stroke: Option<f64>,
}

/// Commands are executed immediately, so a failed canvas operation stops drawing
/// at the same point as the source, without restoring state after a throw.
#[derive(Clone, Debug, PartialEq)]
pub enum DrawCommand {
    Save,
    Restore,
    Begin,
    Close,
    Move([f64; 2]),
    Line([f64; 2]),
    Quadratic([f64; 4]),
    Cubic([f64; 6]),
    Arc([f64; 5]),
    Ellipse([f64; 7], bool),
    Fill,
    Stroke,
    Rect([f64; 4]),
    FillRect([f64; 4]),
    Clip,
    ClipRect([f64; 4]),
    StrokeFromFill,
    Background(String),
    LineWidth(f64),
    SquareCap,
    Pattern(u16, &'static [&'static [u8]], i32),
    LogError(String),
}

/// The browser executes real Canvas2D operations; native tests record operations.
/// This keeps path arithmetic portable without allocating a command list per glyph.
pub trait GlyphCanvas {
    type Error;
    fn draw(&mut self, command: DrawCommand) -> Result<(), Self::Error>;
}

#[derive(Clone, Copy)]
pub(super) struct Area {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}
impl Area {
    pub fn rect(self) -> [f64; 4] {
        [self.x, self.y, self.width, self.height]
    }
}
