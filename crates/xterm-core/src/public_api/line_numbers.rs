// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
//! JavaScript Number reads cannot be represented by a rounded native cell index.
//! Optional raw words retain TypedArray `undefined`, including NaN addressing.
use crate::buffer::{
    constants::{bg_flags, content, fg_flags},
    AttributeData, BufferLine, CellData, ExtendedAttrs,
};
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NumericCellError {
    MissingCombinedString,
    MissingExtendedAttributes,
}
impl fmt::Display for NumericCellError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::MissingCombinedString => "Combined cell string is undefined",
            Self::MissingExtendedAttributes => "Extended cell attributes are undefined",
        })
    }
}
impl std::error::Error for NumericCellError {}

#[derive(Debug)]
pub struct NumericCell {
    pub content: Option<u32>,
    pub fg: Option<u32>,
    pub bg: Option<u32>,
    pub combined_data: Option<Vec<u16>>,
    pub extended: Option<ExtendedAttrs>,
}
impl Clone for NumericCell {
    fn clone(&self) -> Self {
        Self {
            content: self.content,
            fg: self.fg,
            bg: self.bg,
            combined_data: self.combined_data.clone(),
            extended: self.extended.as_ref().map(ExtendedAttrs::share),
        }
    }
}
impl From<CellData> for NumericCell {
    fn from(cell: CellData) -> Self {
        Self {
            content: Some(cell.content),
            fg: Some(cell.attributes.fg),
            bg: Some(cell.attributes.bg),
            combined_data: Some(cell.combined_data),
            extended: Some(cell.attributes.extended),
        }
    }
}
impl NumericCell {
    /// Bitwise JavaScript operations coerce missing words to zero. Callers of
    /// strict equality and string/extended getters must use the methods below.
    pub fn normalized(&self) -> CellData {
        CellData {
            content: self.content.unwrap_or(0),
            attributes: AttributeData {
                fg: self.fg.unwrap_or(0),
                bg: self.bg.unwrap_or(0),
                extended: self
                    .extended
                    .as_ref()
                    .map(ExtendedAttrs::share)
                    .unwrap_or_default(),
            },
            combined_data: self.combined_data.clone().unwrap_or_default(),
        }
    }
    pub fn chars(&self) -> Option<Vec<u16>> {
        if self.content.unwrap_or(0) & content::IS_COMBINED_MASK != 0 {
            self.combined_data.clone()
        } else {
            Some(self.normalized().get_chars())
        }
    }
    pub fn code(&self) -> Result<Option<u32>, NumericCellError> {
        if self.content.unwrap_or(0) & content::IS_COMBINED_MASK != 0 {
            let data = self
                .combined_data
                .as_ref()
                .ok_or(NumericCellError::MissingCombinedString)?;
            Ok(data.last().copied().map(u32::from))
        } else {
            Ok(Some(self.content.unwrap_or(0) & content::CODEPOINT_MASK))
        }
    }
    pub fn is_attribute_default(&self) -> bool {
        self.fg == Some(0) && self.bg == Some(0)
    }
    pub fn require_extended(&self) -> Result<(), NumericCellError> {
        if self.bg.unwrap_or(0) & bg_flags::HAS_EXTENDED != 0 && self.extended.is_none() {
            Err(NumericCellError::MissingExtendedAttributes)
        } else {
            Ok(())
        }
    }
    pub fn require_underline_style(&self) -> Result<(), NumericCellError> {
        if self.fg.unwrap_or(0) & fg_flags::UNDERLINE != 0 {
            self.require_extended()
        } else {
            Ok(())
        }
    }
    pub fn attributes_equals(&self, other: &Self) -> Result<bool, NumericCellError> {
        let a = self.normalized();
        let b = other.normalized();
        // Preserve the source's early exits before its first extended access.
        if a.get_fg_color_mode() != b.get_fg_color_mode()
            || a.get_fg_color() != b.get_fg_color()
            || a.get_bg_color_mode() != b.get_bg_color_mode()
            || a.get_bg_color() != b.get_bg_color()
            || a.is_inverse() != b.is_inverse()
            || a.is_bold() != b.is_bold()
        {
            return Ok(false);
        }
        self.require_extended()?;
        other.require_extended()?;
        Ok(a.attributes_equals(&b))
    }
}

pub fn get_cell_number(line: &BufferLine, index: f64) -> Option<NumericCell> {
    if index < 0.0 || index >= line.len() as f64 {
        return None;
    }
    let start = index * 3.0;
    let content = line.numeric_word(start);
    let fg = line.numeric_word(start + 1.0);
    let bg = line.numeric_word(start + 2.0);
    Some(NumericCell {
        content,
        fg,
        bg,
        combined_data: if content.unwrap_or(0) & content::IS_COMBINED_MASK != 0 {
            line.numeric_combined(index)
        } else {
            Some(Vec::new())
        },
        extended: if bg.unwrap_or(0) & bg_flags::HAS_EXTENDED != 0 {
            line.numeric_extended(index)
        } else {
            Some(ExtendedAttrs::default())
        },
    })
}
