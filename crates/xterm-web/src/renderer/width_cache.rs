// Copyright (c) 2023 The xterm.js authors. All rights reserved.
// Licensed under MIT. Ported from browser/renderer/dom/WidthCache.ts.
use std::{collections::HashMap, fmt};

pub const FLAT_UNSET: f32 = -9999.0;
pub const FLAT_SIZE: usize = 256;

#[derive(Clone, Debug, PartialEq)]
pub struct Font {
    pub family: String,
    pub size: f64,
    pub weight: String,
    pub weight_bold: String,
}
impl Default for Font {
    fn default() -> Self {
        Self {
            family: String::new(),
            size: 0.0,
            weight: "normal".into(),
            weight_bold: "bold".into(),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum FontVariant {
    Regular,
    Bold,
    Italic,
    BoldItalic,
}

/// Implementations own four independent font variants and propagate host errors.
pub trait WidthMeasurer {
    type Error;
    fn configure(&mut self, font: &Font) -> Result<(), Self::Error>;
    fn measure(&mut self, text: &[u16], variant: FontVariant) -> Result<f64, Self::Error>;
}
#[derive(Debug, PartialEq)]
pub enum WidthError<E> {
    Disposed,
    Measure(E),
}
impl<E: fmt::Display> fmt::Display for WidthError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Disposed => f.write_str("width measurement canvas has been disposed"),
            Self::Measure(error) => error.fmt(f),
        }
    }
}
impl<E: fmt::Debug + fmt::Display> std::error::Error for WidthError<E> {}

pub struct WidthCache<M: WidthMeasurer> {
    flat: [f32; FLAT_SIZE],
    holey: Option<HashMap<Vec<u16>, f64>>,
    font: Font,
    measurer: Option<M>,
}
impl<M: WidthMeasurer> WidthCache<M> {
    pub fn new(measurer: M) -> Self {
        Self {
            flat: [FLAT_UNSET; FLAT_SIZE],
            holey: Some(HashMap::new()),
            font: Font::default(),
            measurer: Some(measurer),
        }
    }
    pub fn dispose(&mut self) {
        self.measurer = None;
        self.holey = None;
    }
    pub fn clear(&mut self) {
        self.flat.fill(FLAT_UNSET);
        self.holey = Some(HashMap::new());
    }
    pub fn set_font(&mut self, font: Font) -> Result<(), WidthError<M::Error>> {
        if self.font == font {
            return Ok(());
        }
        self.font = font;
        self.measurer
            .as_mut()
            .ok_or(WidthError::Disposed)?
            .configure(&self.font)
            .map_err(WidthError::Measure)?;
        self.clear();
        Ok(())
    }
    pub fn get(
        &mut self,
        text: &[u16],
        bold: bool,
        italic: bool,
    ) -> Result<f64, WidthError<M::Error>> {
        if !bold && !italic && text.len() == 1 && usize::from(text[0]) < FLAT_SIZE {
            let index = usize::from(text[0]);
            if self.flat[index] != FLAT_UNSET {
                return Ok(f64::from(self.flat[index]));
            }
            let width = self.measure(text, FontVariant::Regular)?;
            if width > 0.0 {
                self.flat[index] = width as f32;
            }
            return Ok(width);
        }
        // Keep the source key encoding, including its possible suffix collisions.
        let mut key = text.to_vec();
        if bold {
            key.push(u16::from(b'B'));
        }
        if italic {
            key.push(u16::from(b'I'));
        }
        if let Some(width) = self.holey.as_ref().ok_or(WidthError::Disposed)?.get(&key) {
            return Ok(*width);
        }
        let variant = match (bold, italic) {
            (false, false) => FontVariant::Regular,
            (true, false) => FontVariant::Bold,
            (false, true) => FontVariant::Italic,
            (true, true) => FontVariant::BoldItalic,
        };
        let width = self.measure(text, variant)?;
        if width > 0.0 {
            self.holey
                .as_mut()
                .ok_or(WidthError::Disposed)?
                .insert(key, width);
        }
        Ok(width)
    }
    fn measure(&mut self, text: &[u16], variant: FontVariant) -> Result<f64, WidthError<M::Error>> {
        self.measurer
            .as_mut()
            .ok_or(WidthError::Disposed)?
            .measure(text, variant)
            .map_err(WidthError::Measure)
    }
}
#[cfg(test)]
#[path = "width_cache_tests.rs"]
mod tests;
