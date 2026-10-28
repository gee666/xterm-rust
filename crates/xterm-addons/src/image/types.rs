// Copyright (c) 2020-2026 The xterm.js authors. MIT license.
//! Owned decoded pixels shared by native image protocols and browser adapters.
use std::fmt;

pub const RGBA_CHANNELS: usize = 4;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RgbaImage {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RasterError {
    DimensionsOverflow,
    PixelLength { expected: usize, actual: usize },
}
impl fmt::Display for RasterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DimensionsOverflow => {
                f.write_str("image dimensions exceed addressable RGBA storage")
            }
            Self::PixelLength { expected, actual } => {
                write!(f, "expected {expected} RGBA bytes, received {actual}")
            }
        }
    }
}
impl std::error::Error for RasterError {}

impl RgbaImage {
    /// Empty images remain representable for SIXEL's cursor-only raster outcome.
    pub fn new(width: u32, height: u32, pixels: Vec<u8>) -> Result<Self, RasterError> {
        let width_usize = usize::try_from(width).map_err(|_| RasterError::DimensionsOverflow)?;
        let height_usize = usize::try_from(height).map_err(|_| RasterError::DimensionsOverflow)?;
        let expected = width_usize
            .checked_mul(height_usize)
            .and_then(|n| n.checked_mul(RGBA_CHANNELS))
            .ok_or(RasterError::DimensionsOverflow)?;
        if pixels.len() != expected {
            return Err(RasterError::PixelLength {
                expected,
                actual: pixels.len(),
            });
        }
        Ok(Self {
            width,
            height,
            pixels,
        })
    }
    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }
    pub fn into_pixels(self) -> Vec<u8> {
        self.pixels
    }
}
