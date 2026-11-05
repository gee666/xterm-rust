// Copyright (c) 2017 The xterm.js authors. All rights reserved.
// Licensed under MIT. TextureAtlas.ts color resolution and contrast caching.
use super::config::{AtlasConfig, ConfigFactory};
use crate::theme::ColorRef;
use std::{error::Error, fmt, rc::Rc};
use xterm_core::{
    buffer::{constants::attributes, AttributeData},
    color::{channels, rgba, value as color},
};

const DIM_OPACITY: f64 = 0.5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AtlasColorError {
    pub index: i32,
}
impl fmt::Display for AtlasColorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "No color found for idx {}", self.index)
    }
}
impl Error for AtlasColorError {}

pub struct GlyphColors {
    pub background: ColorRef,
    pub foreground: ColorRef,
}
#[derive(Clone, Copy)]
struct Paint {
    mode: u32,
    value: i32,
}
#[derive(Clone, Copy)]
pub struct GlyphPaint {
    cache_bg: u32,
    cache_fg: u32,
    bg: Paint,
    fg: Paint,
    inverse: bool,
    bold: bool,
    dim: bool,
}

/// Colors stay shared with the config and its contrast caches. Resolve references
/// here, but release RefCell guards before calling any canvas/user code.
pub struct AtlasColors {
    config: Rc<AtlasConfig>,
    null: ColorRef,
}
impl AtlasColors {
    pub fn new(config: Rc<AtlasConfig>, factory: &ConfigFactory) -> Self {
        Self {
            config,
            null: factory.null_color(),
        }
    }
    pub fn resolve(
        &self,
        attrs: &AttributeData,
        exclude_contrast: bool,
    ) -> Result<GlyphColors, AtlasColorError> {
        let paint = Self::paint(attrs);
        Ok(GlyphColors {
            background: self.background_for(paint)?,
            foreground: self.foreground_for(paint, exclude_contrast)?,
        })
    }
    /// Capture source local attribute values before any canvas callback. Color
    /// and cache lookups themselves remain deferred to their drawing phase.
    pub fn paint(attrs: &AttributeData) -> GlyphPaint {
        let inverse = attrs.is_inverse() != 0;
        let bold = attrs.is_bold() != 0;
        let dim = attrs.is_dim() != 0;
        let mut bg = Paint {
            mode: attrs.get_bg_color_mode(),
            value: attrs.get_bg_color(),
        };
        let mut fg = Paint {
            mode: attrs.get_fg_color_mode(),
            value: attrs.get_fg_color(),
        };
        if inverse {
            std::mem::swap(&mut bg, &mut fg);
        }
        GlyphPaint {
            cache_bg: attrs.bg,
            cache_fg: attrs.fg,
            bg,
            fg,
            inverse,
            bold,
            dim,
        }
    }
    pub fn background_for(&self, paint: GlyphPaint) -> Result<ColorRef, AtlasColorError> {
        self.background(paint.bg, paint.inverse)
    }
    pub fn foreground_for(
        &self,
        paint: GlyphPaint,
        exclude_contrast: bool,
    ) -> Result<ColorRef, AtlasColorError> {
        let GlyphPaint {
            fg,
            inverse,
            bold,
            dim,
            ..
        } = paint;
        Ok(
            if let Some(adjusted) = self.contrast(paint, exclude_contrast)? {
                // Source returns adjusted contrast directly, without applying dim
                // opacity afterward. Dim instead halves the requested contrast.
                adjusted
            } else {
                let mut result = self.foreground(fg, inverse, bold)?;
                if self.config.options.allow_transparency {
                    let opaque = color::opaque(&result.borrow());
                    result = opaque.into();
                }
                if dim {
                    let dimmed = color::multiply_opacity(&result.borrow(), DIM_OPACITY);
                    result = dimmed.into();
                }
                result
            },
        )
    }
    pub fn palette(&self, index: i32) -> Result<ColorRef, AtlasColorError> {
        let index_native = usize::try_from(index).map_err(|_| AtlasColorError { index })?;
        self.config
            .colors
            .ansi
            .get(index_native)
            .cloned()
            .ok_or(AtlasColorError { index })
    }
    fn background(&self, paint: Paint, inverse: bool) -> Result<ColorRef, AtlasColorError> {
        if self.config.options.allow_transparency {
            return Ok(self.null.clone());
        }
        let selected = match paint.mode {
            attributes::CM_P16 | attributes::CM_P256 => self.palette(paint.value)?,
            attributes::CM_RGB => rgb(paint.value),
            _ => {
                if inverse {
                    self.config.colors.foreground.clone()
                } else {
                    self.config.colors.background.clone()
                }
            }
        };
        let opaque = color::opaque(&selected.borrow());
        Ok(opaque.into())
    }
    fn foreground(
        &self,
        paint: Paint,
        inverse: bool,
        bold: bool,
    ) -> Result<ColorRef, AtlasColorError> {
        Ok(match paint.mode {
            attributes::CM_P16 | attributes::CM_P256 => {
                self.palette(self.bright_index(paint.value, bold))?
            }
            attributes::CM_RGB => rgb(paint.value),
            _ => {
                if inverse {
                    self.config.colors.background.clone()
                } else {
                    self.config.colors.foreground.clone()
                }
            }
        })
    }
    fn bright_index(&self, index: i32, bold: bool) -> i32 {
        if self.config.options.draw_bold_text_in_bright_colors && bold && index < 8 {
            index + 8
        } else {
            index
        }
    }
    fn background_rgba(&self, paint: Paint, inverse: bool) -> Result<u32, AtlasColorError> {
        Ok(match paint.mode {
            attributes::CM_P16 | attributes::CM_P256 => self.palette(paint.value)?.borrow().rgba,
            attributes::CM_RGB => (paint.value as u32).wrapping_shl(8),
            _ => {
                if inverse {
                    self.config.colors.foreground.borrow().rgba
                } else {
                    self.config.colors.background.borrow().rgba
                }
            }
        })
    }
    fn foreground_rgba(
        &self,
        paint: Paint,
        inverse: bool,
        bold: bool,
    ) -> Result<u32, AtlasColorError> {
        Ok(match paint.mode {
            attributes::CM_P16 | attributes::CM_P256 => {
                self.palette(self.bright_index(paint.value, bold))?
                    .borrow()
                    .rgba
            }
            attributes::CM_RGB => (paint.value as u32).wrapping_shl(8),
            _ => {
                if inverse {
                    self.config.colors.background.borrow().rgba
                } else {
                    self.config.colors.foreground.borrow().rgba
                }
            }
        })
    }
    fn contrast(
        &self,
        state: GlyphPaint,
        excluded: bool,
    ) -> Result<Option<ColorRef>, AtlasColorError> {
        let GlyphPaint {
            cache_bg,
            cache_fg,
            bg,
            fg,
            inverse,
            bold,
            dim,
        } = state;
        let ratio = self.config.options.minimum_contrast_ratio;
        if ratio == 1.0 || excluded {
            return Ok(None);
        }
        let cache = if dim {
            &self.config.colors.half_contrast_cache
        } else {
            &self.config.colors.contrast_cache
        };
        if let Some(value) = cache.borrow().get_color(cache_bg, cache_fg).cloned() {
            return Ok(value);
        }
        let adjusted = rgba::ensure_contrast_ratio(
            self.background_rgba(bg, inverse)?,
            self.foreground_rgba(fg, inverse, bold)?,
            ratio / if dim { 2.0 } else { 1.0 },
        );
        let result: Option<ColorRef> = adjusted.map(|rgba| rgb((rgba >> 8) as i32));
        cache
            .borrow_mut()
            .set_color(cache_bg, cache_fg, result.clone());
        Ok(result)
    }
}
fn rgb(value: i32) -> ColorRef {
    let [r, g, b] = AttributeData::to_color_rgb(value as u32);
    channels::to_color(r.into(), g.into(), b.into(), None).into()
}
