// Copyright (c) 2018 The xterm.js authors. All rights reserved.
// Licensed under MIT. Port of addon-webgl/src/TypedArray.ts.

//! Typed-array copies preserve JS number arguments, including the legacy
//! fallback's remainder and fractional-index behavior. Uint8ClampedArray uses
//! `u8` storage after JS has performed its input clamping. Allocation failures
//! are returned instead of panicking on terminal-controlled dimensions.

use std::collections::TryReserveError;

mod sealed {
    pub trait Numeric {}
    macro_rules! numeric {
        ($($kind:ty),*) => { $(impl Numeric for $kind {})* };
    }
    numeric!(u8, u16, u32, i8, i16, i32, f32, f64);
}

/// Numeric storage supported by the nine upstream typed-array constructors.
pub trait TypedArrayElement: Copy + sealed::Numeric {
    /// Assignment of JavaScript `undefined` to this typed-array element.
    fn undefined() -> Self;
}
macro_rules! integer {
    ($($kind:ty),*) => { $(impl TypedArrayElement for $kind {
        fn undefined() -> Self { 0 }
    })* };
}
integer!(u8, u16, u32, i8, i16, i32);
impl TypedArrayElement for f32 {
    fn undefined() -> Self {
        Self::NAN
    }
}
impl TypedArrayElement for f64 {
    fn undefined() -> Self {
        Self::NAN
    }
}

/// Modern TypedArray.prototype.slice over existing numeric storage.
/// `None` represents an omitted or undefined argument, not JavaScript null.
pub fn slice<T: TypedArrayElement>(
    array: &[T],
    start: Option<f64>,
    end: Option<f64>,
) -> Result<Vec<T>, TryReserveError> {
    let start = relative_index(start.unwrap_or(0.0), array.len());
    let end = relative_index(end.unwrap_or(array.len() as f64), array.len()).max(start);
    copy(&array[start..end])
}

/// The exact fallback retained by upstream for environments without `.slice`.
/// This intentionally differs from the modern operation on fractional indexes
/// and values below `-length`; do not replace JS remainder with Euclidean mod.
pub fn slice_fallback<T: TypedArrayElement>(
    array: &[T],
    start: Option<f64>,
    end: Option<f64>,
) -> Result<Vec<T>, TryReserveError> {
    let length = array.len() as f64;
    let mut start = start.unwrap_or(0.0);
    let mut end = end.unwrap_or(length);
    if start < 0.0 {
        start = (length + start) % length;
    }
    if end >= length {
        end = length;
    } else {
        end = (length + end) % length;
    }
    // Rust's f64::min ignores NaN, unlike JavaScript Math.min.
    start = if start.is_nan() || end.is_nan() {
        f64::NAN
    } else {
        start.min(end)
    };
    let span = end - start;
    // The source's Math.min ensures a nonnegative or NaN span. Rust's
    // saturating cast has the same NaN-to-zero/truncation here; reserve checks
    // the resulting native capacity before allocating. Indexing still uses get.
    let result_length = span as usize;
    let mut result = Vec::new();
    result.try_reserve_exact(result_length)?;
    for index in 0..result_length {
        let source = start + index as f64;
        let value = if source.is_finite() && source >= 0.0 && source.fract() == 0.0 {
            array.get(source as usize).copied()
        } else {
            None
        };
        result.push(value.unwrap_or_else(T::undefined));
    }
    Ok(result)
}

fn copy<T: Copy>(array: &[T]) -> Result<Vec<T>, TryReserveError> {
    let mut result = Vec::new();
    result.try_reserve_exact(array.len())?;
    result.extend_from_slice(array);
    Ok(result)
}

fn relative_index(value: f64, length: usize) -> usize {
    let integer = if value.is_nan() { 0.0 } else { value.trunc() };
    let length_number = length as f64;
    let index = if integer < 0.0 {
        (length_number + integer).max(0.0)
    } else {
        integer.min(length_number)
    };
    // Numeric conversion is saturating; the second clamp also covers f64
    // rounding on hypothetical slices longer than exactly representable ints.
    (index as usize).min(length)
}
