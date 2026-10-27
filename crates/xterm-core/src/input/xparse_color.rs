// Copyright (c) 2021 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. Ported from common/input/XParseColor.ts.

//! X11 color specifications used by OSC color queries and changes.

/// Parse an X11 `rgb:r/g/b` or `#RGB` specification into 8-bit channels.
///
/// Each channel must have the same width, from one to four hex digits.
/// `rgb:` scales and rounds channels to 8 bits. Hash syntax instead takes
/// the most significant eight bits, padding a single digit with a low zero.
/// Named colors, `rgbi:`, whitespace and non-ASCII digits are unsupported.
pub fn parse_color(data: &str) -> Option<[u8; 3]> {
    let bytes = data.as_bytes();
    if bytes
        .get(..4)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case(b"rgb:"))
    {
        let mut parts = data[4..].split('/');
        let channels = [parts.next()?, parts.next()?, parts.next()?];
        let width = channels[0].len();
        if parts.next().is_some()
            || !(1..=4).contains(&width)
            || channels.iter().any(|channel| channel.len() != width)
        {
            return None;
        }
        let base = (1_u32 << (4 * width)) - 1;
        let mut result = [0; 3];
        for (out, channel) in result.iter_mut().zip(channels) {
            let value = hex(channel.as_bytes())?;
            // All bases are odd, so nearest-integer rounding has no ties.
            *out = ((value * 255 + base / 2) / base) as u8;
        }
        Some(result)
    } else if bytes.first() == Some(&b'#') {
        let digits = &bytes[1..];
        if ![3, 6, 9, 12].contains(&digits.len()) {
            return None;
        }
        let width = digits.len() / 3;
        let mut result = [0; 3];
        for (out, channel) in result.iter_mut().zip(digits.chunks_exact(width)) {
            let value = hex(channel)?;
            *out = if width == 1 {
                (value << 4) as u8
            } else {
                (value >> (4 * (width - 2))) as u8
            };
        }
        Some(result)
    } else {
        None
    }
}

fn hex(digits: &[u8]) -> Option<u32> {
    digits.iter().try_fold(0, |value, digit| {
        let nibble = match digit {
            b'0'..=b'9' => digit - b'0',
            b'a'..=b'f' => digit - b'a' + 10,
            b'A'..=b'F' => digit - b'A' + 10,
            _ => return None,
        };
        Some(value * 16 + u32::from(nibble))
    })
}

/// Format 8-bit channels as an X11 `rgb:` specification.
///
/// `None` selects the upstream default of 16 bits per channel. Depths 4, 8
/// and 12 select those widths; every other value uses 16 bits, like upstream.
/// Four-bit output uses the first unpadded hex digit, including for values
/// below 16. Higher depths repeat the 8-bit representation then truncate.
pub fn to_rgb_string(color: [u8; 3], bits: Option<u32>) -> String {
    let [r, g, b] = color.map(|channel| pad(channel, bits.unwrap_or(16)));
    format!("rgb:{r}/{g}/{b}")
}

fn pad(channel: u8, bits: u32) -> String {
    match bits {
        4 => format!("{channel:x}")[..1].to_owned(),
        8 => format!("{channel:02x}"),
        12 => format!("{channel:02x}{:x}", channel >> 4),
        _ => format!("{channel:02x}{channel:02x}"),
    }
}
