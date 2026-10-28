// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
//! Lossless command text decoding. UTF-16 escapes may contain lone surrogates.
use std::{error::Error, fmt};
#[derive(Debug, PartialEq, Eq)]
pub struct DecodeError {
    index: usize,
    reason: &'static str,
}
impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "escape at byte {}: {}", self.index, self.reason)
    }
}
impl Error for DecodeError {}

pub fn decode(text: &str) -> Result<Vec<u16>, DecodeError> {
    let mut output = Vec::new();
    let mut chars = text.char_indices();
    while let Some((index, character)) = chars.next() {
        if character != '\\' {
            output.extend_from_slice(character.encode_utf16(&mut [0; 2]));
            continue;
        }
        let (_, escaped) = chars.next().ok_or(DecodeError {
            index,
            reason: "missing escape character",
        })?;
        let unit = match escaped {
            'e' => 0x1b,
            'n' => 0x0a,
            'r' => 0x0d,
            't' => 0x09,
            '0' => 0,
            '\\' => u16::from(b'\\'),
            'x' | 'u' => {
                let mut value = 0_u16;
                for _ in 0..if escaped == 'x' { 2 } else { 4 } {
                    let (_, digit) = chars.next().ok_or(DecodeError {
                        index,
                        reason: "incomplete hexadecimal escape",
                    })?;
                    let digit = digit.to_digit(16).ok_or(DecodeError {
                        index,
                        reason: "expected hexadecimal digit",
                    })?;
                    // At most four hexadecimal digits fit exactly in u16.
                    value = (value << 4) | digit as u16;
                }
                value
            }
            _ => {
                return Err(DecodeError {
                    index,
                    reason: "unknown escape",
                })
            }
        };
        output.push(unit);
    }
    Ok(output)
}

/// A terminal-safe representation that preserves every UTF-16 unit.
pub fn encode(units: &[u16]) -> String {
    let mut output = String::new();
    for &unit in units {
        match unit {
            0x1b => output.push_str("\\e"),
            0x0a => output.push_str("\\n"),
            0x0d => output.push_str("\\r"),
            0x09 => output.push_str("\\t"),
            0 => output.push_str("\\0"),
            92 => output.push_str("\\\\"),
            // The ASCII range guarantees this narrowing is lossless.
            32..=126 => output.push(char::from(unit as u8)),
            _ => output.push_str(&format!("\\u{unit:04X}")),
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn decodes_controls_utf16_pairs_and_literal_unicode() {
        assert_eq!(
            decode("\\e[31m\\r\\n\\t\\x41\\uD83D\\uDE00界").unwrap(),
            "\x1b[31m\r\n\tA😀界".encode_utf16().collect::<Vec<_>>()
        );
        assert_eq!(decode("\\uD800").unwrap(), [0xd800]);
        assert_eq!(decode("\\\\\\0").unwrap(), [92, 0]);
    }
    #[test]
    fn encoded_observations_roundtrip_every_utf16_unit() {
        let units: Vec<u16> = (0..=u16::MAX).collect();
        assert_eq!(decode(&encode(&units)).unwrap(), units);
    }

    #[test]
    fn malformed_sequences_return_errors() {
        for input in ["\\", "\\x", "\\xG0", "\\u123", "\\q"] {
            assert!(decode(input).is_err(), "{input}");
        }
    }
}
