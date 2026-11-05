// Copyright (c) 2017 The xterm.js authors. All rights reserved.
// Licensed under MIT. TextureAtlas.ts pixel filtering and glyph bounds.
use crate::{api::Size, renderer::utils::js_round};
use std::{error::Error, fmt};

const CHANNELS: usize = 4;
const ALPHA: usize = 3;
// Source contrast-relative threshold, not an adjustable rendering option.
const CLEAR_THRESHOLD_DIVISOR: u32 = 12;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PixelError {
    IncompletePixel,
    ImageDimensions,
    ScanDimensions,
}
impl fmt::Display for PixelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::IncompletePixel => "RGBA data must contain complete pixels",
            Self::ImageDimensions => "RGBA byte length does not match image dimensions",
            Self::ScanDimensions => {
                "Glyph scan exceeds image dimensions or uses invalid cell geometry"
            }
        })
    }
}
impl Error for PixelError {}

fn channels(rgba: u32) -> [u8; 3] {
    [(rgba >> 24) as u8, (rgba >> 16) as u8, (rgba >> 8) as u8]
}
fn validate_pixels(pixels: &[u8]) -> Result<(), PixelError> {
    if pixels.len() % CHANNELS == 0 {
        Ok(())
    } else {
        Err(PixelError::IncompletePixel)
    }
}

/// Clear exact/near background RGB pixels without changing their RGB channels.
/// The returned source "empty" flag depends on color matches, not on preexisting
/// alpha. A nonmatching pixel with alpha zero therefore still makes it false.
pub fn clear_background(
    pixels: &mut [u8],
    bg: u32,
    fg: u32,
    threshold_check: bool,
) -> Result<bool, PixelError> {
    validate_pixels(pixels)?;
    let bg = channels(bg);
    let fg = channels(fg);
    let threshold = bg
        .iter()
        .zip(fg)
        .map(|(&a, b)| a.abs_diff(b) as u32)
        .sum::<u32>()
        / CLEAR_THRESHOLD_DIVISOR;
    let mut empty = true;
    for pixel in pixels.chunks_exact_mut(CHANNELS) {
        let exact = pixel[..ALPHA] == bg;
        let near = threshold_check
            && pixel[..ALPHA]
                .iter()
                .zip(bg)
                .map(|(&a, b)| a.abs_diff(b) as u32)
                .sum::<u32>()
                < threshold;
        if exact || near {
            pixel[ALPHA] = 0;
        } else {
            empty = false;
        }
    }
    Ok(empty)
}

pub fn completely_transparent(pixels: &[u8]) -> Result<bool, PixelError> {
    validate_pixels(pixels)?;
    Ok(pixels.chunks_exact(CHANNELS).all(|pixel| pixel[ALPHA] == 0))
}

/// Physical canvas dimensions are integral. Character dimensions may be
/// fractional; source centering uses floor horizontally and JS round vertically.
#[derive(Clone, Copy, Debug)]
pub struct GlyphScan {
    pub allowed_width: usize,
    pub padding: usize,
    pub restricted: bool,
    pub custom: bool,
    pub cell: Size,
    pub character: Size,
    pub line_height: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GlyphBounds {
    pub top: usize,
    pub left: usize,
    pub bottom: usize,
    pub right: usize,
    pub size: [f64; 2],
    pub offset: [f64; 2],
}

/// Preserve the four independent, asymmetric scans. This is deliberately not a
/// common min/max rectangle: source top/bottom omit right padding, and its
/// all-transparent fallback has inclusive width/height plus one.
pub fn find_glyph_bounds(
    pixels: &[u8],
    canvas_width: usize,
    canvas_height: usize,
    scan: GlyphScan,
) -> Result<GlyphBounds, PixelError> {
    if canvas_width
        .checked_mul(canvas_height)
        .and_then(|n| n.checked_mul(CHANNELS))
        != Some(pixels.len())
    {
        return Err(PixelError::ImageDimensions);
    }
    if ![
        scan.cell.width,
        scan.cell.height,
        scan.character.width,
        scan.character.height,
        scan.line_height,
    ]
    .iter()
    .all(|v| v.is_finite())
    {
        return Err(PixelError::ScanDimensions);
    }
    let (width, height) = if scan.restricted {
        let physical = |n: f64| {
            if n >= 0.0 && n.fract() == 0.0 && n < usize::MAX as f64 {
                Some(n as usize)
            } else {
                None
            }
        };
        (
            physical(scan.cell.width).ok_or(PixelError::ScanDimensions)?,
            physical(scan.cell.height).ok_or(PixelError::ScanDimensions)?,
        )
    } else {
        (scan.allowed_width, canvas_height)
    };
    let padded_width = width
        .checked_add(scan.padding)
        .ok_or(PixelError::ScanDimensions)?;
    if width > canvas_width || height > canvas_height {
        return Err(PixelError::ScanDimensions);
    }
    // Padded horizontal scans deliberately cross row boundaries. When the
    // canvas grew to allowedWidth, source reads past the final row as undefined;
    // undefined !== 0 is true. Keep that behavior without unchecked indexing.
    let alpha = |x: usize, y: usize| {
        y.checked_mul(canvas_width)
            .and_then(|n| n.checked_add(x))
            .and_then(|n| n.checked_mul(CHANNELS))
            .and_then(|n| n.checked_add(ALPHA))
            .and_then(|index| pixels.get(index))
            .is_none_or(|&value| value != 0)
    };
    // Empty inner scans cannot find alpha. Avoid walking index-sized empty
    // rows/columns when a zero-area image has one very large dimension.
    let vertical_rows = if width == 0 { 0 } else { height };
    let horizontal_columns = if height == 0 { 0 } else { padded_width };
    let top = (0..vertical_rows)
        .find(|&y| (0..width).any(|x| alpha(x, y)))
        .unwrap_or(0);
    let left = (0..horizontal_columns)
        .find(|&x| (0..height).any(|y| alpha(x, y)))
        .unwrap_or(0);
    let right = (scan.padding..horizontal_columns)
        .rev()
        .find(|&x| (0..height).any(|y| alpha(x, y)))
        .unwrap_or(width);
    let bottom = (0..vertical_rows)
        .rev()
        .find(|&y| (0..width).any(|x| alpha(x, y)))
        .unwrap_or(height);
    let centered = scan.restricted || scan.custom;
    let x_center = if centered {
        ((scan.cell.width - scan.character.width) / 2.0).floor()
    } else {
        0.0
    };
    let y_center = if centered && scan.line_height != 1.0 {
        // JS Math.round breaks ties toward positive infinity, unlike f64::round.
        js_round((scan.cell.height - scan.character.height) / 2.0)
    } else {
        0.0
    };
    let size = [
        right as f64 - left as f64 + 1.0,
        bottom as f64 - top as f64 + 1.0,
    ];
    Ok(GlyphBounds {
        top,
        left,
        bottom,
        right,
        size,
        offset: [
            -(left as f64) + scan.padding as f64 + x_center,
            -(top as f64) + scan.padding as f64 + y_center,
        ],
    })
}
