// Copyright (c) 2017-2018 The xterm.js authors. All rights reserved.
// Licensed under MIT. Portable WebglUtils.ts and Constants.ts operations.

use std::{collections::TryReserveError, error::Error, fmt};

/// Translate normalized top-left coordinates into WebGL clip coordinates.
pub const PROJECTION_MATRIX: [f32; 16] = [
    2.0, 0.0, 0.0, 0.0, 0.0, -2.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, -1.0, 1.0, 0.0, 1.0,
];
pub const DIM_OPACITY: f64 = 0.5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextBaseline {
    Bottom,
    Ideographic,
}
impl TextBaseline {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Bottom => "bottom",
            Self::Ideographic => "ideographic",
        }
    }
}
/// Source uses bottom only for Firefox and legacy Edge to avoid truncation.
pub fn text_baseline(is_firefox: bool, is_legacy_edge: bool) -> TextBaseline {
    if is_firefox || is_legacy_edge {
        TextBaseline::Bottom
    } else {
        TextBaseline::Ideographic
    }
}

#[derive(Debug)]
pub enum FloatArrayError {
    InvalidLength(f64),
    Allocation(TryReserveError),
}
impl fmt::Display for FloatArrayError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLength(length) => {
                write!(formatter, "invalid typed array length: {length}")
            }
            Self::Allocation(error) => error.fmt(formatter),
        }
    }
}
impl Error for FloatArrayError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Allocation(error) => Some(error),
            Self::InvalidLength(_) => None,
        }
    }
}
impl From<TryReserveError> for FloatArrayError {
    fn from(error: TryReserveError) -> Self {
        Self::Allocation(error)
    }
}

/// Double capacity up to max, preserving source's Float32Array construction.
/// A max smaller than the source truncates data; new elements are positive zero.
pub fn expand_float32_array(source: &[f32], max: f64) -> Result<Vec<f32>, FloatArrayError> {
    let doubled = source.len() as f64 * 2.0;
    let length = if max.is_nan() {
        f64::NAN
    } else {
        doubled.min(max)
    };
    let integer = if length.is_nan() { 0.0 } else { length.trunc() };
    if integer < 0.0 || !integer.is_finite() {
        return Err(FloatArrayError::InvalidLength(length));
    }
    // The length is at most twice an existing numeric slice's length. Native
    // capacity is checked by reserve; no untrusted allocation may panic here.
    let length = integer as usize;
    let mut result = Vec::new();
    result.try_reserve_exact(length)?;
    result.resize(length, 0.0);
    let count = source.len().min(length);
    result[..count].copy_from_slice(&source[..count]);
    Ok(result)
}

/// CPU-side texture/version bookkeeping. Actual texture ownership stays with
/// the renderer; the version starts at -1 so a fresh texture always uploads.
pub struct GlTexture<T> {
    pub texture: T,
    pub version: f64,
}
impl<T> GlTexture<T> {
    pub fn new(texture: T) -> Self {
        Self {
            texture,
            version: -1.0,
        }
    }
}
