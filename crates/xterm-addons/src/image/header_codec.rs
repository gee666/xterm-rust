// Copyright (c) 2023 The xterm.js authors. MIT license.
//! Header names use Node Buffer or browser atob policy, not strict payload base64.
use super::header::HeaderValue;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NameDecodePolicy {
    #[default]
    NodeBuffer,
    BrowserAtob,
}

pub(super) fn decode(key: &[u16], data: &[u32], policy: NameDecodePolicy) -> Option<HeaderValue> {
    let name = String::from_utf16(key).ok();
    match name.as_deref() {
        Some("inline" | "size" | "preserveAspectRatio") => {
            let mut value = 0.0;
            for &digit in data {
                if !(48..=57).contains(&digit) {
                    return None;
                }
                value = value * 10.0 + f64::from(digit) - 48.0;
            }
            Some(HeaderValue::Number(value))
        }
        Some("width" | "height") => {
            let units = units(data);
            if units == [97, 117, 116, 111] {
                return Some(HeaderValue::Text(units));
            }
            let end = if units.ends_with(&[112, 120]) {
                units.len() - 2
            } else if units.ends_with(&[37]) {
                units.len() - 1
            } else {
                units.len()
            };
            if end == 0 || !units[..end].iter().all(|v| (48..=57).contains(v)) {
                return None;
            }
            Some(HeaderValue::Text(units))
        }
        Some("name") => {
            let input = units(data);
            let bytes = match policy {
                NameDecodePolicy::NodeBuffer => node_base64(&input),
                NameDecodePolicy::BrowserAtob => crate::base64::decode_bytes(&input)?,
            };
            let text = String::from_utf8_lossy(&bytes);
            let text = if policy == NameDecodePolicy::BrowserAtob {
                text.strip_prefix('\u{feff}').unwrap_or(&text)
            } else {
                &text
            };
            Some(HeaderValue::Text(text.encode_utf16().collect()))
        }
        Some("__proto__" | "__defineGetter__" | "__defineSetter__") => None,
        Some("toString" | "toLocaleString") => Some(HeaderValue::Text(
            "[object Object]".encode_utf16().collect(),
        )),
        Some("hasOwnProperty" | "isPrototypeOf" | "propertyIsEnumerable") => {
            Some(HeaderValue::Boolean(false))
        }
        Some("__lookupGetter__" | "__lookupSetter__") => Some(HeaderValue::Undefined),
        Some("valueOf") => Some(HeaderValue::DecoderObject),
        _ => Some(HeaderValue::Codepoints(data.to_vec())),
    }
}
fn units(data: &[u32]) -> Vec<u16> {
    data.iter().map(|&code| (code & 0xffff) as u16).collect()
}
fn node_base64(data: &[u16]) -> Vec<u8> {
    let mut result = Vec::with_capacity(data.len() * 3 / 4);
    let mut bits = 0;
    let mut accumulator = 0_u32;
    for &code in data {
        // Buffer's base64 reader accepts the URL alphabet and ignores junk.
        if code == 61 {
            break;
        }
        let digit = match code {
            65..=90 => u32::from(code - 65),
            97..=122 => u32::from(code - 97 + 26),
            48..=57 => u32::from(code - 48 + 52),
            43 | 45 => 62,
            47 | 95 => 63,
            _ => continue,
        };
        accumulator = (accumulator << 6) | digit;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            result.push(((accumulator >> bits) & 255) as u8);
        }
    }
    result
}
