// Copyright (c) 2026 The xterm.js authors. All rights reserved.
// Licensed under MIT. Raw-pixel branch of KittyGraphicsHandler._createBitmap.

use crate::image::types::{RasterError, RgbaImage};
use std::collections::TryReserveError;
use std::fmt;

const MAX_SAFE_INTEGER: f64 = 9_007_199_254_740_991.0;
const RGBA_BYTES: usize = 4;
const RGB_BYTES: usize = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RawFormat {
    Rgb,
    Rgba,
}

#[derive(Debug)]
pub enum PixelError {
    MissingDimensions,
    InsufficientData,
    InvalidDimensions,
    Allocation(TryReserveError),
    Raster(RasterError),
}
impl fmt::Display for PixelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingDimensions => f.write_str("Width and height required for raw pixel data"),
            Self::InsufficientData => f.write_str("Insufficient pixel data"),
            Self::InvalidDimensions => {
                f.write_str("raw image dimensions do not match ImageData storage")
            }
            Self::Allocation(error) => error.fmt(f),
            Self::Raster(error) => error.fmt(f),
        }
    }
}
impl std::error::Error for PixelError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Allocation(error) => Some(error),
            Self::Raster(error) => Some(error),
            _ => None,
        }
    }
}

/// Prepare the source raw ImageData pixels after optional decompression. PNG
/// uses the host codec instead. Unknown non-PNG source formats select RGB in the
/// controller, just as the original's format32 check does.
///
/// Float metadata is not rejected merely for being fractional. The source's
/// byte-length check precedes typed-array/ImageData conversions; a small fraction
/// can still produce a valid integer raster when the truncated view length fits.
/// Browser bitmap creation and premultiplication remain platform operations.
pub fn raw_pixels(
    bytes: &[u8],
    width: f64,
    height: f64,
    format: RawFormat,
) -> Result<RgbaImage, PixelError> {
    if width == 0.0 || height == 0.0 || width.is_nan() || height.is_nan() {
        return Err(PixelError::MissingDimensions);
    }
    let pixel_count = width * height;
    let bytes_per_pixel = if format == RawFormat::Rgba {
        RGBA_BYTES
    } else {
        RGB_BYTES
    };
    if (bytes.len() as f64) < pixel_count * bytes_per_pixel as f64 {
        return Err(PixelError::InsufficientData);
    }
    let view_length = (pixel_count * RGBA_BYTES as f64).floor();
    if !width.is_finite()
        || !height.is_finite()
        || width < 1.0
        || height < 1.0
        || width.floor() > u32::MAX as f64
        || height.floor() > u32::MAX as f64
        || !view_length.is_finite()
        || !(0.0..=MAX_SAFE_INTEGER).contains(&view_length)
        || view_length > usize::MAX as f64
    {
        return Err(PixelError::InvalidDimensions);
    }
    // Valid output dimensions fit the unsigned ImageData range; oversized and
    // negative metadata was rejected above before conversion or allocation.
    let raster_width = width.floor() as u32;
    let raster_height = height.floor() as u32;
    let output_length = usize::try_from(raster_width)
        .ok()
        .and_then(|width| {
            usize::try_from(raster_height)
                .ok()
                .and_then(|height| width.checked_mul(height))
        })
        .and_then(|pixels| pixels.checked_mul(RGBA_BYTES))
        .ok_or(PixelError::InvalidDimensions)?;
    if view_length != output_length as f64 {
        return Err(PixelError::InvalidDimensions);
    }
    let mut output = Vec::new();
    output
        .try_reserve_exact(output_length)
        .map_err(PixelError::Allocation)?;
    if format == RawFormat::Rgba {
        output.extend_from_slice(
            bytes
                .get(..output_length)
                .ok_or(PixelError::InsufficientData)?,
        );
    } else {
        // Byte triples preserve the source's block/remainder pixels on every
        // endian and alignment, without the JS implementation's Uint32 aliasing.
        for pixel in bytes
            .chunks_exact(RGB_BYTES)
            .take(output_length / RGBA_BYTES)
        {
            output.extend_from_slice(pixel);
            output.push(u8::MAX);
        }
    }
    RgbaImage::new(raster_width, raster_height, output).map_err(PixelError::Raster)
}
