// Copyright (c) 2017 The xterm.js authors. All rights reserved.
// Licensed under MIT. Port of addon-webgl/src/CharAtlasUtils.ts.

use crate::theme::{ColorRef, ColorSet};
use std::{error::Error, fmt, rc::Rc};
use xterm_core::color::Color;

#[derive(Clone, Debug, PartialEq)]
pub enum FontWeight {
    Text(Rc<[u16]>),
    Number(f64),
}

/// Values already validated by the terminal options service. UTF-16 strings
/// stay shared and immutable, like the source's primitive JavaScript strings.
#[derive(Clone, Debug)]
pub struct AtlasOptions {
    pub letter_spacing: f64,
    pub line_height: f64,
    pub font_family: Rc<[u16]>,
    pub font_size: f64,
    pub font_weight: FontWeight,
    pub font_weight_bold: FontWeight,
    pub allow_transparency: bool,
    pub draw_bold_text_in_bright_colors: bool,
    pub minimum_contrast_ratio: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AtlasDimensions {
    pub device_pixel_ratio: f64,
    pub device_max_texture_size: f64,
    pub device_cell_width: f64,
    pub device_cell_height: f64,
    pub device_char_width: f64,
    pub device_char_height: f64,
}

#[derive(Clone, Debug)]
pub struct AtlasConfig {
    pub custom_glyphs: bool,
    pub dimensions: AtlasDimensions,
    pub options: AtlasOptions,
    pub colors: ColorSet,
}

/// Scope the source module's shared NULL_COLOR identity to an explicitly owned
/// runtime. Configs from this factory share null/color/cache objects, but each
/// owns an independent ANSI vector. No portable mutable global is required.
pub struct ConfigFactory {
    null_color: ColorRef,
}
impl Default for ConfigFactory {
    fn default() -> Self {
        Self {
            null_color: Color::null().into(),
        }
    }
}
impl ConfigFactory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn generate(
        &self,
        dimensions: AtlasDimensions,
        options: &AtlasOptions,
        colors: &ColorSet,
        custom_glyphs: bool,
    ) -> AtlasConfig {
        let null = &self.null_color;
        let mut colors = colors.clone();
        colors.cursor = null.clone();
        colors.cursor_accent = null.clone();
        colors.selection_foreground = Some(null.clone());
        colors.selection_background_transparent = null.clone();
        colors.selection_background_opaque = null.clone();
        colors.selection_inactive_background_transparent = null.clone();
        colors.selection_inactive_background_opaque = null.clone();
        colors.overview_ruler_border = null.clone();
        colors.scrollbar_slider_background = null.clone();
        colors.scrollbar_slider_hover_background = null.clone();
        colors.scrollbar_slider_active_background = null.clone();
        AtlasConfig {
            custom_glyphs,
            dimensions,
            options: options.clone(),
            colors,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MissingAnsiColor {
    pub index: usize,
}
impl fmt::Display for MissingAnsiColor {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "atlas config comparison needs ANSI color at index {}",
            self.index
        )
    }
}
impl Error for MissingAnsiColor {}

/// Source compares ANSI entries only over the left array's length. A longer
/// right array is deliberately ignored; a shorter right array throws upstream.
/// CSS strings, caches and nulled colors do not participate in equality.
pub fn config_equals(a: &AtlasConfig, b: &AtlasConfig) -> Result<bool, MissingAnsiColor> {
    for (index, color) in a.colors.ansi.iter().enumerate() {
        let other = b.colors.ansi.get(index).ok_or(MissingAnsiColor { index })?;
        if color.borrow().rgba != other.borrow().rgba {
            return Ok(false);
        }
    }
    Ok(a.custom_glyphs == b.custom_glyphs
        && a.dimensions == b.dimensions
        && a.options.letter_spacing == b.options.letter_spacing
        && a.options.line_height == b.options.line_height
        && a.options.font_family == b.options.font_family
        && a.options.font_size == b.options.font_size
        && a.options.font_weight == b.options.font_weight
        && a.options.font_weight_bold == b.options.font_weight_bold
        && a.options.allow_transparency == b.options.allow_transparency
        && a.options.draw_bold_text_in_bright_colors == b.options.draw_bold_text_in_bright_colors
        && a.options.minimum_contrast_ratio == b.options.minimum_contrast_ratio
        && a.colors.foreground.borrow().rgba == b.colors.foreground.borrow().rgba
        && a.colors.background.borrow().rgba == b.colors.background.borrow().rgba)
}

/// Packed color modes are shared with the core attribute representation.
pub fn is_256_color(color_code: u32) -> bool {
    use xterm_core::buffer::constants::attributes;
    matches!(
        color_code & attributes::CM_MASK,
        attributes::CM_P16 | attributes::CM_P256
    )
}
