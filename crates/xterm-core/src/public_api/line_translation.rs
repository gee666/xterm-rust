// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
//! Number-preserving BufferLine translation for the browser binding. Typed
//! allocation/non-progress errors replace process exhaustion on hostile ranges.
use crate::{
    buffer::{
        constants::{content, CELL_WORDS},
        BufferLine,
    },
    input::text_decoder::string_from_code_point,
};
use std::{collections::HashSet, fmt};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NumericTranslationError {
    MissingCombinedString,
    NonProgressingRange,
    Allocation,
}
impl fmt::Display for NumericTranslationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::MissingCombinedString => "Combined cell string is undefined",
            Self::NonProgressingRange => "Buffer translation range cannot terminate",
            Self::Allocation => "Buffer translation allocation failed",
        })
    }
}
impl std::error::Error for NumericTranslationError {}

#[derive(Debug)]
pub struct NumericTranslation {
    pub text: Vec<u16>,
    pub columns: Vec<f64>,
}

pub fn translate_to_string_number(
    line: &BufferLine,
    trim_right: bool,
    start: Option<f64>,
    end: Option<f64>,
    with_columns: bool,
) -> Result<NumericTranslation, NumericTranslationError> {
    if start.is_none_or(|value| value == 0.0) && end.is_none() && !with_columns {
        return Ok(NumericTranslation {
            text: line.translate_to_string(trim_right, None, None, None),
            columns: Vec::new(),
        });
    }
    let mut position = start.unwrap_or(0.0);
    let mut end = end.unwrap_or(line.len() as f64);
    if trim_right && !end.is_nan() {
        end = end.min(line.get_trimmed_length() as f64);
    }
    let mut text = Vec::new();
    let mut columns = Vec::new();
    // Malformed packed widths can move backward and revisit an address. Detect
    // that cycle without imposing an arbitrary limit on valid line lengths.
    let mut visited = HashSet::new();
    while position < end {
        visited
            .try_reserve(1)
            .map_err(|_| NumericTranslationError::Allocation)?;
        if !position.is_finite() || !end.is_finite() || !visited.insert(position.to_bits()) {
            return Err(NumericTranslationError::NonProgressingRange);
        }
        let packed = line.numeric_word(position * CELL_WORDS as f64).unwrap_or(0);
        let chars = if packed & content::IS_COMBINED_MASK != 0 {
            line.numeric_combined(position)
        } else {
            let point = packed & content::CODEPOINT_MASK;
            Some(if point == 0 {
                vec![32]
            } else {
                string_from_code_point(point)
            })
        };
        match chars {
            Some(chars) => {
                text.try_reserve(chars.len())
                    .map_err(|_| NumericTranslationError::Allocation)?;
                if with_columns {
                    columns
                        .try_reserve(chars.len())
                        .map_err(|_| NumericTranslationError::Allocation)?;
                    columns.extend(std::iter::repeat_n(position, chars.len()));
                }
                text.extend(chars);
            }
            None if with_columns => return Err(NumericTranslationError::MissingCombinedString),
            // Array.join renders an undefined element as an empty string.
            None => {}
        }
        let width = (packed as i32) >> content::WIDTH_SHIFT;
        let next = position + f64::from(if width == 0 { 1 } else { width });
        if next == position {
            return Err(NumericTranslationError::NonProgressingRange);
        }
        position = next;
    }
    if with_columns {
        columns
            .try_reserve(1)
            .map_err(|_| NumericTranslationError::Allocation)?;
        columns.push(position);
    }
    Ok(NumericTranslation { text, columns })
}
