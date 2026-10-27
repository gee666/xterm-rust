// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
//! The source parser operates on UTF-16, not Unicode scalar values.

use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParseError {
    UnterminatedString,
    InvalidCodePoint(u32),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnterminatedString => f.write_str("Unterminated string"),
            Self::InvalidCodePoint(value) => write!(f, "Invalid code point {value}"),
        }
    }
}

impl std::error::Error for ParseError {}

fn whitespace(unit: u16) -> bool {
    matches!(unit, 0x0009..=0x000D | 0x0020 | 0x00A0 | 0x1680 | 0x2000..=0x200A | 0x2028 | 0x2029 | 0x202F | 0x205F | 0x3000 | 0xFEFF)
}

fn hex(unit: u16) -> Option<u32> {
    match unit {
        0x30..=0x39 => Some(u32::from(unit - 0x30)),
        0x41..=0x46 => Some(u32::from(unit - 0x41) + 10),
        0x61..=0x66 => Some(u32::from(unit - 0x61) + 10),
        _ => None,
    }
}

struct Context<'a> {
    input: &'a [u16],
    offset: usize,
}

impl Context<'_> {
    fn unicode(&mut self, output: &mut Vec<u16>) -> Result<(), ParseError> {
        let mut value = 0_u32;
        let mut digits = 0;
        while let Some(&unit) = self.input.get(self.offset) {
            if whitespace(unit) {
                self.offset += 1;
                break;
            }
            if digits == 6 {
                break;
            }
            let Some(digit) = hex(unit) else { break };
            value = value * 16 + digit;
            digits += 1;
            self.offset += 1;
        }
        if value > 0x10FFFF {
            return Err(ParseError::InvalidCodePoint(value));
        }
        // String.fromCodePoint deliberately allows isolated surrogate values.
        if let Ok(unit) = u16::try_from(value) {
            output.push(unit);
        } else {
            let supplementary = value - 0x10000;
            output.push(0xD800 | (supplementary >> 10) as u16);
            output.push(0xDC00 | (supplementary & 0x3FF) as u16);
        }
        Ok(())
    }

    fn component(&mut self, quote: Option<u16>) -> Result<Vec<u16>, ParseError> {
        let mut output = Vec::new();
        let mut escaped = false;
        while let Some(&unit) = self.input.get(self.offset) {
            self.offset += 1;
            if escaped {
                if hex(unit).is_some() {
                    self.offset -= 1;
                    self.unicode(&mut output)?;
                } else if quote.is_none() || unit != 0x0A {
                    output.push(unit);
                }
                escaped = false;
            } else if quote == Some(unit) || (quote.is_none() && unit == 0x2C) {
                return Ok(output);
            } else if unit == 0x5C {
                escaped = true;
            } else if quote.is_none() && whitespace(unit) {
                if output.last() != Some(&0x20) {
                    output.push(0x20);
                }
            } else {
                output.push(unit);
            }
        }
        if quote.is_some() {
            Err(ParseError::UnterminatedString)
        } else {
            Ok(output)
        }
    }
}

/// Parse exactly as the upstream ligatures CSS parser, including empty entries
/// and its omission of a final quoted component without a comma.
pub fn parse_families(input: &[u16]) -> Result<Vec<Vec<u16>>, ParseError> {
    let mut context = Context { input, offset: 0 };
    let mut families = Vec::new();
    let mut current = Vec::new();
    while let Some(&unit) = input.get(context.offset) {
        context.offset += 1;
        match unit {
            0x22 | 0x27 => current.extend(context.component(Some(unit))?),
            0x2C => families.push(std::mem::take(&mut current)),
            _ if !whitespace(unit) => {
                context.offset -= 1;
                current.extend(context.component(None)?);
                families.push(std::mem::take(&mut current));
            }
            _ => {}
        }
    }
    Ok(families)
}
