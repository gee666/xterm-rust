// Copyright (c) 2023 The xterm.js authors. All rights reserved.
// Licensed under MIT. Port of addon-clipboard Base64.

//! Browser-compatible UTF-8 and forgiving base64. Invalid base64 clears the
//! clipboard, matching the source implementation rather than its throws docs.

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub fn encode_text(data: &[u16]) -> String {
    let text = String::from_utf16_lossy(data);
    let mut output = String::with_capacity(text.len().div_ceil(3) * 4);
    for chunk in text.as_bytes().chunks(3) {
        let word = (u32::from(chunk[0]) << 16)
            | (u32::from(*chunk.get(1).unwrap_or(&0)) << 8)
            | u32::from(*chunk.get(2).unwrap_or(&0));
        for shift in [18, 12] {
            output.push(ALPHABET[((word >> shift) & 63) as usize] as char);
        }
        output.push(if chunk.len() > 1 {
            ALPHABET[((word >> 6) & 63) as usize] as char
        } else {
            '='
        });
        output.push(if chunk.len() > 2 {
            ALPHABET[(word & 63) as usize] as char
        } else {
            '='
        });
    }
    output
}

pub fn decode_text(data: &[u16]) -> Vec<u16> {
    let Some(bytes) = decode_bytes(data) else {
        return Vec::new();
    };
    let text = String::from_utf8_lossy(&bytes);
    // TextDecoder consumes one initial BOM by default.
    text.strip_prefix('\u{feff}')
        .unwrap_or(&text)
        .encode_utf16()
        .collect()
}

fn decode_bytes(data: &[u16]) -> Option<Vec<u8>> {
    let mut units: Vec<u16> = data
        .iter()
        .copied()
        .filter(|u| !matches!(u, 9 | 10 | 12 | 13 | 32))
        .collect();
    if units.len() % 4 == 0 {
        if units.last() == Some(&61) {
            units.pop();
        }
        if units.last() == Some(&61) {
            units.pop();
        }
    }
    if units.len() % 4 == 1 {
        return None;
    }
    let mut output = Vec::with_capacity(units.len() * 3 / 4);
    let mut accumulator = 0_u32;
    let mut bits = 0;
    for unit in units {
        let digit = match unit {
            65..=90 => unit - 65,
            97..=122 => unit - 97 + 26,
            48..=57 => unit - 48 + 52,
            43 => 62,
            47 => 63,
            _ => return None,
        };
        accumulator = (accumulator << 6) | u32::from(digit);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            output.push(((accumulator >> bits) & 255) as u8);
        }
    }
    Some(output)
}
