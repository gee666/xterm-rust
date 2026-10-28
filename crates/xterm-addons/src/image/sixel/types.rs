// Copyright (c) 2021 Joerg Breitbart. MIT license.
use super::super::types::RasterError;
use super::colors;
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum SixelMode {
    Undecided = 0,
    Dynamic = 1,
    Truncated = 2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum SixelLevel {
    Undecided = 0,
    One = 1,
    Two = 2,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RasterAttributes {
    pub numerator: u32,
    pub denominator: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Debug)]
pub struct DecoderOptions {
    /// None and numeric zero disable the original pixel allocation limit.
    /// Fractions, negative numbers, NaN and infinity retain JS comparisons.
    pub memory_limit: Option<f64>,
    pub foreground: u32,
    pub fill: u32,
    pub palette: Vec<u32>,
    pub palette_limit: f64,
    pub truncate: bool,
}

impl Default for DecoderOptions {
    fn default() -> Self {
        Self {
            memory_limit: Some(134_217_728.0),
            foreground: u32::MAX,
            fill: 0xff00_0000,
            palette: colors::VT340.to_vec(),
            palette_limit: 4096.0,
            truncate: true,
        }
    }
}

pub(super) fn js_uint32(value: f64) -> u32 {
    if !value.is_finite() || value == 0.0 {
        return 0;
    }
    value.trunc().rem_euclid(4_294_967_296.0) as u32
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecodeError {
    PaletteTooLarge,
    MemoryLimit,
    Allocation,
    InvalidDimensions,
    InvalidColor,
    PaletteZero,
    MemoryAccess,
    Arithmetic,
    Raster(RasterError),
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::PaletteTooLarge => "DecoderOptions.paletteLimit must not exceed 4096",
            Self::MemoryLimit => "image exceeds memory limit",
            Self::Allocation => "cannot allocate SIXEL pixel storage",
            Self::InvalidDimensions => "invalid SIXEL raster dimensions",
            Self::InvalidColor => "table index is out of bounds",
            Self::PaletteZero => "remainder by zero",
            Self::MemoryAccess => "memory access out of bounds",
            Self::Arithmetic => "SIXEL command exceeds addressable pixel storage",
            Self::Raster(error) => return error.fmt(f),
        })
    }
}
impl DecodeError {
    pub(super) fn js_name(self) -> &'static str {
        match self {
            Self::InvalidColor | Self::PaletteZero | Self::MemoryAccess => "RuntimeError",
            Self::Allocation | Self::InvalidDimensions | Self::Arithmetic | Self::Raster(_) => {
                "RangeError"
            }
            Self::PaletteTooLarge | Self::MemoryLimit => "Error",
        }
    }
}

impl std::error::Error for DecodeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Raster(error) => Some(error),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecoderProperties {
    pub width: i64,
    pub height: i64,
    pub mode: SixelMode,
    pub level: SixelLevel,
    pub truncate: bool,
    pub palette_limit: u32,
    pub fill_color: u32,
    pub memory_usage: usize,
    pub raster: RasterAttributes,
}
