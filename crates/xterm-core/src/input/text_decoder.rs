// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. Port of common/input/TextDecoder.ts.

//! Streaming decoders preserve xterm.js's treatment of malformed input.
//!
//! JavaScript strings map to UTF-16 units, not Rust `str`, because lone
//! surrogates are observable. Vector APIs append and report new values. Fixed
//! slice APIs preserve JS typed-array overflow drops and logical write counts.

#[path = "text_decoder_utf8.rs"]
mod utf8_pending;

/// Convert a code point to JavaScript UTF-16 units, including lone surrogates.
/// Out-of-range values retain the upstream `String.fromCharCode` truncation.
pub fn string_from_code_point(code_point: u32) -> Vec<u16> {
    let mut result = Vec::with_capacity(2);
    push_code_point(code_point, &mut result);
    result
}

/// Convert UTF-32 values into JavaScript string units.
/// Slice the input to select the upstream `start..end` range.
pub fn utf32_to_string(data: &[u32]) -> Vec<u16> {
    let mut result = Vec::with_capacity(data.len());
    for &code_point in data {
        push_code_point(code_point, &mut result);
    }
    result
}

fn push_code_point(code_point: u32, target: &mut Vec<u16>) {
    if code_point > 0xffff {
        let value = code_point - 0x10000;
        // JavaScript >> coerces to signed 32-bit before shifting.
        target.push(((value as i32 >> 10) + 0xd800) as u16);
        target.push(((value % 0x400) + 0xdc00) as u16);
    } else {
        target.push(code_point as u16);
    }
}

trait Utf32Output {
    fn position(&self) -> usize;
    fn push(&mut self, value: u32);
    fn pair(&mut self, first: u32, second: u32) {
        self.push(first);
        self.push(second);
    }
}

impl Utf32Output for Vec<u32> {
    fn position(&self) -> usize {
        self.len()
    }
    fn push(&mut self, value: u32) {
        Vec::push(self, value);
    }
    fn pair(&mut self, first: u32, second: u32) {
        self.extend([first, second]);
    }
}

struct SliceOutput<'a> {
    data: &'a mut [u32],
    attempted: usize,
}
impl Utf32Output for SliceOutput<'_> {
    fn position(&self) -> usize {
        self.attempted
    }
    fn push(&mut self, value: u32) {
        if let Some(slot) = self.data.get_mut(self.attempted) {
            *slot = value;
        }
        // Output is bounded by input length plus one pending surrogate/prefix.
        self.attempted += 1;
    }
}

/// Streaming UTF-16 decoder. Lone surrogates pass through as UCS-2 values.
#[derive(Debug, Default)]
pub struct StringToUtf32 {
    interim: Option<u16>,
}

impl StringToUtf32 {
    pub fn new() -> Self {
        Self::default()
    }

    /// Discard any pending high surrogate.
    pub fn clear(&mut self) {
        self.interim = None;
    }

    /// Append decoded values, retaining a final high surrogate for the next call.
    /// Empty input does not flush pending state. U+FEFF is dropped except when
    /// it immediately follows an unpaired high surrogate, matching upstream.
    pub fn decode(&mut self, input: &[u16], target: &mut Vec<u32>) -> usize {
        self.decode_into(input, target)
    }

    /// Write from index zero, discarding out-of-bounds writes like a JS typed
    /// array. Returns the logical count, which may exceed target.len().
    pub fn decode_to_slice(&mut self, input: &[u16], target: &mut [u32]) -> usize {
        self.decode_into(
            input,
            &mut SliceOutput {
                data: target,
                attempted: 0,
            },
        )
    }

    fn decode_into<T: Utf32Output>(&mut self, input: &[u16], target: &mut T) -> usize {
        if input.is_empty() {
            return 0;
        }
        let original_len = target.position();
        let mut index = 0;
        if let Some(high) = self.interim.take() {
            decode_pair(high, input[0], target);
            index = 1;
        }
        while index < input.len() {
            let code = input[index];
            index += 1;
            if (0xd800..=0xdbff).contains(&code) {
                if index == input.len() {
                    self.interim = Some(code);
                    break;
                }
                decode_pair(code, input[index], target);
                index += 1;
            } else if code != 0xfeff {
                target.push(u32::from(code));
            }
        }
        target.position() - original_len
    }
}

fn decode_pair<T: Utf32Output>(high: u16, second: u16, target: &mut T) {
    if (0xdc00..=0xdfff).contains(&second) {
        target.push((u32::from(high) - 0xd800) * 0x400 + u32::from(second) - 0xdc00 + 0x10000);
    } else {
        target.pair(u32::from(high), u32::from(second));
    }
}

/// Streaming UTF-8 decoder. Invalid sequences are discarded, not replaced.
#[derive(Debug, Default)]
pub struct Utf8ToUtf32 {
    /// Pending prefix bytes, zero padded, matching upstream's public state.
    pub interim: [u8; 3],
}

impl Utf8ToUtf32 {
    pub fn new() -> Self {
        Self::default()
    }

    /// Discard the pending prefix.
    pub fn clear(&mut self) {
        self.interim.fill(0);
    }

    /// Append decoded code points. BOM, overlong sequences, surrogate values,
    /// invalid starters and out-of-range values are discarded. A noncontinuation
    /// byte is retried as a starter. Empty input does not flush the prefix.
    pub fn decode(&mut self, input: &[u8], target: &mut Vec<u32>) -> usize {
        self.decode_into(input, target)
    }

    /// Write from index zero, discarding out-of-bounds writes like a JS typed
    /// array. Returns the logical count, which may exceed target.len().
    pub fn decode_to_slice(&mut self, input: &[u8], target: &mut [u32]) -> usize {
        self.decode_into(
            input,
            &mut SliceOutput {
                data: target,
                attempted: 0,
            },
        )
    }

    fn decode_into<T: Utf32Output>(&mut self, input: &[u8], target: &mut T) -> usize {
        if input.is_empty() {
            return 0;
        }
        let original_len = target.position();
        let mut index = 0;
        if self.interim[0] != 0 {
            let Some(start) = self.decode_pending(input, target) else {
                return 0;
            };
            if start < 0 {
                // With a manually completed overlong public prefix, the JS
                // ASCII shortcut reads input[-1] and stores it as typed-array0.
                if input.len() >= 4 && input[..3].iter().all(u8::is_ascii) {
                    target.push(0);
                    for &byte in &input[..3] {
                        target.push(u32::from(byte));
                    }
                    index = 3;
                }
            } else {
                index = start as usize;
            }
        }
        while index < input.len() {
            let first = input[index];
            index += 1;
            if first < 0x80 {
                target.push(u32::from(first));
            } else if let Some(length) = sequence_length(first) {
                self.decode_sequence([first, 0, 0, 0], 1, length, input, &mut index, target);
                if self.interim[0] != 0 {
                    break;
                }
            }
        }
        target.position() - original_len
    }

    fn decode_sequence<T: Utf32Output>(
        &mut self,
        mut sequence: [u8; 4],
        mut count: usize,
        length: usize,
        input: &[u8],
        index: &mut usize,
        target: &mut T,
    ) {
        while count < length {
            let Some(&byte) = input.get(*index) else {
                self.interim[..count].copy_from_slice(&sequence[..count]);
                return;
            };
            if byte & 0xc0 != 0x80 {
                return;
            }
            sequence[count] = byte;
            count += 1;
            *index += 1;
        }
        let mask = match length {
            2 => 0x1f,
            3 => 0x0f,
            _ => 0x07,
        };
        let mut code_point = u32::from(sequence[0] & mask);
        for &byte in &sequence[1..length] {
            code_point = (code_point << 6) | u32::from(byte & 0x3f);
        }
        let valid = match length {
            2 => code_point >= 0x80,
            3 => {
                code_point >= 0x800
                    && !(0xd800..=0xdfff).contains(&code_point)
                    && code_point != 0xfeff
            }
            _ => (0x10000..=0x10ffff).contains(&code_point),
        };
        if valid {
            target.push(code_point);
        }
        // Upstream retries an overlong two-byte sequence's final byte. It is
        // always a continuation and immediately discarded, so consuming it
        // here gives the same output without another iteration.
    }
}

fn sequence_length(first: u8) -> Option<usize> {
    if first & 0xe0 == 0xc0 {
        Some(2)
    } else if first & 0xf0 == 0xe0 {
        Some(3)
    } else if first & 0xf8 == 0xf0 {
        Some(4)
    } else {
        None
    }
}
