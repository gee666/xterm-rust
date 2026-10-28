// Copyright (c) 2020, 2023 The xterm.js authors. MIT license.
use std::fmt;
use xterm_core::buffer::AttributeData;

/// Theme colors retain xterm's packed big-endian RGBA number representation.
#[derive(Clone, Copy, Debug)]
pub struct ActiveColors<'a> {
    pub foreground: u32,
    pub background: u32,
    pub ansi: &'a [u32],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BackgroundError {
    pub palette_index: i32,
}
impl fmt::Display for BackgroundError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "missing SIXEL background palette index {}",
            self.palette_index
        )
    }
}
impl std::error::Error for BackgroundError {}

/// Returns an ABGR word. Bytes are serialized explicitly by the decoder, so
/// this conversion does not depend on the native CPU's byte order.
pub fn active_background(
    attr: &AttributeData,
    colors: Option<ActiveColors<'_>>,
) -> Result<u32, BackgroundError> {
    // The original returns transparent black before checking RGB attributes
    // when the theme service is absent, for example before terminal.open().
    let Some(colors) = colors else {
        return Ok(0);
    };
    let inverse = attr.is_inverse() != 0;
    let (default, rgb, value) = if inverse {
        (attr.is_fg_default(), attr.is_fg_rgb(), attr.get_fg_color())
    } else {
        (attr.is_bg_default(), attr.is_bg_rgb(), attr.get_bg_color())
    };
    if default {
        let rgba = if inverse {
            colors.foreground
        } else {
            colors.background
        };
        return Ok(rgba.swap_bytes());
    }
    if rgb {
        // RGB-mode getters mask to 24 bits, so this integer is nonnegative.
        let [r, g, b] = AttributeData::to_color_rgb(value as u32);
        return Ok(u32::from_le_bytes([r, g, b, 255]));
    }
    let index = usize::try_from(value).map_err(|_| BackgroundError {
        palette_index: value,
    })?;
    colors
        .ansi
        .get(index)
        .map(|rgba| rgba.swap_bytes())
        .ok_or(BackgroundError {
            palette_index: value,
        })
}
